use gramgraph::{csv_reader, data::PlotData, parser, runtime, OutputFormat, RenderOptions};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[command(name = "gramgraph")]
#[command(about = "Generate graphs from CSV data using GramGraph DSL", long_about = None)]
struct Args {
    /// GramGraph DSL string (e.g., 'chart(x: time, y: temp) | layer_line(color: "red")')
    dsl: String,

    /// Output width in pixels
    #[arg(long, default_value_t = 800)]
    width: u32,

    /// Output height in pixels
    #[arg(long, default_value_t = 600)]
    height: u32,

    /// Output format (png, svg)
    #[arg(long, value_enum, default_value_t = FormatArg::Png)]
    format: FormatArg,

    /// Define variables for DSL substitution (e.g., -D x=time -D color=red)
    #[arg(short = 'D', long = "define", value_parser = parse_key_val)]
    defines: Vec<(String, String)>,
}

/// Helper parser for key=value pairs
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[derive(Debug, Clone, ValueEnum)]
enum FormatArg {
    Png,
    Svg,
}

impl From<FormatArg> for OutputFormat {
    fn from(arg: FormatArg) -> Self {
        match arg {
            FormatArg::Png => OutputFormat::Png,
            FormatArg::Svg => OutputFormat::Svg,
        }
    }
}

/// Process DSL and CSV data to generate PNG bytes
/// This function is extracted for testability
pub fn process_dsl(
    dsl: &str,
    csv_content: impl Read,
    options: RenderOptions,
    variables: HashMap<String, String>,
) -> Result<Vec<u8>> {
    // 1. Preprocess: Expand variables immediately
    let expanded_dsl = gramgraph::preprocessor::expand_variables(dsl, &variables)
        .context("Failed to expand variables")?;

    // Read CSV
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_content);

    let headers = reader
        .headers()
        .context("Failed to read CSV headers")?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        rows.push(row);
    }

    if rows.is_empty() {
        return Err(anyhow!("CSV must contain at least one data row"));
    }

    let csv_data = csv_reader::CsvData { headers, rows };
    let plot_data = PlotData::from_csv(csv_data);

    // Parse the DSL string
    let plot_spec = match parser::parse_plot_spec(&expanded_dsl) {
        Ok((remaining, plot_spec)) => {
            if !remaining.trim().is_empty() {
                eprintln!("Warning: unparsed input: '{}'", remaining);
            }
            plot_spec
        }
        Err(e) => {
            return Err(anyhow!("Parse error: {:?}", e));
        }
    };

    // Render the plot
    runtime::render_plot(plot_spec, plot_data, options).context("Failed to render plot")
}

fn main() -> Result<()> {
    let args = Args::parse();

    let options = RenderOptions {
        width: args.width,
        height: args.height,
        format: args.format.into(),
    };

    // Convert defines Vec to HashMap
    let variables: HashMap<String, String> = args.defines.into_iter().collect();

    let bytes = process_dsl(&args.dsl, io::stdin(), options, variables)?;

    // Write output to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(&bytes)
        .context("Failed to write output to stdout")?;
    handle.flush().context("Failed to flush stdout")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Cursor;

    #[test]
    fn test_process_dsl_line_chart() {
        let csv = "x,y\n1,10\n2,20\n3,30\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: x, y: y) | line()",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_ok());
        let png_bytes = result.unwrap();
        assert!(png_bytes.len() > 8);
        assert_eq!(&png_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_process_dsl_parse_error() {
        let csv = "x,y\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "invalid syntax here",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_process_dsl_csv_error() {
        let csv = "x,y\n"; // No data rows
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: x, y: y) | line()",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one data row"));
    }

    #[test]
    fn test_process_dsl_column_not_found() {
        let csv = "a,b\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: x, y: y) | line()",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_err());
        // Error is wrapped with context, so check for the context message
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to render plot"));
    }

    #[test]
    fn test_process_dsl_bar_chart() {
        let csv = "cat,val\nA,10\nB,20\nC,30\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: cat, y: val) | bar()",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_multiple_layers() {
        let csv = "x,y\n1,10\n2,20\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: x, y: y) | line() | point()",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_unparsed_input() {
        // Trailing unparsed input causes parse error
        let csv = "x,y\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "line() extra_stuff",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_process_dsl_empty_input() {
        let csv = "x,y\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("", cursor, RenderOptions::default(), HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_process_dsl_unicode_data() {
        let csv = "x,température\n1,20.5\n2,22.0\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: x, y: température) | line()",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_point_chart() {
        let csv = "height,weight\n170,70\n180,85\n160,60\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: height, y: weight) | point(size: 5)",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_with_variables() {
        // Test variable substitution in aes()
        let csv = "time,temp\n1,20\n2,25\n3,30\n";
        let cursor = Cursor::new(csv);
        let mut vars = HashMap::new();
        vars.insert("xcol".to_string(), "time".to_string());
        vars.insert("ycol".to_string(), "temp".to_string());
        let result = process_dsl(
            "aes(x: $xcol, y: $ycol) | line()",
            cursor,
            RenderOptions::default(),
            vars,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_variable_in_geom() {
        // Test variable substitution in geometry
        let csv = "x,y\n1,10\n2,20\n";
        let cursor = Cursor::new(csv);
        let mut vars = HashMap::new();
        // Quote the value to make it a string literal
        vars.insert("line_color".to_string(), "\"red\"".to_string());
        let result = process_dsl(
            "aes(x: x, y: y) | line(color: $line_color)",
            cursor,
            RenderOptions::default(),
            vars,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_undefined_variable() {
        // Test that undefined variables cause an error
        let csv = "x,y\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl(
            "aes(x: $undefined, y: y) | line()",
            cursor,
            RenderOptions::default(),
            HashMap::new(),
        );
        assert!(result.is_err());
        // Check the full error chain
        let err_str = format!("{:?}", result.unwrap_err());
        assert!(err_str.contains("Variable '$undefined' not defined"));
    }
}

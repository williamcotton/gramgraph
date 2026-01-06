mod csv_reader;
mod graph;
mod palette;
mod parser;
mod runtime;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use csv::ReaderBuilder;
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[command(name = "gramgraph")]
#[command(about = "Generate graphs from CSV data using GramGraph DSL", long_about = None)]
struct Args {
    /// GramGraph DSL string (e.g., 'chart(x: time, y: temp) | layer_line(color: "red")')
    dsl: String,
}

/// Process DSL and CSV data to generate PNG bytes
/// This function is extracted for testability
pub fn process_dsl(dsl: &str, csv_content: impl Read) -> Result<Vec<u8>> {
    // Read CSV
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(csv_content);

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

    // Parse the DSL string
    let plot_spec = match parser::parse_plot_spec(dsl) {
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
    runtime::render_plot(plot_spec, csv_data).context("Failed to render plot")
}

fn main() -> Result<()> {
    let args = Args::parse();
    let png_bytes = process_dsl(&args.dsl, io::stdin())?;

    // Write PNG to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(&png_bytes)
        .context("Failed to write PNG to stdout")?;
    handle.flush().context("Failed to flush stdout")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_process_dsl_line_chart() {
        let csv = "x,y\n1,10\n2,20\n3,30\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("aes(x: x, y: y) | line()", cursor);
        assert!(result.is_ok());
        let png_bytes = result.unwrap();
        assert!(png_bytes.len() > 8);
        assert_eq!(&png_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_process_dsl_parse_error() {
        let csv = "x,y\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("invalid syntax here", cursor);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_process_dsl_csv_error() {
        let csv = "x,y\n"; // No data rows
        let cursor = Cursor::new(csv);
        let result = process_dsl("aes(x: x, y: y) | line()", cursor);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one data row"));
    }

    #[test]
    fn test_process_dsl_column_not_found() {
        let csv = "a,b\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("aes(x: x, y: y) | line()", cursor);
        assert!(result.is_err());
        // Error is wrapped with context, so check for the context message
        assert!(result.unwrap_err().to_string().contains("Failed to render plot"));
    }

    #[test]
    fn test_process_dsl_bar_chart() {
        let csv = "cat,val\nA,10\nB,20\nC,30\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("aes(x: cat, y: val) | bar()", cursor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_multiple_layers() {
        let csv = "x,y\n1,10\n2,20\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("aes(x: x, y: y) | line() | point()", cursor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_unparsed_input() {
        // Trailing unparsed input causes parse error
        let csv = "x,y\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("line() extra_stuff", cursor);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_process_dsl_empty_input() {
        let csv = "x,y\n1,10\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("", cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_dsl_unicode_data() {
        let csv = "x,température\n1,20.5\n2,22.0\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("aes(x: x, y: température) | line()", cursor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_dsl_point_chart() {
        let csv = "height,weight\n170,70\n180,85\n160,60\n";
        let cursor = Cursor::new(csv);
        let result = process_dsl("aes(x: height, y: weight) | point(size: 5)", cursor);
        assert!(result.is_ok());
    }
}

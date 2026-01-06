mod csv_reader;
mod graph;

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "gramgraph")]
#[command(about = "Generate line graphs from CSV data", long_about = None)]
struct Args {
    #[arg(short = 'x', long = "x", required = true, help = "X-axis column (name or 0-based index)")]
    x_column: String,

    #[arg(short = 'y', long = "y", required = true, help = "Y-axis column (name or 0-based index)")]
    y_column: String,

    #[arg(long = "width", default_value = "800", help = "Output width in pixels")]
    width: u32,

    #[arg(long = "height", default_value = "600", help = "Output height in pixels")]
    height: u32,

    #[arg(short = 't', long = "title", help = "Graph title")]
    title: Option<String>,

    #[arg(long = "x-label", help = "X-axis label (defaults to column name)")]
    x_label: Option<String>,

    #[arg(long = "y-label", help = "Y-axis label (defaults to column name)")]
    y_label: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let csv_data = csv_reader::read_csv_from_stdin()
        .context("Failed to read CSV from stdin")?;

    let x_selector = csv_reader::parse_column_selector(&args.x_column);
    let (x_col_name, x_values) = csv_reader::extract_column(&csv_data, x_selector)
        .context("Failed to extract X column")?;

    let y_selector = csv_reader::parse_column_selector(&args.y_column);
    let (y_col_name, y_values) = csv_reader::extract_column(&csv_data, y_selector)
        .context("Failed to extract Y column")?;

    let config = graph::GraphConfig {
        title: args.title,
        x_label: args.x_label.unwrap_or(x_col_name),
        y_label: args.y_label.unwrap_or(y_col_name),
        width: args.width,
        height: args.height,
    };

    let png_bytes = graph::generate_line_graph(x_values, y_values, config)
        .context("Failed to generate graph")?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(&png_bytes)
        .context("Failed to write PNG to stdout")?;
    handle.flush().context("Failed to flush stdout")?;

    Ok(())
}

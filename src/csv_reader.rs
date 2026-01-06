use anyhow::{anyhow, Context, Result};
use csv::ReaderBuilder;
use std::io;

#[derive(Debug)]
pub struct CsvData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub enum ColumnSelector {
    Index(usize),
    Name(String),
}

pub fn read_csv_from_stdin() -> Result<CsvData> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(io::stdin());

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

    Ok(CsvData { headers, rows })
}

pub fn parse_column_selector(input: &str) -> ColumnSelector {
    match input.parse::<usize>() {
        Ok(index) => ColumnSelector::Index(index),
        Err(_) => ColumnSelector::Name(input.to_string()),
    }
}

pub fn extract_column(data: &CsvData, selector: ColumnSelector) -> Result<(String, Vec<f64>)> {
    let (column_index, column_name) = match selector {
        ColumnSelector::Index(idx) => {
            if idx >= data.headers.len() {
                return Err(anyhow!(
                    "Column index {} out of bounds (available columns: {})",
                    idx,
                    data.headers.len()
                ));
            }
            (idx, data.headers[idx].clone())
        }
        ColumnSelector::Name(name) => {
            let idx = data
                .headers
                .iter()
                .position(|h| h.eq_ignore_ascii_case(&name))
                .ok_or_else(|| {
                    anyhow!(
                        "Column '{}' not found. Available columns: {}",
                        name,
                        data.headers.join(", ")
                    )
                })?;
            (idx, data.headers[idx].clone())
        }
    };

    let mut values = Vec::new();
    for (row_idx, row) in data.rows.iter().enumerate() {
        if column_index >= row.len() {
            return Err(anyhow!(
                "Row {} has only {} columns, expected at least {}",
                row_idx + 1,
                row.len(),
                column_index + 1
            ));
        }

        let value_str = &row[column_index];
        let value = value_str.parse::<f64>().with_context(|| {
            format!(
                "Failed to parse value '{}' as number in column '{}' at row {}",
                value_str,
                column_name,
                row_idx + 1
            )
        })?;
        values.push(value);
    }

    Ok((column_name, values))
}

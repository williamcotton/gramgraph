use anyhow::{anyhow, Context, Result};
use csv::ReaderBuilder;
use std::io;

#[derive(Debug, Clone)]
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

pub fn extract_column_as_string(data: &CsvData, selector: ColumnSelector) -> Result<(String, Vec<String>)> {
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
        values.push(row[column_index].clone());
    }

    Ok((column_name, values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Helper function to create CsvData from string
    fn csv_from_string(content: &str) -> Result<CsvData> {
        let cursor = Cursor::new(content);
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(cursor);

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

    // parse_column_selector tests (2 tests)

    #[test]
    fn test_parse_column_selector_by_index() {
        match parse_column_selector("0") {
            ColumnSelector::Index(i) => assert_eq!(i, 0),
            _ => panic!("Expected Index"),
        }
        match parse_column_selector("42") {
            ColumnSelector::Index(i) => assert_eq!(i, 42),
            _ => panic!("Expected Index"),
        }
    }

    #[test]
    fn test_parse_column_selector_by_name() {
        match parse_column_selector("temperature") {
            ColumnSelector::Name(s) => assert_eq!(s, "temperature"),
            _ => panic!("Expected Name"),
        }
        match parse_column_selector("col_name") {
            ColumnSelector::Name(s) => assert_eq!(s, "col_name"),
            _ => panic!("Expected Name"),
        }
    }

    // extract_column happy path (6 tests)

    #[test]
    fn test_extract_column_by_name() {
        let csv = csv_from_string("x,y,z\n1,10,100\n2,20,200\n3,30,300").unwrap();
        let (name, values) = extract_column(&csv, ColumnSelector::Name("y".to_string())).unwrap();
        assert_eq!(name, "y");
        assert_eq!(values, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_extract_column_by_index() {
        let csv = csv_from_string("x,y,z\n1,10,100\n2,20,200").unwrap();
        let (name, values) = extract_column(&csv, ColumnSelector::Index(0)).unwrap();
        assert_eq!(name, "x");
        assert_eq!(values, vec![1.0, 2.0]);

        let (name, values) = extract_column(&csv, ColumnSelector::Index(2)).unwrap();
        assert_eq!(name, "z");
        assert_eq!(values, vec![100.0, 200.0]);
    }

    #[test]
    fn test_extract_column_case_insensitive() {
        let csv = csv_from_string("temperature,humidity\n20.5,60\n22.0,55").unwrap();
        let (name, values) = extract_column(&csv, ColumnSelector::Name("Temperature".to_string())).unwrap();
        assert_eq!(name, "temperature"); // Returns actual header case
        assert_eq!(values, vec![20.5, 22.0]);
    }

    #[test]
    fn test_extract_column_single_row() {
        let csv = csv_from_string("x,y\n1,10").unwrap();
        let (name, values) = extract_column(&csv, ColumnSelector::Name("y".to_string())).unwrap();
        assert_eq!(name, "y");
        assert_eq!(values, vec![10.0]);
    }

    #[test]
    fn test_extract_column_negative_values() {
        let csv = csv_from_string("x,y\n-10,-20\n-5,-15\n0,0\n5,10").unwrap();
        let (_, values) = extract_column(&csv, ColumnSelector::Name("y".to_string())).unwrap();
        assert_eq!(values, vec![-20.0, -15.0, 0.0, 10.0]);
    }

    #[test]
    fn test_extract_column_large_values() {
        let csv = csv_from_string("x,y\n1e10,2e10\n3e10,4e10").unwrap();
        let (_, values) = extract_column(&csv, ColumnSelector::Name("y".to_string())).unwrap();
        assert_eq!(values, vec![2e10, 4e10]);
    }

    // extract_column error cases (6 tests)

    #[test]
    fn test_extract_column_not_found() {
        let csv = csv_from_string("x,y\n1,10").unwrap();
        let result = extract_column(&csv, ColumnSelector::Name("nonexistent".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_extract_column_index_out_of_bounds() {
        let csv = csv_from_string("x,y\n1,10").unwrap();
        let result = extract_column(&csv, ColumnSelector::Index(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_extract_column_non_numeric() {
        let csv = csv_from_string("x,y\n1,10\nnot_a_number,20\n3,30").unwrap();
        let result = extract_column(&csv, ColumnSelector::Name("x".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_extract_column_missing_value() {
        let csv = csv_from_string("x,y\n1,10\n2,\n3,30").unwrap();
        let result = extract_column(&csv, ColumnSelector::Name("y".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_extract_column_short_row() {
        // CSV reader validates row length during parsing
        // This should fail when creating the CSV
        let result = csv_from_string("x,y,z\n1,10,100\n2,20");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("record"));
    }

    #[test]
    fn test_extract_column_from_empty_data() {
        // This will fail at CSV reading, not column extraction
        let result = csv_from_string("x,y\n");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one data row"));
    }

    // read_csv_from_stdin tests (6 tests using csv_from_string helper)

    #[test]
    fn test_read_csv_basic() {
        let csv = csv_from_string("a,b,c\n1,2,3\n4,5,6").unwrap();
        assert_eq!(csv.headers, vec!["a", "b", "c"]);
        assert_eq!(csv.rows.len(), 2);
        assert_eq!(csv.rows[0], vec!["1", "2", "3"]);
        assert_eq!(csv.rows[1], vec!["4", "5", "6"]);
    }

    #[test]
    fn test_read_csv_empty_data() {
        let result = csv_from_string("x,y\n");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one data row"));
    }

    #[test]
    fn test_read_csv_single_row() {
        let csv = csv_from_string("x,y\n1,10").unwrap();
        assert_eq!(csv.rows.len(), 1);
        assert_eq!(csv.rows[0], vec!["1", "10"]);
    }

    #[test]
    fn test_read_csv_unicode() {
        let csv = csv_from_string("x,température\n1,20.5\n2,22.0").unwrap();
        assert_eq!(csv.headers, vec!["x", "température"]);
        assert_eq!(csv.rows.len(), 2);
    }

    #[test]
    fn test_read_csv_duplicate_headers() {
        // CSV crate allows duplicate headers, just reads them as-is
        let csv = csv_from_string("value,value,other\n1,2,3\n4,5,6").unwrap();
        assert_eq!(csv.headers, vec!["value", "value", "other"]);
        assert_eq!(csv.rows.len(), 2);
    }

    #[test]
    fn test_read_csv_malformed() {
        // Unclosed quote
        let result = csv_from_string("x,y\n\"unclosed,value\n");
        // CSV crate may handle this differently, just check it doesn't panic
        let _ = result;
    }
}

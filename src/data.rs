use anyhow::{anyhow, Result};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct PlotData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl PlotData {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self { headers, rows }
    }

    /// Create PlotData from an existing CsvData struct (for legacy/CLI support)
    pub fn from_csv(csv: crate::csv_reader::CsvData) -> Self {
        Self {
            headers: csv.headers,
            rows: csv.rows,
        }
    }

    /// Create PlotData from a JSON Array of Objects
    pub fn from_json(value: &Value) -> Result<Self> {
        let array = value
            .as_array()
            .ok_or_else(|| anyhow!("Input data must be a JSON array of objects"))?;

        if array.is_empty() {
            return Err(anyhow!("Input data array is empty"));
        }

        // Extract headers from the first object
        let first_obj = array[0]
            .as_object()
            .ok_or_else(|| anyhow!("Items in array must be objects"))?;

        let headers: Vec<String> = first_obj.keys().cloned().collect();

        let mut rows = Vec::new();
        for item in array {
            let obj = item
                .as_object()
                .ok_or_else(|| anyhow!("Items in array must be objects"))?;

            let mut row = Vec::new();
            for header in &headers {
                let val_str = match obj.get(header) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    Some(Value::Null) | None => "".to_string(),
                    _ => return Err(anyhow!("Unsupported value type for field '{}'", header)),
                };
                row.push(val_str);
            }
            rows.push(row);
        }

        Ok(Self { headers, rows })
    }
}

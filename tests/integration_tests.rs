use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

/// Helper function to run gramgraph with DSL and CSV input
fn run_gramgraph(dsl: &str, csv_content: &str) -> Result<Vec<u8>, String> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "gramgraph", "--", dsl])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    // Write CSV to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(csv_content.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Check if bytes are a valid PNG
fn is_valid_png(bytes: &[u8]) -> bool {
    bytes.len() > 8 && &bytes[0..8] == &[137, 80, 78, 71, 13, 10, 26, 10]
}

#[test]
fn test_end_to_end_line_chart() {
    let csv = fs::read_to_string("test/timeseries.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: date, y: temperature) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes), "Output is not a valid PNG");
}

#[test]
fn test_end_to_end_scatter_plot() {
    let csv = fs::read_to_string("test/scatter.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: height, y: weight) | point()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_line_plus_points() {
    let csv = fs::read_to_string("test/timeseries.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: date, y: temperature) | line(color: \"blue\") | point(size: 5)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_bar_chart() {
    let csv = fs::read_to_string("test/bar_chart.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: category, y: value1) | bar()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_dodge_bars() {
    let csv = fs::read_to_string("test/sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: region, y: q1) | bar(position: \"dodge\", color: \"blue\") | bar(y: q2, position: \"dodge\", color: \"green\")",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_stack_bars() {
    let csv = fs::read_to_string("test/bar_chart.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: category, y: value1) | bar(position: \"stack\", color: \"blue\") | bar(y: value2, position: \"stack\", color: \"green\")",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_invalid_syntax() {
    let csv = "x,y\n1,10\n2,20\n";
    let result = run_gramgraph("invalid syntax here", csv);
    assert!(result.is_err(), "Should have failed with parse error");
    assert!(result.unwrap_err().contains("Parse error"));
}

#[test]
fn test_end_to_end_column_not_found() {
    let csv = "a,b\n1,10\n2,20\n";
    let result = run_gramgraph("aes(x: x, y: y) | line()", csv);
    assert!(result.is_err(), "Should have failed with column not found");
}

#[test]
fn test_end_to_end_empty_csv() {
    let csv = "x,y\n";
    let result = run_gramgraph("aes(x: x, y: y) | line()", csv);
    assert!(result.is_err(), "Should have failed with empty CSV error");
    assert!(result.unwrap_err().contains("at least one data row"));
}

#[test]
fn test_end_to_end_non_numeric_data() {
    let csv = fs::read_to_string("test/mixed_types.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: x, y: y) | line()", &csv);
    assert!(result.is_err(), "Should have failed with non-numeric data");
}

#[test]
fn test_end_to_end_large_dataset() {
    let csv = fs::read_to_string("test/many_rows.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: x, y: y) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_negative_values() {
    let csv = fs::read_to_string("test/negative_values.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: x, y: y) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_unicode() {
    let csv = fs::read_to_string("test/unicode.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: x, y: température) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_mixing_bar_and_line() {
    let csv = "x,y\n1,10\n2,20\n3,30\n";
    let result = run_gramgraph("aes(x: x, y: y) | bar() | line()", csv);
    assert!(result.is_ok(), "Mixing bar and line should now succeed");
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_styled_layers() {
    let csv = fs::read_to_string("test/timeseries.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: date, y: temperature) | line(color: \"red\", width: 2, alpha: 0.7)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

// Data-driven aesthetics tests

#[test]
fn test_end_to_end_grouped_line_by_color() {
    let csv = fs::read_to_string("test/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales, color: region) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
    // Should have legend, so PNG should be larger than ungrouped version
    assert!(png_bytes.len() > 10000, "PNG should include legend");
}

#[test]
fn test_end_to_end_grouped_scatter_by_color() {
    let csv = fs::read_to_string("test/iris.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: sepal_length, y: sepal_width, color: species) | point()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_grouped_with_size() {
    let csv = fs::read_to_string("test/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales, size: region) | point()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

// Faceting tests

#[test]
fn test_end_to_end_facet_wrap_basic() {
    let csv = fs::read_to_string("test/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales) | line() | facet_wrap(by: region)", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
    // Faceted plot should be significantly larger
    assert!(png_bytes.len() > 100000, "Faceted PNG should be larger");
}

#[test]
fn test_end_to_end_facet_wrap_scatter() {
    let csv = fs::read_to_string("test/iris.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: sepal_length, y: sepal_width) | point() | facet_wrap(by: species)", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_facet_with_ncol() {
    let csv = fs::read_to_string("test/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales) | line() | facet_wrap(by: region, ncol: Some(2))", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

// Combined features tests

#[test]
fn test_end_to_end_facet_plus_grouping() {
    let csv = fs::read_to_string("test/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales, color: product) | line() | facet_wrap(by: region)", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_multiple_layers_grouped() {
    let csv = fs::read_to_string("test/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales, color: region) | line() | point()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

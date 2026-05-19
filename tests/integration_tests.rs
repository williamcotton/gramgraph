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

/// Helper function to run gramgraph and request SVG output.
fn run_gramgraph_svg(dsl: &str, csv_content: &str) -> Result<String, String> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "gramgraph", "--", dsl, "--format", "svg"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(csv_content.as_bytes())
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 SVG: {}", e))
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
    let csv = fs::read_to_string("fixtures/timeseries.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: date, y: temperature) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes), "Output is not a valid PNG");
}

#[test]
fn test_end_to_end_scatter_plot() {
    let csv = fs::read_to_string("fixtures/scatter.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: height, y: weight) | point()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_line_plus_points() {
    let csv = fs::read_to_string("fixtures/timeseries.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: date, y: temperature) | line(color: \"blue\") | point(size: 5)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_datetime_scale_formats_svg_ticks() {
    let csv = "\
time,temp
2026-05-18T00:00,13.8
2026-05-18T20:00,18.4
2026-05-19T16:00,23.1
";
    let result = run_gramgraph_svg(
        r#"aes(x: time, y: temp) | line() | point() | scale_x_datetime(interval: "20h", format: "%b %-d %H:%M")"#,
        csv,
    );

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(
        svg.contains("May 18 00:00"),
        "SVG did not contain formatted datetime tick: {}",
        svg
    );
    assert!(
        !svg.contains("2026-05-18T00:00"),
        "SVG still contained the raw ISO timestamp"
    );
}

#[test]
fn test_end_to_end_log10_scale_formats_original_values() {
    let csv = "\
x,y
1,1
10,2
100,3
1000,4
";
    let result = run_gramgraph_svg(
        r#"aes(x: x, y: y) | point(shape: "triangle", size: 7) | scale_x_log10()"#,
        csv,
    );

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(
        svg.contains("\n1000\n"),
        "SVG did not contain original-value log tick labels: {}",
        svg
    );
}

#[test]
fn test_end_to_end_sqrt_scale() {
    let csv = "\
x,y
0,1
25,3
100,5
";
    let result = run_gramgraph_svg(
        r#"aes(x: x, y: y) | point(shape: "diamond", size: 7) | scale_x_sqrt()"#,
        csv,
    );

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(
        svg.contains("\n100\n"),
        "SVG did not contain sqrt-scale tick labels: {}",
        svg
    );
}

#[test]
fn test_end_to_end_area_step_and_reference_lines() {
    let csv = "\
x,y
1,2
2,4
3,3
4,6
";
    let result = run_gramgraph_svg(
        r#"aes(x: x, y: y) | area(color: "steelblue", alpha: 0.25) | step(direction: "mid", color: "steelblue", width: 2) | hline(yintercept: 3, color: "gray", width: 1, label: "Target") | vline(xintercept: 2.5, color: "red", alpha: 0.5, label: "Marker") | theme_minimal()"#,
        csv,
    );

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(
        svg.contains("polygon"),
        "area should render as a polygon: {}",
        svg
    );
    assert!(
        svg.contains("#FF0000"),
        "vline should use the requested red color: {}",
        svg
    );
    assert!(
        svg.contains("Target") && svg.contains("Marker"),
        "labeled reference lines should create legend entries: {}",
        svg
    );
    assert!(
        !svg.contains("default"),
        "reference lines should not expose their synthetic group key: {}",
        svg
    );
}

#[test]
fn test_end_to_end_reference_line_without_aes() {
    let csv = "x,y\n1,1\n";
    let result = run_gramgraph_svg(r#"hline(yintercept: 0, color: "gray")"#, csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(
        !svg.contains("default"),
        "unlabeled reference lines should not create default legend entries: {}",
        svg
    );
}

#[test]
fn test_end_to_end_theme_void() {
    let csv = "x,y\n1,1\n2,4\n3,9\n";
    let result = run_gramgraph_svg(r#"aes(x: x, y: y) | point() | theme_void()"#, csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_end_to_end_bar_chart() {
    let csv = fs::read_to_string("fixtures/bar_chart.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: category, y: value1) | bar()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_dodge_bars() {
    let csv = fs::read_to_string("fixtures/sales.csv").expect("Failed to read test CSV");
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
    let csv = fs::read_to_string("fixtures/bar_chart.csv").expect("Failed to read test CSV");
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
    // Unified renderer is more flexible: it treats non-numeric x-data as categorical
    // This allows line charts with categorical x-axis (like ggplot2)
    let csv = fs::read_to_string("fixtures/mixed_types.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: x, y: y) | line()", &csv);
    assert!(
        result.is_ok(),
        "Unified renderer handles mixed types by using categorical scale"
    );
}

#[test]
fn test_end_to_end_large_dataset() {
    let csv = fs::read_to_string("fixtures/many_rows.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: x, y: y) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_negative_values() {
    let csv = fs::read_to_string("fixtures/negative_values.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: x, y: y) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_unicode() {
    let csv = fs::read_to_string("fixtures/unicode.csv").expect("Failed to read test CSV");
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
    let csv = fs::read_to_string("fixtures/timeseries.csv").expect("Failed to read test CSV");
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
    let csv =
        fs::read_to_string("fixtures/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales, color: region) | line()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
    // Should have legend, so PNG should be larger than ungrouped version
    assert!(png_bytes.len() > 10000, "PNG should include legend");
}

#[test]
fn test_end_to_end_grouped_scatter_by_color() {
    let csv = fs::read_to_string("fixtures/iris.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: sepal_length, y: sepal_width, color: species) | point()",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_grouped_with_size() {
    let csv =
        fs::read_to_string("fixtures/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: time, y: sales, size: region) | point()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_grouped_with_shape_and_alpha() {
    let csv = fs::read_to_string("fixtures/iris.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: sepal_length, y: sepal_width, shape: species, alpha: species) | point(size: 7)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

// Faceting tests

#[test]
fn test_end_to_end_facet_wrap_basic() {
    let csv =
        fs::read_to_string("fixtures/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: time, y: sales) | line() | facet_wrap(by: region)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
    // Faceted plot should be significantly larger
    assert!(png_bytes.len() > 100000, "Faceted PNG should be larger");
}

#[test]
fn test_end_to_end_facet_wrap_scatter() {
    let csv = fs::read_to_string("fixtures/iris.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: sepal_length, y: sepal_width) | point() | facet_wrap(by: species)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_facet_with_ncol() {
    let csv =
        fs::read_to_string("fixtures/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: time, y: sales) | line() | facet_wrap(by: region, ncol: 2)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

// Combined features tests

#[test]
fn test_end_to_end_facet_plus_grouping() {
    let csv =
        fs::read_to_string("fixtures/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: time, y: sales, color: product) | line() | facet_wrap(by: region)",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_multiple_layers_grouped() {
    let csv =
        fs::read_to_string("fixtures/multiregion_sales.csv").expect("Failed to read test CSV");
    let result = run_gramgraph(
        "aes(x: time, y: sales, color: region) | line() | point()",
        &csv,
    );
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

#[test]
fn test_end_to_end_boxplot() {
    let csv = fs::read_to_string("fixtures/iris.csv").expect("Failed to read test CSV");
    let result = run_gramgraph("aes(x: species, y: sepal_length) | boxplot()", &csv);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    let png_bytes = result.unwrap();
    assert!(is_valid_png(&png_bytes));
}

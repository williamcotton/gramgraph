# GramGraph - Grammar of Graphics DSL for Rust

A command-line tool for generating data visualizations from CSV data using a Grammar of Graphics DSL inspired by ggplot2.

## Overview

GramGraph implements a **Grammar of Graphics** approach to data visualization, separating concerns between:
- **Aesthetics**: Mappings from data columns to visual properties
- **Geometries**: Visual representations (line, point, bar, etc.)
- **Layers**: Independent, composable visualization layers

This architecture enables powerful, declarative chart specifications with clean composition semantics.

## Architecture

```
CSV Data (stdin) → Parser → PlotSpec → Runtime → Canvas → PNG (stdout)
```

### Core Principles

1. **Separation of Aesthetics and Geometries**
   - Aesthetics define WHAT data maps to visual properties
   - Geometries define HOW that data is rendered

2. **Layer Composition**
   - Each geometry creates an independent layer
   - Layers are rendered in sequence, composing naturally
   - Multiple layers share coordinate space and ranges

3. **Aesthetic Inheritance**
   - Global aesthetics defined once with `aes()`
   - Individual layers inherit global aesthetics
   - Layers can override aesthetics locally

## DSL Syntax

### Basic Structure

```
aes(x: column, y: column) | geom() | geom() | ...
```

### Examples

**Simple line chart:**
```bash
cat data.csv | gramgraph 'aes(x: time, y: temperature) | line()'
```

**Styled line:**
```bash
cat data.csv | gramgraph 'aes(x: time, y: temp) | line(color: "red", width: 2)'
```

**Multiple layers (line + points):**
```bash
cat data.csv | gramgraph 'aes(x: time, y: temp) | line(color: "blue") | point(size: 5)'
```

**Per-layer aesthetic override:**
```bash
cat data.csv | gramgraph 'aes(x: time, y: temp) | line(y: high, color: "red") | line(y: low, color: "blue")'
```

**Bar chart:**
```bash
cat data.csv | gramgraph 'aes(x: category, y: value) | bar()'
```

**Side-by-side (dodged) bars:**
```bash
cat data.csv | gramgraph 'aes(x: region, y: q1) | bar(position: "dodge", color: "blue") | bar(y: q2, position: "dodge", color: "green")'
```

**Stacked bars:**
```bash
cat data.csv | gramgraph 'aes(x: month, y: product_a) | bar(position: "stack", color: "blue") | bar(y: product_b, position: "stack", color: "orange")'
```

### Supported Commands

#### `aes(x: col, y: col, ...)`
Defines global aesthetic mappings from data columns to visual properties.

**Required parameters:**
- `x:` - Column name for x-axis
- `y:` - Column name for y-axis

**Optional parameters (data-driven aesthetics):**
- `color: column` - Map column values to colors (creates grouped visualization with legend)
- `size: column` - Map column values to sizes
- `shape: column` - Map column values to shapes
- `alpha: column` - Map column values to transparency

#### `line(...)`
Renders data as a line series.

Optional parameters:
- `x: column` - Override x aesthetic for this layer
- `y: column` - Override y aesthetic for this layer
- `color: "red"` - Line color (red, green, blue, black, yellow, cyan, magenta)
- `width: 2` - Line width in pixels
- `alpha: 0.5` - Transparency (0.0-1.0)

#### `point(...)`
Renders data as points/scatter plot.

Optional parameters:
- `x: column` - Override x aesthetic
- `y: column` - Override y aesthetic
- `color: "blue"` - Point color
- `size: 5` - Point size in pixels
- `shape: "circle"` - Point shape (future)
- `alpha: 0.8` - Transparency

#### `bar(...)`
Renders data as a bar chart (categorical x-axis).

Optional parameters:
- `x: column` - Override x aesthetic for this layer
- `y: column` - Override y aesthetic for this layer
- `color: "red"` - Bar color (red, green, blue, black, yellow, cyan, magenta)
- `alpha: 0.7` - Transparency (0.0-1.0)
- `width: 0.8` - Bar width as fraction of category space (0.0-1.0)
- `position: "dodge"` - Positioning mode:
  - `"identity"` - Bars overlap at same position (default)
  - `"dodge"` - Bars side-by-side
  - `"stack"` - Bars stacked vertically

**Note**: Bar charts use categorical x-axis and cannot be mixed with line/point charts in the same plot.

## Module Structure

```
src/
├── main.rs              # CLI entry point
├── csv_reader.rs        # CSV parsing from stdin
├── graph.rs             # Canvas & rendering (Plotters backend)
├── runtime.rs           # Execute PlotSpec → PNG
└── parser/              # Grammar of Graphics parser
    ├── mod.rs           # Public API exports
    ├── ast.rs           # AST types (PlotSpec, Aesthetics, Layer, etc.)
    ├── lexer.rs         # Token parsing (identifier, string, number)
    ├── aesthetics.rs    # Parse aes(x: col, y: col)
    ├── geom.rs          # Parse line() and point() geometries
    └── pipeline.rs      # Parse complete plot specifications
```

## Parser Architecture

### AST Structure

```rust
// Complete plot specification
pub struct PlotSpec {
    pub aesthetics: Option<Aesthetics>,  // Global aes()
    pub layers: Vec<Layer>,              // Geometries
    pub labels: Option<Labels>,          // Title, axis labels
}

// Aesthetic mappings
pub struct Aesthetics {
    pub x: String,  // Column name
    pub y: String,  // Column name
}

// Individual layers
pub enum Layer {
    Line(LineLayer),
    Point(PointLayer),
}

pub struct LineLayer {
    // Aesthetic overrides
    pub x: Option<String>,
    pub y: Option<String>,

    // Visual properties
    pub color: Option<String>,
    pub width: Option<f64>,
    pub alpha: Option<f64>,
}

pub struct PointLayer {
    pub x: Option<String>,
    pub y: Option<String>,

    pub color: Option<String>,
    pub size: Option<f64>,
    pub shape: Option<String>,
    pub alpha: Option<f64>,
}
```

### Parsing Flow

1. **Lexer** (`lexer.rs`): Tokenize input
   - Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
   - String literals: `"..."`
   - Numbers: floats/integers
   - Operators: `|`, `:`, `,`, `(`, `)`

2. **Aesthetics Parser** (`aesthetics.rs`): Parse `aes(x: col, y: col)`
   - Extracts global aesthetic mappings
   - Returns `Aesthetics` struct

3. **Geometry Parser** (`geom.rs`): Parse `line()`, `point()`, etc.
   - Parses function name
   - Parses optional named arguments
   - Builds `Layer` enum variants

4. **Pipeline Parser** (`pipeline.rs`): Combine into `PlotSpec`
   - Parse optional `aes()` (global aesthetics)
   - Parse geometries separated by `|`
   - Build complete `PlotSpec`

## Runtime Architecture

### Layer Rendering

```rust
pub fn render_plot(spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>>
```

1. **Aesthetic Resolution**
   - For each layer, resolve x/y columns
   - Layer-specific aesthetics override global
   - Validate: must have x and y for each layer

2. **Data Extraction**
   - Extract columns from CSV data
   - Convert to `Vec<f64>` for plotting
   - Accumulate all data for range calculation

3. **Canvas Creation**
   - Calculate global x/y ranges from all layers
   - Add 5% padding for visual breathing room
   - Create Canvas with shared coordinate space

4. **Layer Composition**
   - Render each layer in sequence
   - Each layer draws on shared canvas
   - Layers compose naturally (line + points, etc.)

5. **PNG Encoding**
   - Finalize drawing area
   - Encode RGB buffer as PNG
   - Return PNG bytes

### Canvas API (`graph.rs`)

```rust
pub struct Canvas {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    x_range: Range<f64>,
    y_range: Range<f64>,
    title: Option<String>,
    chart_initialized: bool,
}

impl Canvas {
    pub fn new(width, height, title, all_x_data, all_y_data) -> Result<Self>
    pub fn add_line_layer(&mut self, x_data, y_data, style) -> Result<()>
    pub fn add_point_layer(&mut self, x_data, y_data, style) -> Result<()>
    pub fn render(self) -> Result<Vec<u8>>
}
```

**Key Design:**
- Canvas owns the pixel buffer
- Calculates global ranges from all data upfront
- Each `add_*_layer()` draws on the shared buffer
- Multiple layers share the same coordinate system

## Dependencies

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }  # CLI argument parsing
csv = "1.3"                                         # CSV reading
plotters = "0.3"                                    # Plotting backend
image = "0.24"                                      # PNG encoding
anyhow = "1.0"                                      # Error handling
nom = "7.1"                                         # Parser combinators
```

## Usage Examples

### Line Chart
```bash
cat data.csv | cargo run -- 'aes(x: date, y: value) | line()'
```

### Styled Line Chart
```bash
cat data.csv | cargo run -- 'aes(x: date, y: value) | line(color: "red", width: 2)'
```

### Scatter Plot
```bash
cat data.csv | cargo run -- 'aes(x: height, y: weight) | point(size: 3)'
```

### Line + Points (Layer Composition)
```bash
cat data.csv | cargo run -- 'aes(x: date, y: value) | line(color: "blue") | point(size: 5, color: "red")'
```

### Multiple Lines (Different Y Columns)
```bash
cat data.csv | cargo run -- 'aes(x: date, y: high) | line(color: "red") | line(y: low, color: "blue")'
```

### Bar Chart
```bash
cat data.csv | cargo run -- 'aes(x: category, y: value) | bar()'
```

### Side-by-Side (Dodge) Bars
```bash
cat data.csv | cargo run -- 'aes(x: region, y: q1) | bar(position: "dodge", color: "blue") | bar(y: q2, position: "dodge", color: "green")'
```

### Stacked Bars
```bash
cat data.csv | cargo run -- 'aes(x: month, y: product_a) | bar(position: "stack", color: "blue") | bar(y: product_b, position: "stack", color: "orange")'
```

## Design Decisions

### Why Grammar of Graphics?

The Grammar of Graphics approach provides:

1. **Composability**: Layers stack naturally (`line() | point()`)
2. **Reusability**: Define aesthetics once, use in multiple layers
3. **Extensibility**: Easy to add new geometries, scales, facets
4. **Declarative**: Describe WHAT you want, not HOW to draw it
5. **Intuitive**: Mirrors successful tools like ggplot2

### Why Separate Aesthetics from Geometries?

**Problem with coupled approach:**
```
chart(x: time, y: temp) | layer_line(color: "red")
```
- Chart command conflates data mapping with initialization
- Aesthetics are not reusable across layers
- Doesn't scale to complex multi-layer plots

**Grammar of Graphics solution:**
```
aes(x: time, y: temp) | line(color: "red") | point(size: 5)
```
- Clear separation: `aes()` maps data, `line()`/`point()` render
- Aesthetics defined once, inherited by all layers
- Each layer can override as needed
- Natural composition of multiple geometries

### Layer Rendering Strategy

**Challenge**: Multiple layers need to share coordinate space.

**Solution**: Two-pass approach
1. **Pass 1 (Data Collection)**:
   - Resolve aesthetics for each layer
   - Extract all data
   - Calculate global x/y ranges

2. **Pass 2 (Rendering)**:
   - Create canvas with global ranges
   - Render each layer in sequence
   - All layers share coordinate system

This ensures layers align correctly and don't clip each other.

## Future Extensions

The Grammar of Graphics architecture naturally supports:

### 1. Data-Driven Aesthetics
```
aes(x: time, y: temp, color: region) | line()
# Different colored lines per region (grouping)
```

### 2. Faceting (Small Multiples)
```
aes(x: time, y: temp) | line() | facet_wrap(by: region)
# Grid of subplots, one per region
```

### 3. Scales & Transformations
```
aes(x: time, y: temp) | line() | scale_y_log10()
# Logarithmic y-axis
```

### 4. Statistical Transformations
```
aes(x: category) | bar(stat: "count")
# Bar chart showing counts of each category
```

### 5. More Geometries
- `area()` - Filled area plots
- `ribbon()` - Confidence intervals
- `histogram()` - Frequency distributions
- `boxplot()` - Box-and-whisker plots
- `violin()` - Violin plots
- `heatmap()` - 2D density/heatmaps

### 6. Labels & Themes
```
aes(x: time, y: temp) | line() | labs(title: "Temperature", x: "Date", y: "Temp (°F)")
```

### 7. Coordinate Systems
```
aes(x: category, y: value) | bar() | coord_flip()
# Horizontal bar chart
```

## Testing

### Test Coverage Requirements

**GramGraph maintains 100% test coverage** across all modules. This ensures:
- Reliable functionality for all features
- Early detection of regressions
- Confidence in error handling
- Safe refactoring

### Running Tests

Run all tests:
```bash
cargo test
```

Run unit tests only:
```bash
cargo test --lib
```

Run integration tests only:
```bash
cargo test --test '*'
```

Run parser tests:
```bash
cargo test --lib parser
```

### Generating Coverage Reports

Install cargo-llvm-cov:
```bash
cargo install cargo-llvm-cov
```

Generate HTML coverage report:
```bash
cargo llvm-cov --html
open target/llvm-cov/html/index.html
```

Generate terminal summary:
```bash
cargo llvm-cov
```

Coverage should be 100% across all modules.

### Test Data Files

The `test/` directory contains CSV files for various testing scenarios:

#### Basic Test Files
- **test/basic.csv** - Simple 3x3 numeric data for basic functionality
- **test/timeseries.csv** - Time series data with multiple numeric columns
- **test/scatter.csv** - X-Y scatter plot data
- **test/bar_chart.csv** - Categorical data with multiple value columns
- **test/sales.csv** - Multi-region sales data for dodge/stack testing

#### Edge Case Test Files
- **test/empty.csv** - Empty file (headers only, no data rows)
- **test/single_row.csv** - Single data row
- **test/single_column.csv** - Single column of data
- **test/large_values.csv** - Very large numeric values (1e10)
- **test/small_values.csv** - Very small numeric values (1e-10)
- **test/negative_values.csv** - Negative numeric values
- **test/mixed_types.csv** - Mix of numeric and text (for error testing)
- **test/duplicate_headers.csv** - Duplicate column names
- **test/missing_values.csv** - Empty cells in data
- **test/special_chars.csv** - Special characters in column names
- **test/unicode.csv** - Unicode characters in data
- **test/long_column_names.csv** - Very long column names
- **test/many_rows.csv** - Large dataset (10,000+ rows)

#### Creating Test CSV Files

When adding new tests:
1. Create CSV files with descriptive names in `test/` directory
2. Include header row with column names
3. Add at least 3-5 data rows for meaningful tests
4. Document the purpose in test comments

Example test CSV structure:
```csv
x_column,y_column,category
1.0,10.0,A
2.0,20.0,B
3.0,30.0,C
```

### Test Organization

Tests are organized as:
- **Unit tests**: Inline `#[cfg(test)]` modules in each source file
- **Integration tests**: `tests/` directory for end-to-end workflows
- **Test fixtures**: `test/` directory for CSV data files

### Manual Testing Examples

Line and point charts:
```bash
cat test/timeseries.csv | cargo run -- 'aes(x: date, y: temperature) | line()'
cat test/timeseries.csv | cargo run -- 'aes(x: date, y: temperature) | line(color: "red") | point(size: 5)'
cat test/scatter.csv | cargo run -- 'aes(x: height, y: weight) | point()'
```

Bar charts:
```bash
cat test/bar_chart.csv | cargo run -- 'aes(x: category, y: value1) | bar()'
cat test/bar_chart.csv | cargo run -- 'aes(x: category, y: value1) | bar(color: "red")'
```

Side-by-side (dodge) bars:
```bash
cat test/bar_chart.csv | cargo run -- 'aes(x: category, y: value1) | bar(position: "dodge", color: "blue") | bar(y: value2, position: "dodge", color: "red")'
cat test/sales.csv | cargo run -- 'aes(x: region, y: q1) | bar(position: "dodge", color: "blue") | bar(y: q2, position: "dodge", color: "green")'
```

Stacked bars:
```bash
cat test/bar_chart.csv | cargo run -- 'aes(x: category, y: value1) | bar(position: "stack", color: "blue") | bar(y: value2, position: "stack", color: "green") | bar(y: value3, position: "stack", color: "red")'
```

Overlapping bars (identity):
```bash
cat test/bar_chart.csv | cargo run -- 'aes(x: category, y: value1) | bar(alpha: 0.5, color: "blue") | bar(y: value2, alpha: 0.5, color: "red")'
```

## Implementation Notes

### Parser Choice: nom

**Why nom?**
- Parser combinator library for Rust
- Type-safe, zero-copy parsing
- Composable parsers (match architecture)
- Excellent error messages with `context()`
- No separate lexer needed

### Rendering Backend: Plotters

**Why Plotters?**
- Pure Rust plotting library
- Multiple backends (bitmap, SVG, HTML canvas)
- Clean API for programmatic chart construction
- Supports complex multi-layer compositions
- Good performance for static chart generation

### CSV Parsing

Uses the `csv` crate for robust CSV handling:
- Automatic header detection
- Column selection by name or index
- Type conversion to `f64` for numeric plotting
- Clear error messages for invalid data

## Contributing

When adding new features:

1. **New Geometry Types**: Add to `ast.rs` Layer enum, implement parser in `geom.rs`, add rendering in `runtime.rs` and `graph.rs`

2. **New Aesthetics**: Extend `Aesthetics` struct, update `aesthetics.rs` parser, handle in runtime resolution

3. **Statistical Transformations**: Add transformation stage between data extraction and rendering

4. **Scales**: Implement scale transformations in Canvas coordinate mapping

## License

[Add your license here]

## Credits

Inspired by:
- **ggplot2** (Hadley Wickham) - Grammar of Graphics for R
- **The Grammar of Graphics** (Leland Wilkinson) - Theoretical foundation
- **Plotters** - Rust plotting library
- **nom** - Parser combinator library

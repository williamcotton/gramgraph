# GramGraph - Grammar of Graphics DSL for Rust

A command-line tool for generating data visualizations from CSV data using a Grammar of Graphics DSL inspired by ggplot2.

## Overview

GramGraph implements a **Grammar of Graphics** approach to data visualization, separating concerns between:
- **Aesthetics**: Mappings from data columns to visual properties
- **Geometries**: Visual representations (line, point, bar, etc.)
- **Layers**: Independent, composable visualization layers

This architecture enables powerful, declarative chart specifications with clean composition semantics.

## Features

### ✅ Implemented

- **Core Geometries**: `line()`, `point()`, `bar()` with full styling options
- **Data-Driven Aesthetics**: Automatic grouping by color, size, shape, or alpha with legends
- **Faceting**: Multi-panel subplot grids with `facet_wrap()` and flexible axis scales
- **Layer Composition**: Multiple geometries on shared coordinate space
- **Bar Charts**: Categorical x-axis with dodge, stack, and identity positioning
- **Automatic Legends**: Generated for grouped visualizations
- **Color Palettes**: Category10 scheme with 10 distinct colors
- **Flexible Parsing**: Order-independent named arguments in DSL

### 🚀 Coming Soon

- Scale transformations (log, sqrt, etc.)
- Statistical transformations (count, bin, smooth, etc.)
- Additional geometries (area, ribbon, histogram, boxplot, violin, heatmap)
- Custom labels and themes
- Coordinate system transformations

## Architecture

GramGraph employs a strict **Grammar of Graphics** pipeline, moving data through five distinct phases:

```
CSV Data → Resolution → Transformation → Scaling → Compilation → Rendering → PNG
```

### Core Principles

1. **Separation of Aesthetics and Geometries**
   - Aesthetics define WHAT data maps to visual properties
   - Geometries define HOW that data is rendered

2. **Unidirectional Data Flow**
   - Data is transformed, scaled, and compiled in strict sequence.
   - Rendering is "dumb" and only executes drawing commands.

3. **Layer Composition**
   - Multiple layers share the same coordinate space (Scales).
   - Layers are processed independently but rendered onto a shared canvas.

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

#### `facet_wrap(by: column, ...)`
Creates a grid of subplots (small multiples), one for each unique value in the specified column.

**Required parameters:**
- `by: column` - Column name to facet by (creates one subplot per unique value)

**Optional parameters:**
- `ncol: Some(n)` - Number of columns in the grid layout (auto-calculated if omitted)
- `scales: "mode"` - Axis scale sharing mode:
  - `"fixed"` - All facets share the same x and y ranges (default)
  - `"free_x"` - Independent x ranges, shared y range
  - `"free_y"` - Shared x range, independent y ranges
  - `"free"` - Independent x and y ranges for each facet

## Module Structure

```
src/
├── main.rs              # CLI entry point (wires library modules)
├── lib.rs               # Library export
├── csv_reader.rs        # CSV parsing from stdin
├── ir.rs                # Intermediate Representation (Data Contracts)
├── resolve.rs           # Phase 1: Aesthetic Resolution
├── transform.rs         # Phase 2: Data Transformation (Stats/Position)
├── scale.rs             # Phase 3: Scale Calculation
├── compiler.rs          # Phase 4: Compile to SceneGraph
├── graph.rs             # Phase 5: Rendering Backend (Plotters)
├── palette.rs           # Color/size/shape palettes
├── runtime.rs           # Pipeline Coordinator
└── parser/              # Grammar of Graphics parser
    ├── mod.rs           # Public API exports
    ├── ast.rs           # AST types
    ├── lexer.rs         # Token parsing
    ├── aesthetics.rs    # Parse aes()
    ├── geom.rs          # Parse geom()
    ├── facet.rs         # Parse facet_wrap()
    └── pipeline.rs      # Parse full pipeline
```

## Runtime Architecture (The Pipeline)

GramGraph executes a strict, linear pipeline implemented in `src/runtime.rs`:

### Phase 1: Resolution (`resolve.rs`)
- **Input:** `PlotSpec`, `CsvData`
- **Output:** `ResolvedSpec`
- **Logic:**
  - Validates that requested columns exist in CSV.
  - Merges global `aes()` with layer-specific overrides.
  - Resolves facet configuration.

### Phase 2: Transformation (`transform.rs`)
- **Input:** `ResolvedSpec`, `CsvData`
- **Output:** `RenderData`
- **Logic:**
  - **Partitioning:** Splits data into panels if faceting is enabled.
  - **Grouping:** Splits data within panels by `color`/`size`/etc. columns.
  - **Position Adjustment:** Calculates stacking offsets (`y_start`, `y_end`) for stacked bars.
  - **Normalization:** Converts categorical data to indices, parses numeric data.

### Phase 3: Scaling (`scale.rs`)
- **Input:** `RenderData`
- **Output:** `ScaleSystem`
- **Logic:**
  - measures the extent (min/max) of the transformed data.
  - Handles `fixed`, `free`, `free_x`, `free_y` scaling rules for facets.
  - Produces `Scale` objects (Continuous or Categorical).

### Phase 4: Compilation (`compiler.rs`)
- **Input:** `RenderData`, `ScaleSystem`
- **Output:** `SceneGraph`
- **Logic:**
  - Translates data coordinates into drawing commands (`DrawLine`, `DrawRect`, etc.).
  - Handles **Dodge** positioning for bar charts (visual X-offsets).
  - Assigns visual styles (colors, widths) from the transformation phase.

### Phase 5: Rendering (`graph.rs`)
- **Input:** `SceneGraph`
- **Output:** `Vec<u8>` (PNG)
- **Logic:**
  - Pure backend execution using the `plotters` library.
  - Sets up the grid layout.
  - Draws axes, grids, and primitives.
  - Encodes the result to PNG.

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

See previous sections or `tests/integration_tests.rs`.

## Design Decisions

### Why Grammar of Graphics?

The Grammar of Graphics approach provides:

1. **Composability**: Layers stack naturally (`line() | point()`)
2. **Reusability**: Define aesthetics once, use in multiple layers
3. **Extensibility**: Easy to add new geometries, scales, facets
4. **Declarative**: Describe WHAT you want, not HOW to draw it

### Layer Rendering Strategy

**Challenge**: Multiple layers need to share coordinate space.

**Solution**:
The **Scale** phase measures *all* layers across *all* panels before any drawing happens. This ensures that the coordinate system is globally consistent (or consistently independent per facet) regardless of the order of layers.

## Contributing

When adding new features:

1. **New Geometry Types**: 
   - Add to `ast.rs` `Layer` enum.
   - Implement parser in `geom.rs`.
   - Update `transform.rs` to extract data for it.
   - Update `compiler.rs` to generate drawing commands.

2. **New Aesthetics**: 
   - Extend `Aesthetics` struct.
   - Update `resolve.rs` to handle resolution.
   - Update `transform.rs` to map data to the aesthetic.

3. **Statistical Transformations**: 
   - Add transformation logic in `transform.rs` (e.g., binning for histograms).

4. **Scales**: 
   - Implement new scale types in `scale.rs`.

## License

[Add your license here]
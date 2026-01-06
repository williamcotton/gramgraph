// Abstract Syntax Tree for Grammar of Graphics DSL

/// Complete plot specification
#[derive(Debug, Clone, PartialEq)]
pub struct PlotSpec {
    pub aesthetics: Option<Aesthetics>,
    pub layers: Vec<Layer>,
    pub labels: Option<Labels>,
    pub facet: Option<Facet>,
}

/// Global aesthetic mappings (data columns → visual properties)
#[derive(Debug, Clone, PartialEq)]
pub struct Aesthetics {
    /// Column name for x-axis
    pub x: String,
    /// Column name for y-axis
    pub y: String,
    /// Optional column name for color grouping
    pub color: Option<String>,
    /// Optional column name for size grouping
    pub size: Option<String>,
    /// Optional column name for shape grouping
    pub shape: Option<String>,
    /// Optional column name for alpha grouping
    pub alpha: Option<String>,
}

/// Represents either a fixed literal value or a data-driven column mapping
#[derive(Debug, Clone, PartialEq)]
pub enum AestheticValue<T> {
    /// Fixed literal value (e.g., line(color: "red"))
    Fixed(T),
    /// Column name for data-driven mapping (e.g., aes(color: region))
    Mapped(String),
}

/// Individual visualization layer
#[derive(Debug, Clone, PartialEq)]
pub enum Layer {
    Line(LineLayer),
    Point(PointLayer),
    Bar(BarLayer),
    // Future: Area, Ribbon, Histogram, etc.
}

/// Line geometry layer
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LineLayer {
    // Aesthetic overrides (None = inherit from global)
    pub x: Option<String>,
    pub y: Option<String>,

    // Visual properties (can be fixed or data-driven)
    pub color: Option<AestheticValue<String>>,
    pub width: Option<AestheticValue<f64>>,
    pub alpha: Option<AestheticValue<f64>>,
    // Future: linetype (solid, dashed, dotted)
}

/// Point geometry layer
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PointLayer {
    // Aesthetic overrides
    pub x: Option<String>,
    pub y: Option<String>,

    // Visual properties (can be fixed or data-driven)
    pub color: Option<AestheticValue<String>>,
    pub size: Option<AestheticValue<f64>>,
    pub shape: Option<AestheticValue<String>>,
    pub alpha: Option<AestheticValue<f64>>,
}

/// Bar geometry layer
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BarLayer {
    // Aesthetic overrides
    pub x: Option<String>,
    pub y: Option<String>,

    // Visual properties (can be fixed or data-driven)
    pub color: Option<AestheticValue<String>>,
    pub alpha: Option<AestheticValue<f64>>,
    pub width: Option<AestheticValue<f64>>, // Bar width (0.0-1.0, relative to category spacing)

    // Positioning strategy
    pub position: BarPosition,
}

/// Bar positioning modes (how bars are arranged)
#[derive(Debug, Clone, PartialEq)]
pub enum BarPosition {
    Identity, // Bars overlap at same x position
    Dodge,    // Bars side-by-side
    Stack,    // Bars stacked vertically
}

impl Default for BarPosition {
    fn default() -> Self {
        BarPosition::Identity
    }
}

/// Plot labels (title, axes)
#[derive(Debug, Clone, PartialEq)]
pub struct Labels {
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
}

/// Facet specification for creating subplot grids
#[derive(Debug, Clone, PartialEq)]
pub struct Facet {
    /// Column name to facet by (creates one subplot per unique value)
    pub by: String,
    /// Number of columns in the grid layout (auto-calculated if None)
    pub ncol: Option<usize>,
    /// Axis scale sharing mode
    pub scales: FacetScales,
}

/// Facet axis scale sharing modes
#[derive(Debug, Clone, PartialEq)]
pub enum FacetScales {
    /// All facets share the same x and y ranges (default)
    Fixed,
    /// Independent x ranges, shared y range
    FreeX,
    /// Shared x range, independent y ranges
    FreeY,
    /// Independent x and y ranges for each facet
    Free,
}

impl Default for FacetScales {
    fn default() -> Self {
        FacetScales::Fixed
    }
}

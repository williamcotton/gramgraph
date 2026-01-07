use crate::parser::ast::Layer;
use crate::graph::{LineStyle, PointStyle, BarStyle};

// =============================================================================
// Phase 1: Resolution
// =============================================================================

/// Result of resolving aesthetics against the CSV headers (but not data values yet)
#[derive(Debug, Clone)]
pub struct ResolvedSpec {
    pub layers: Vec<ResolvedLayer>,
    pub facet: Option<ResolvedFacet>,
}

#[derive(Debug, Clone)]
pub struct ResolvedLayer {
    pub original_layer: Layer,
    pub aesthetics: ResolvedAesthetics,
}

#[derive(Debug, Clone)]
pub struct ResolvedAesthetics {
    pub x_col: String,
    pub y_col: String,
    // Optional grouping columns
    pub color: Option<String>,
    pub size: Option<String>,
    pub shape: Option<String>,
    pub alpha: Option<String>,
    // Fixed values (if not mapped) can be stored here or retrieved from Layer
}

#[derive(Debug, Clone)]
pub struct ResolvedFacet {
    pub col: String,
    pub ncol: Option<usize>,
    pub scales: crate::parser::ast::FacetScales,
}

// =============================================================================
// Phase 2: Transformation
// =============================================================================

/// The normalized data ready for scaling and rendering.
/// It is split into "Panels" (for faceting). If no faceting, there is 1 panel.
#[derive(Debug, Clone)]
pub struct RenderData {
    pub panels: Vec<PanelData>,
    pub facet_layout: FacetLayout,
}

#[derive(Debug, Clone)]
pub struct FacetLayout {
    pub nrow: usize,
    pub ncol: usize,
    pub panel_titles: Vec<String>, // Index matches panels
}

/// Data for a single plot panel (one facet)
#[derive(Debug, Clone)]
pub struct PanelData {
    pub index: usize,
    pub layers: Vec<LayerData>, // Corresponds 1:1 with ResolvedSpec.layers
}

/// Data for a single layer within a panel.
/// Contains one or more "Groups" (e.g. different colored lines).
#[derive(Debug, Clone)]
pub struct LayerData {
    pub groups: Vec<GroupData>,
}

/// The atomic unit of rendering: a set of points sharing the same visual style.
#[derive(Debug, Clone)]
pub struct GroupData {
    pub key: String, // Legend key (e.g. "Region A")
    
    // Normalized Geometry Data
    // For Line/Point: x and y are straightforward.
    // For Bar: x is category index, y is value.
    // Stacking is pre-calculated here: y_start, y_end.
    pub x: Vec<f64>,
    pub y: Vec<f64>,      // Main value (or y_end)
    pub y_start: Vec<f64>, // For stacked bars (0.0 if not stacked)
    
    // Original category names for x-axis (if categorical)
    pub x_categories: Option<Vec<String>>, 
    
    // Resolved Visual Style for this group
    pub style: RenderStyle,
}

#[derive(Debug, Clone)]
pub enum RenderStyle {
    Line(LineStyle),
    Point(PointStyle),
    Bar(BarStyle),
}

// =============================================================================
// Phase 3: Scaling
// =============================================================================

/// Holds the scales for the entire plot (potentially multiple panels)
#[derive(Debug, Clone)]
pub struct ScaleSystem {
    // One scale pair per panel
    pub panels: Vec<PanelScales>,
}

#[derive(Debug, Clone)]
pub struct PanelScales {
    pub x: Scale,
    pub y: Scale,
}

#[derive(Debug, Clone)]
pub struct Scale {
    pub domain: (f64, f64), // Data min/max
    pub range: (f64, f64),  // Pixel/Coordinate min/max
    pub is_categorical: bool,
    pub categories: Vec<String>, // If categorical, maps index -> label
}

// =============================================================================
// Phase 4: Compilation (Scene Graph)
// =============================================================================

/// A list of primitive drawing commands.
/// The Backend just executes these blindly.
#[derive(Debug, Clone)]
pub struct SceneGraph {
    pub width: u32,
    pub height: u32,
    pub panels: Vec<PanelScene>,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PanelScene {
    pub row: usize,
    pub col: usize,
    pub title: Option<String>,
    pub x_scale: Scale, // For drawing axes
    pub y_scale: Scale,
    pub commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    DrawLine {
        points: Vec<(f64, f64)>,
        style: LineStyle,
        legend: Option<String>,
    },
    DrawPoint {
        points: Vec<(f64, f64)>,
        style: PointStyle,
        legend: Option<String>,
    },
    DrawRect {
        // Top-Left, Bottom-Right
        tl: (f64, f64),
        br: (f64, f64),
        style: BarStyle,
        legend: Option<String>,
    },
}

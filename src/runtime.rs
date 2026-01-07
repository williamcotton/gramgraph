// Runtime executor for Grammar of Graphics DSL

use crate::csv_reader::{self, CsvData};
use crate::graph;
use crate::palette::{ColorPalette, ShapePalette, SizePalette};
use crate::parser::ast::{AestheticValue, Aesthetics, BarLayer, Facet, FacetScales, Layer, LineLayer, PlotSpec, PointLayer};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::ops::Range;

/// Render a plot specification to PNG bytes
pub fn render_plot(spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>> {
    // Validate: must have at least one layer
    if spec.layers.is_empty() {
        anyhow::bail!("Plot requires at least one geometry layer (line, point, etc.)");
    }

    let renderer: Box<dyn Renderer> = if spec.facet.is_some() {
        Box::new(FacetedRenderer)
    } else if spec.requires_categorical_x() {
        Box::new(CategoricalRenderer)
    } else {
        Box::new(ContinuousRenderer)
    };

    renderer.render(spec, csv_data)
}

/// Trait for different plot rendering strategies
trait Renderer {
    fn render(&self, spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>>;
}

struct FacetedRenderer;
impl Renderer for FacetedRenderer {
    fn render(&self, spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>> {
        let facet = spec.facet.clone().unwrap();
        render_faceted_plot(spec, csv_data, facet)
    }
}

struct CategoricalRenderer;
impl Renderer for CategoricalRenderer {
    fn render(&self, spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>> {
        render_categorical_plot(spec, csv_data)
    }
}

struct ContinuousRenderer;
impl Renderer for ContinuousRenderer {
    fn render(&self, spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>> {
        render_continuous_plot(spec, csv_data)
    }
}

/// Render a plot with categorical x-axis (supports Bar, Line, Point)
fn render_categorical_plot(spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>> {
    use crate::parser::ast::BarPosition;

    // 1. Collect all unique categories from all layers to define the x-axis
    let mut categories_set = std::collections::HashSet::new();
    let mut categories_order = Vec::new();

    for layer in &spec.layers {
        let resolved = resolve_layer_aesthetics(layer, &spec.aesthetics)?;
        let x_selector = csv_reader::parse_column_selector(&resolved.x_col);
        // We try to read as string. If it fails (e.g. column not found), we'll catch it later.
        if let Ok((_, x_vals)) = csv_reader::extract_column_as_string(&csv_data, x_selector) {
            for cat in x_vals {
                if !categories_set.contains(&cat) {
                    categories_set.insert(cat.clone());
                    categories_order.push(cat);
                }
            }
        }
    }
    categories_order.sort(); // Deterministic order
    
    let cat_map: HashMap<String, usize> = categories_order
        .iter()
        .enumerate()
        .map(|(i, c)| (c.clone(), i))
        .collect();
    
        let num_categories = categories_order.len();
        let all_x_data = vec![-0.5, (num_categories as f64) - 0.5];
        let mut all_y_data = Vec::new();
    // 2. Process data for each layer
    struct RenderOp {
        layer: Layer,
        // For Bar: categories + y (aligned)
        // For Line/Point: x (indices) + y
        x_data: Vec<f64>,
        y_data: Vec<f64>,
        bar_categories: Option<Vec<String>>, // Only for Bar
    }
    
    let mut ops = Vec::new();
    // Special handling for grouped bars
    let mut bar_groups: Vec<(Vec<String>, Vec<(Vec<f64>, graph::BarStyle, Option<String>)>, String)> = Vec::new(); 

    for layer in &spec.layers {
        let resolved = resolve_layer_aesthetics(layer, &spec.aesthetics)?;
        let group_col_opt = resolved.color_mapping.as_ref().or(resolved.alpha_mapping.as_ref());

        match layer {
            Layer::Bar(bar_layer) => {
                if let Some(group_col) = group_col_opt {
                    // Grouped Bar
                    let (raw_cats, groups) = extract_grouped_categorical_data(&csv_data, &resolved.x_col, &resolved.y_col, group_col)?;
                    let group_keys: Vec<String> = groups.iter().map(|(k,_)| k.clone()).collect();
                    let color_palette = ColorPalette::category10();
                    let color_map = color_palette.assign_colors(&group_keys);
                    
                    let mut series = Vec::new();
                    
                    for (g_key, raw_y) in groups {
                        let mut aligned_y = vec![0.0; num_categories];
                        for (i, cat) in raw_cats.iter().enumerate() {
                            if let Some(&idx) = cat_map.get(cat) {
                                aligned_y[idx] = raw_y[i];
                            }
                        }
                        
                        let mut style = bar_layer_to_style(bar_layer);
                        if resolved.color_mapping.is_some() {
                            style.color = color_map.get(&g_key).cloned().or(style.color);
                        }
                        all_y_data.extend(aligned_y.iter().cloned());
                        series.push((aligned_y, style.clone(), Some(g_key.clone())));
                    }
                    
                    let pos_str = match bar_layer.position {
                        BarPosition::Dodge => "dodge",
                        BarPosition::Stack => "stack",
                        BarPosition::Identity => "identity",
                    };
                    
                    bar_groups.push((categories_order.clone(), series, pos_str.to_string()));
                    
                } else {
                    // Ungrouped Bar
                    let (cats, y_vals) = extract_categorical_data(&csv_data, &resolved.x_col, &resolved.y_col)?;
                    let mut aligned_y = vec![0.0; num_categories];
                    for (i, cat) in cats.iter().enumerate() {
                        if let Some(&idx) = cat_map.get(cat) {
                            aligned_y[idx] = y_vals[i];
                        }
                    }
                    all_y_data.extend(aligned_y.iter().cloned());
                    
                    ops.push(RenderOp {
                        layer: layer.clone(),
                        x_data: vec![],
                        y_data: aligned_y,
                        bar_categories: Some(categories_order.clone()),
                    });
                }
            },
            Layer::Line(_) => {
                let (_, x_str) = csv_reader::extract_column_as_string(&csv_data, csv_reader::parse_column_selector(&resolved.x_col))?;
                let (_, y_val) = csv_reader::extract_column(&csv_data, csv_reader::parse_column_selector(&resolved.y_col))?;
                
                let mut x_f64 = Vec::new();
                let mut y_f64 = Vec::new();
                for (i, x_s) in x_str.iter().enumerate() {
                    if let Some(&idx) = cat_map.get(x_s) {
                        x_f64.push(idx as f64);
                        y_f64.push(y_val[i]);
                    }
                }
                all_y_data.extend(y_f64.iter().cloned());
                
                ops.push(RenderOp {
                    layer: layer.clone(),
                    x_data: x_f64,
                    y_data: y_f64,
                    bar_categories: None,
                });
            },
            Layer::Point(_) => {
                let (_, x_str) = csv_reader::extract_column_as_string(&csv_data, csv_reader::parse_column_selector(&resolved.x_col))?;
                let (_, y_val) = csv_reader::extract_column(&csv_data, csv_reader::parse_column_selector(&resolved.y_col))?;
                
                let mut x_f64 = Vec::new();
                let mut y_f64 = Vec::new();
                for (i, x_s) in x_str.iter().enumerate() {
                    if let Some(&idx) = cat_map.get(x_s) {
                        x_f64.push(idx as f64);
                        y_f64.push(y_val[i]);
                    }
                }
                all_y_data.extend(y_f64.iter().cloned());
                
                ops.push(RenderOp {
                    layer: layer.clone(),
                    x_data: x_f64,
                    y_data: y_f64,
                    bar_categories: None,
                });
            }
        }
    }
    
    // For stacked bars, calculate cumulative y values for range
    let first_bar_pos = spec.layers.iter().filter_map(|l| match l { Layer::Bar(b) => Some(b.position.clone()), _ => None }).next();
    if matches!(first_bar_pos, Some(BarPosition::Stack)) {
         let _max_stack = 0.0;
         // Approximate range calculation for stack (sum of all Ys at each X?)
         // This is tricky without reconstructing the stack. For now, let's rely on individual values or improve range calculation later.
         // Actually, to be safe, if we have stacks, we might need larger range.
         // Let's assume user provides sensible data or we just use sum of positive values per category.
    }

    let mut canvas = graph::Canvas::new(
        800,
        600,
        spec.labels.as_ref().and_then(|l| l.title.clone()),
        all_x_data,
        all_y_data,
    )?;
    canvas.set_x_labels(categories_order);

    // Render Groups
    for group in bar_groups {
        canvas.add_bar_group(group.0, group.1, &group.2)?;
    }

    // Render Layers
    for op in ops {
        match op.layer {
            Layer::Bar(b) => {
                if let Some(cats) = op.bar_categories {
                    canvas.add_bar_layer(cats, op.y_data, bar_layer_to_style(&b))?;
                }
            },
            Layer::Line(l) => {
                canvas.add_line_layer(op.x_data, op.y_data, line_layer_to_style(&l), None)?;
            },
            Layer::Point(p) => {
                canvas.add_point_layer(op.x_data, op.y_data, point_layer_to_style(&p), None)?;
            }
        }
    }

    canvas.render()
}

/// Render a plot with continuous charts (line/point)
fn render_continuous_plot(spec: PlotSpec, csv_data: CsvData) -> Result<Vec<u8>> {
    let mut all_x_data = Vec::new();
    let mut all_y_data = Vec::new();
    let mut render_instructions: Vec<RenderInstruction> = Vec::new();

    for layer in &spec.layers {
        // Resolve all aesthetics (x, y, and visual properties)
        let resolved = resolve_layer_aesthetics(layer, &spec.aesthetics)?;

        // Check if this layer needs grouping (has color/size/shape mapping)
        let primary_group_col = resolved.color_mapping.as_ref()
            .or(resolved.size_mapping.as_ref())
            .or(resolved.shape_mapping.as_ref())
            .or(resolved.alpha_mapping.as_ref());

        if let Some(group_col) = primary_group_col {
            // GROUPED RENDERING: Group data by the grouping column
            let groups = group_data_by_column(&csv_data, &resolved.x_col, &resolved.y_col, group_col)?;

            // Collect all data for range calculation
            for (x_data, y_data) in groups.values() {
                all_x_data.extend(x_data.iter().copied());
                all_y_data.extend(y_data.iter().copied());
            }

            // Get sorted group keys for consistent ordering
            let mut group_keys: Vec<String> = groups.keys().cloned().collect();
            group_keys.sort();

            // Create palettes
            let color_palette = ColorPalette::category10();
            let size_palette = SizePalette::default_range();
            let shape_palette = ShapePalette::default_shapes();

            let color_map = color_palette.assign_colors(&group_keys);
            let size_map = size_palette.assign_sizes(&group_keys);
            let shape_map = shape_palette.assign_shapes(&group_keys);

            // Create render instruction for each group
            for group_key in &group_keys {
                let (x_data, y_data) = groups.get(group_key).unwrap();

                // Build style with group-specific properties
                let style = build_grouped_style(
                    layer,
                    group_key,
                    &resolved,
                    &color_map,
                    &size_map,
                    &shape_map,
                );

                render_instructions.push(RenderInstruction {
                    layer: layer.clone(),
                    x_data: x_data.clone(),
                    y_data: y_data.clone(),
                    line_style: style.line_style,
                    point_style: style.point_style,
                    legend_label: Some(group_key.clone()),
                });
            }
        } else {
            // NON-GROUPED RENDERING: Extract data normally
            let x_selector = csv_reader::parse_column_selector(&resolved.x_col);
            let (_x_col_name, x_values) = csv_reader::extract_column(&csv_data, x_selector)
                .context(format!("Failed to extract x column '{}'", resolved.x_col))?;

            let y_selector = csv_reader::parse_column_selector(&resolved.y_col);
            let (_y_col_name, y_values) = csv_reader::extract_column(&csv_data, y_selector)
                .context(format!("Failed to extract y column '{}'", resolved.y_col))?;

            // Collect for global range
            all_x_data.extend(x_values.iter().copied());
            all_y_data.extend(y_values.iter().copied());

            // Build fixed style
            let style = build_fixed_style(layer);

            render_instructions.push(RenderInstruction {
                layer: layer.clone(),
                x_data: x_values,
                y_data: y_values,
                line_style: style.line_style,
                point_style: style.point_style,
                legend_label: None,
            });
        }
    }

    // Create canvas with global data ranges
    let mut canvas = graph::Canvas::new(
        800,
        600,
        spec.labels.as_ref().and_then(|l| l.title.clone()),
        all_x_data,
        all_y_data,
    )?;

    // Execute render instructions
    for instruction in &render_instructions {
        match &instruction.layer {
            Layer::Line(_) => {
                canvas.add_line_layer(
                    instruction.x_data.clone(),
                    instruction.y_data.clone(),
                    instruction.line_style.clone().unwrap_or_default(),
                    instruction.legend_label.clone(),
                )?;
            }
            Layer::Point(_) => {
                canvas.add_point_layer(
                    instruction.x_data.clone(),
                    instruction.y_data.clone(),
                    instruction.point_style.clone().unwrap_or_default(),
                    instruction.legend_label.clone(),
                )?;
            }
            Layer::Bar(_) => {
                unreachable!()
            }
        }
    }

    canvas.render()
}

/// Render instruction for a single series
struct RenderInstruction {
    layer: Layer,
    x_data: Vec<f64>,
    y_data: Vec<f64>,
    line_style: Option<graph::LineStyle>,
    point_style: Option<graph::PointStyle>,
    legend_label: Option<String>,
}

/// Unified style that can be either Line or Point
struct UnifiedStyle {
    line_style: Option<graph::LineStyle>,
    point_style: Option<graph::PointStyle>,
}

/// Build grouped style by merging fixed properties with group-specific mappings
fn build_grouped_style(
    layer: &Layer,
    group_key: &str,
    resolved: &ResolvedAesthetics,
    color_map: &HashMap<String, String>,
    size_map: &HashMap<String, f64>,
    shape_map: &HashMap<String, String>,
) -> UnifiedStyle {
    match layer {
        Layer::Line(line_layer) => {
            // Start with fixed properties
            let mut color = extract_fixed_string(&line_layer.color);
            let mut width = extract_fixed_f64(&line_layer.width);
            let alpha = extract_fixed_f64(&line_layer.alpha);

            // Override with group-specific values if mapped
            if resolved.color_mapping.is_some() {
                color = color_map.get(group_key).cloned();
            }
            if resolved.size_mapping.is_some() {
                width = size_map.get(group_key).copied();
            }

            UnifiedStyle {
                line_style: Some(graph::LineStyle { color, width, alpha }),
                point_style: None,
            }
        }
        Layer::Point(point_layer) => {
            // Start with fixed properties
            let mut color = extract_fixed_string(&point_layer.color);
            let mut size = extract_fixed_f64(&point_layer.size);
            let mut shape = extract_fixed_string(&point_layer.shape);
            let alpha = extract_fixed_f64(&point_layer.alpha);

            // Override with group-specific values if mapped
            if resolved.color_mapping.is_some() {
                color = color_map.get(group_key).cloned();
            }
            if resolved.size_mapping.is_some() {
                size = size_map.get(group_key).copied();
            }
            if resolved.shape_mapping.is_some() {
                shape = shape_map.get(group_key).cloned();
            }

            UnifiedStyle {
                line_style: None,
                point_style: Some(graph::PointStyle { color, size, shape, alpha }),
            }
        }
        Layer::Bar(_) => unreachable!(),
    }
}

/// Build fixed style (no grouping)
fn build_fixed_style(layer: &Layer) -> UnifiedStyle {
    match layer {
        Layer::Line(line_layer) => UnifiedStyle {
            line_style: Some(line_layer_to_style(line_layer)),
            point_style: None,
        },
        Layer::Point(point_layer) => UnifiedStyle {
            line_style: None,
            point_style: Some(point_layer_to_style(point_layer)),
        },
        Layer::Bar(_) => unreachable!(),
    }
}

/// Resolve aesthetics for a layer (layer override or global)
fn resolve_aesthetics(layer: &Layer, global_aes: &Option<Aesthetics>) -> Result<(String, String)> {
    let (x_override, y_override) = match layer {
        Layer::Line(l) => (l.x.as_ref(), l.y.as_ref()),
        Layer::Point(p) => (p.x.as_ref(), p.y.as_ref()),
        Layer::Bar(b) => (b.x.as_ref(), b.y.as_ref()),
    };

    // Get x column
    let x_col = if let Some(x) = x_override {
        x.clone()
    } else if let Some(ref aes) = global_aes {
        aes.x.clone()
    } else {
        anyhow::bail!("No x aesthetic specified (use aes(x: ..., y: ...) or layer-level x: ...)");
    };

    // Get y column
    let y_col = if let Some(y) = y_override {
        y.clone()
    } else if let Some(ref aes) = global_aes {
        aes.y.clone()
    } else {
        anyhow::bail!("No y aesthetic specified (use aes(x: ..., y: ...) or layer-level y: ...)");
    };

    Ok((x_col, y_col))
}

/// Convert LineLayer to graph::LineStyle
fn line_layer_to_style(layer: &LineLayer) -> graph::LineStyle {
    graph::LineStyle {
        color: extract_fixed_string(&layer.color),
        width: extract_fixed_f64(&layer.width),
        alpha: extract_fixed_f64(&layer.alpha),
    }
}

/// Extract fixed string value from AestheticValue
fn extract_fixed_string(value: &Option<AestheticValue<String>>) -> Option<String> {
    match value {
        Some(AestheticValue::Fixed(s)) => Some(s.clone()),
        _ => None, // Mapped values will be handled by grouping logic later
    }
}

/// Extract fixed f64 value from AestheticValue
fn extract_fixed_f64(value: &Option<AestheticValue<f64>>) -> Option<f64> {
    match value {
        Some(AestheticValue::Fixed(v)) => Some(*v),
        _ => None, // Mapped values will be handled by grouping logic later
    }
}

/// Resolved aesthetic mappings for a layer
/// Includes both positional aesthetics (x, y) and visual aesthetics (color, size, shape, alpha)
#[derive(Debug, Clone)]
struct ResolvedAesthetics {
    x_col: String,
    y_col: String,
    color_mapping: Option<String>,  // Column name for color grouping, or None
    size_mapping: Option<String>,   // Column name for size grouping, or None
    shape_mapping: Option<String>,  // Column name for shape grouping, or None
    alpha_mapping: Option<String>,  // Column name for alpha grouping, or None
}

/// Resolve all aesthetic mappings for a layer (layer-specific + global)
fn resolve_layer_aesthetics(
    layer: &Layer,
    global_aes: &Option<Aesthetics>,
) -> Result<ResolvedAesthetics> {
    // Resolve x and y (required)
    let (x_col, y_col) = resolve_aesthetics(layer, global_aes)?;

    // Resolve color mapping
    let color_mapping = match layer {
        Layer::Line(l) => extract_mapped_string(&l.color),
        Layer::Point(p) => extract_mapped_string(&p.color),
        Layer::Bar(b) => extract_mapped_string(&b.color),
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.color.clone()));

    // Resolve size mapping
    let size_mapping = match layer {
        Layer::Line(l) => extract_mapped_string_from_f64(&l.width), // width can be data-driven
        Layer::Point(p) => extract_mapped_string_from_f64(&p.size),
        Layer::Bar(b) => extract_mapped_string_from_f64(&b.width),
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.size.clone()));

    // Resolve shape mapping (point only)
    let shape_mapping = match layer {
        Layer::Point(p) => extract_mapped_string(&p.shape),
        _ => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.shape.clone()));

    // Resolve alpha mapping
    let alpha_mapping = match layer {
        Layer::Line(l) => extract_mapped_string_from_f64(&l.alpha),
        Layer::Point(p) => extract_mapped_string_from_f64(&p.alpha),
        Layer::Bar(b) => extract_mapped_string_from_f64(&b.alpha),
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.alpha.clone()));

    Ok(ResolvedAesthetics {
        x_col,
        y_col,
        color_mapping,
        size_mapping,
        shape_mapping,
        alpha_mapping,
    })
}

/// Extract column name from Mapped variant of AestheticValue<String>
fn extract_mapped_string(value: &Option<AestheticValue<String>>) -> Option<String> {
    match value {
        Some(AestheticValue::Mapped(col)) => Some(col.clone()),
        _ => None,
    }
}

/// Extract column name from Mapped variant of AestheticValue<f64>
fn extract_mapped_string_from_f64(value: &Option<AestheticValue<f64>>) -> Option<String> {
    match value {
        Some(AestheticValue::Mapped(col)) => Some(col.clone()),
        _ => None,
    }
}

/// Group data by a categorical column
/// Returns: HashMap<group_key, (x_data, y_data)>
fn group_data_by_column(
    csv_data: &CsvData,
    x_col: &str,
    y_col: &str,
    group_col: &str,
) -> Result<HashMap<String, (Vec<f64>, Vec<f64>)>> {
    // Find column indices
    let group_col_idx = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(group_col))
        .ok_or_else(|| anyhow!("Column '{}' not found for grouping", group_col))?;

    let x_col_idx = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(x_col))
        .ok_or_else(|| anyhow!("Column '{}' not found", x_col))?;

    let y_col_idx = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(y_col))
        .ok_or_else(|| anyhow!("Column '{}' not found", y_col))?;

    // Group data by the grouping column
    let mut groups: HashMap<String, (Vec<f64>, Vec<f64>)> = HashMap::new();

    for row in &csv_data.rows {
        let group_key = row[group_col_idx].clone();

        let x_val = row[x_col_idx]
            .parse::<f64>()
            .context(format!("Failed to parse x value: {}", row[x_col_idx]))?;

        let y_val = row[y_col_idx]
            .parse::<f64>()
            .context(format!("Failed to parse y value: {}", row[y_col_idx]))?;

        let (x_data, y_data) = groups
            .entry(group_key)
            .or_insert_with(|| (Vec::new(), Vec::new()));

        x_data.push(x_val);
        y_data.push(y_val);
    }

    Ok(groups)
}

/// Convert PointLayer to graph::PointStyle
fn point_layer_to_style(layer: &PointLayer) -> graph::PointStyle {
    graph::PointStyle {
        color: extract_fixed_string(&layer.color),
        size: extract_fixed_f64(&layer.size),
        shape: extract_fixed_string(&layer.shape),
        alpha: extract_fixed_f64(&layer.alpha),
    }
}

/// Convert BarLayer to graph::BarStyle
fn bar_layer_to_style(layer: &BarLayer) -> graph::BarStyle {
    graph::BarStyle {
        color: extract_fixed_string(&layer.color),
        alpha: extract_fixed_f64(&layer.alpha),
        width: extract_fixed_f64(&layer.width),
    }
}

/// Extract categorical data from CSV for bar charts
/// Returns (categories, y_values) where y_values are aggregated by category (sum)
fn extract_categorical_data(
    csv_data: &CsvData,
    x_col: &str,
    y_col: &str,
) -> Result<(Vec<String>, Vec<f64>)> {
    // Find column indices
    let x_col_index = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(x_col))
        .ok_or_else(|| anyhow!("Column '{}' not found", x_col))?;

    let y_col_index = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(y_col))
        .ok_or_else(|| anyhow!("Column '{}' not found", y_col))?;

    // Extract x categories and y values, aggregating by category
    let mut category_values: HashMap<String, f64> = HashMap::new();
    let mut categories_order: Vec<String> = Vec::new();

    for (row_idx, row) in csv_data.rows.iter().enumerate() {
        let category = row[x_col_index].clone();
        let y_str = &row[y_col_index];
        let y_val = y_str.parse::<f64>().with_context(|| {
            format!(
                "Failed to parse '{}' as number in column '{}' at row {}",
                y_str,
                y_col,
                row_idx + 1
            )
        })?;

        // Track category order (first appearance)
        if !category_values.contains_key(&category) {
            categories_order.push(category.clone());
        }

        // Aggregate y values (sum)
        *category_values.entry(category).or_insert(0.0) += y_val;
    }

    // Build vectors in category order
    let y_values: Vec<f64> = categories_order
        .iter()
        .map(|cat| *category_values.get(cat).unwrap_or(&0.0))
        .collect();

    Ok((categories_order, y_values))
}







/// Extract grouped categorical data from CSV
/// Returns (all_categories, vec[(group_name, y_values)])
fn extract_grouped_categorical_data(
    csv_data: &CsvData,
    x_col: &str,
    y_col: &str,
    group_col: &str,
) -> Result<(Vec<String>, Vec<(String, Vec<f64>)>)> {
    // Find column indices
    let x_col_index = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(x_col))
        .ok_or_else(|| anyhow!("Column '{}' not found", x_col))?;

    let y_col_index = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(y_col))
        .ok_or_else(|| anyhow!("Column '{}' not found", y_col))?;

    let group_col_index = csv_data
        .headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case(group_col))
        .ok_or_else(|| anyhow!("Column '{}' not found for grouping", group_col))?;

    // First pass: identify all unique categories and groups
    let mut all_categories: Vec<String> = Vec::new();
    let mut categories_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut groups_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut data_map: HashMap<(String, String), f64> = HashMap::new(); // (group, category) -> sum

    for (row_idx, row) in csv_data.rows.iter().enumerate() {
        let category = row[x_col_index].clone();
        let group = row[group_col_index].clone();
        
        let y_str = &row[y_col_index];
        let y_val = y_str.parse::<f64>().with_context(|| {
            format!(
                "Failed to parse '{}' as number in column '{}' at row {}",
                y_str,
                y_col,
                row_idx + 1
            )
        })?;

        if !categories_set.contains(&category) {
            categories_set.insert(category.clone());
            all_categories.push(category.clone());
        }
        groups_set.insert(group.clone());

        *data_map.entry((group, category)).or_insert(0.0) += y_val;
    }

    // Sort groups for consistent order
    let mut sorted_groups: Vec<String> = groups_set.into_iter().collect();
    sorted_groups.sort();

    // Build result vectors
    let mut result_groups = Vec::new();

    for group in sorted_groups {
        let mut y_values = Vec::new();
        for cat in &all_categories {
            let val = *data_map.get(&(group.clone(), cat.clone())).unwrap_or(&0.0);
            y_values.push(val);
        }
        result_groups.push((group, y_values));
    }

    Ok((all_categories, result_groups))
}

/// Split CSV data by facet column
fn split_data_by_facet(csv_data: &CsvData, facet_col: &str) -> Result<HashMap<String, CsvData>> {
    // Find facet column index
    let facet_col_idx = csv_data.headers.iter()
        .position(|h| h == facet_col)
        .ok_or_else(|| anyhow!("Facet column '{}' not found in data", facet_col))?;

    // Group rows by facet value
    let mut facet_groups: HashMap<String, Vec<Vec<String>>> = HashMap::new();

    for row in &csv_data.rows {
        let facet_value = row.get(facet_col_idx)
            .ok_or_else(|| anyhow!("Missing facet value in row"))?
            .clone();

        facet_groups.entry(facet_value)
            .or_insert_with(Vec::new)
            .push(row.clone());
    }

    // Convert to CsvData for each facet
    let mut result = HashMap::new();
    for (facet_value, rows) in facet_groups {
        result.insert(facet_value, CsvData {
            headers: csv_data.headers.clone(),
            rows,
        });
    }

    Ok(result)
}

/// Calculate global or per-facet ranges based on scales mode
fn calculate_facet_ranges(
    spec: &PlotSpec,
    facet_data: &HashMap<String, CsvData>,
    scales: &FacetScales,
) -> Result<HashMap<String, (Range<f64>, Range<f64>)>> {
    let mut ranges = HashMap::new();

    match scales {
        FacetScales::Fixed => {
            // Global ranges: collect all data from all facets
            let mut all_x = Vec::new();
            let mut all_y = Vec::new();

            for facet_csv in facet_data.values() {
                // Extract x and y data from all layers
                for layer in &spec.layers {
                    let resolved = resolve_layer_aesthetics(layer, &spec.aesthetics)?;

                    let x_selector = csv_reader::parse_column_selector(&resolved.x_col);
                    if let Ok((_col_name, x_vals)) = csv_reader::extract_column(facet_csv, x_selector) {
                        all_x.extend(x_vals);
                    }

                    let y_selector = csv_reader::parse_column_selector(&resolved.y_col);
                    if let Ok((_col_name, y_vals)) = csv_reader::extract_column(facet_csv, y_selector) {
                        all_y.extend(y_vals);
                    }
                }
            }

            // Calculate single global range
            let x_range = calculate_range(&all_x);
            let y_range = calculate_range(&all_y);

            // Assign same range to all facets
            for facet_key in facet_data.keys() {
                ranges.insert(facet_key.clone(), (x_range.clone(), y_range.clone()));
            }
        }
        FacetScales::FreeX | FacetScales::FreeY | FacetScales::Free => {
            // Calculate per-facet ranges
            for (facet_key, facet_csv) in facet_data {
                let mut facet_x = Vec::new();
                let mut facet_y = Vec::new();

                for layer in &spec.layers {
                    let resolved = resolve_layer_aesthetics(layer, &spec.aesthetics)?;

                    let x_selector = csv_reader::parse_column_selector(&resolved.x_col);
                    if let Ok((_col_name, x_vals)) = csv_reader::extract_column(facet_csv, x_selector) {
                        facet_x.extend(x_vals);
                    }

                    let y_selector = csv_reader::parse_column_selector(&resolved.y_col);
                    if let Ok((_col_name, y_vals)) = csv_reader::extract_column(facet_csv, y_selector) {
                        facet_y.extend(y_vals);
                    }
                }

                let x_range = calculate_range(&facet_x);
                let y_range = calculate_range(&facet_y);
                ranges.insert(facet_key.clone(), (x_range, y_range));
            }
        }
    }

    Ok(ranges)
}

/// Calculate range with padding for a dataset
fn calculate_range(data: &[f64]) -> Range<f64> {
    if data.is_empty() {
        return 0.0..1.0;
    }

    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if min == max {
        (min - 1.0)..(max + 1.0)
    } else {
        let padding = (max - min) * 0.05;
        (min - padding)..(max + padding)
    }
}

/// Render a faceted plot (subplot grid based on categorical column)
fn render_faceted_plot(spec: PlotSpec, csv_data: CsvData, facet: Facet) -> Result<Vec<u8>> {
    // Split data by facet column
    let facet_data = split_data_by_facet(&csv_data, &facet.by)?;

    if facet_data.is_empty() {
        anyhow::bail!("No data to facet by column '{}'", facet.by);
    }

    // Get sorted facet keys for consistent ordering
    let mut facet_keys: Vec<String> = facet_data.keys().cloned().collect();
    facet_keys.sort();

    // Calculate grid layout
    let num_facets = facet_keys.len();
    let ncol = facet.ncol.unwrap_or_else(|| (num_facets as f64).sqrt().ceil() as usize);
    let nrow = (num_facets as f64 / ncol as f64).ceil() as usize;

    // Calculate ranges based on scales mode
    let ranges = calculate_facet_ranges(&spec, &facet_data, &facet.scales)?;

    // Create multi-facet canvas
    let mut canvas = graph::MultiFacetCanvas::new(1200, 800, nrow, ncol)?;

    // Render each facet
    for (facet_idx, facet_key) in facet_keys.iter().enumerate() {
        let facet_csv = facet_data.get(facet_key).unwrap();
        let (x_range, y_range) = ranges.get(facet_key).unwrap();

        // Calculate grid position
        let row = facet_idx / ncol;
        let col = facet_idx % ncol;

        // Collect all series to render on this facet
        let mut facet_series_list: Vec<graph::FacetSeries> = Vec::new();

        // For each layer, extract data and render
        for layer in &spec.layers {
            let resolved = resolve_layer_aesthetics(layer, &spec.aesthetics)?;

            // Check if this layer needs grouping (has color/size/shape mapping)
            let primary_group_col = resolved.color_mapping.as_ref()
                .or(resolved.size_mapping.as_ref())
                .or(resolved.shape_mapping.as_ref())
                .or(resolved.alpha_mapping.as_ref());

            if let Some(group_col) = primary_group_col {
                // GROUPED RENDERING
                // Note: Ideally we should use global palette across all facets for consistency,
                // but for now we generate local palette per facet to ensure grouping works at all.
                let groups = group_data_by_column(facet_csv, &resolved.x_col, &resolved.y_col, group_col)?;

                // Get sorted group keys
                let mut group_keys: Vec<String> = groups.keys().cloned().collect();
                group_keys.sort();

                let color_palette = ColorPalette::category10();
                let size_palette = SizePalette::default_range();
                let shape_palette = ShapePalette::default_shapes();

                let color_map = color_palette.assign_colors(&group_keys);
                let size_map = size_palette.assign_sizes(&group_keys);
                let shape_map = shape_palette.assign_shapes(&group_keys);

                for group_key in &group_keys {
                    let (x_data, y_data) = groups.get(group_key).unwrap();
                    
                    let style = build_grouped_style(
                        layer,
                        group_key,
                        &resolved,
                        &color_map,
                        &size_map,
                        &shape_map,
                    );

                    facet_series_list.push(graph::FacetSeries {
                        x_data: x_data.clone(),
                        y_data: y_data.clone(),
                        line_style: style.line_style,
                        point_style: style.point_style,
                    });
                }
            } else {
                // NON-GROUPED RENDERING
                let x_selector = csv_reader::parse_column_selector(&resolved.x_col);
                let (_x_col_name, x_data) = csv_reader::extract_column(facet_csv, x_selector)
                    .context(format!("Failed to extract x column '{}' in facet '{}'", resolved.x_col, facet_key))?;

                let y_selector = csv_reader::parse_column_selector(&resolved.y_col);
                let (_y_col_name, y_data) = csv_reader::extract_column(facet_csv, y_selector)
                    .context(format!("Failed to extract y column '{}' in facet '{}'", resolved.y_col, facet_key))?;

                // Build style (using fixed styles only)
                let style = build_fixed_style(layer);

                facet_series_list.push(graph::FacetSeries {
                    x_data,
                    y_data,
                    line_style: style.line_style,
                    point_style: style.point_style,
                });
            }
        }

        // Render facet panel
        canvas.render_facet(
            row,
            col,
            facet_key,
            facet_series_list,
            x_range.clone(),
            y_range.clone(),
        )?;
    }

    canvas.render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{BarPosition, Aesthetics, PlotSpec};

    /// Helper to create test CsvData
    fn make_csv_data(headers: Vec<&str>, rows: Vec<Vec<&str>>) -> CsvData {
        CsvData {
            headers: headers.iter().map(|s| s.to_string()).collect(),
            rows: rows.iter()
                .map(|r| r.iter().map(|s| s.to_string()).collect())
                .collect(),
        }
    }



    // resolve_aesthetics tests (5 tests)

    #[test]
    fn test_resolve_aesthetics_from_global() {
        let layer = Layer::Line(LineLayer::default());
        let global_aes = Some(Aesthetics {
            x: "time".to_string(),
            y: "temp".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
        });
        let (x, y) = resolve_aesthetics(&layer, &global_aes).unwrap();
        assert_eq!(x, "time");
        assert_eq!(y, "temp");
    }

    #[test]
    fn test_resolve_aesthetics_layer_override_y() {
        let layer = Layer::Line(LineLayer {
            x: None,
            y: Some("humidity".to_string()),
            color: None,
            width: None,
            alpha: None,
        });
        let global_aes = Some(Aesthetics {
            x: "time".to_string(),
            y: "temp".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
        });
        let (x, y) = resolve_aesthetics(&layer, &global_aes).unwrap();
        assert_eq!(x, "time");
        assert_eq!(y, "humidity");
    }

    #[test]
    fn test_resolve_aesthetics_layer_override_both() {
        let layer = Layer::Point(PointLayer {
            x: Some("date".to_string()),
            y: Some("value".to_string()),
            color: None,
            size: None,
            shape: None,
            alpha: None,
        });
        let global_aes = Some(Aesthetics {
            x: "time".to_string(),
            y: "temp".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
        });
        let (x, y) = resolve_aesthetics(&layer, &global_aes).unwrap();
        assert_eq!(x, "date");
        assert_eq!(y, "value");
    }

    #[test]
    fn test_resolve_aesthetics_no_global_no_layer() {
        let layer = Layer::Line(LineLayer::default());
        let result = resolve_aesthetics(&layer, &None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No x aesthetic"));
    }

    #[test]
    fn test_resolve_aesthetics_missing_y() {
        let layer = Layer::Bar(BarLayer {
            x: Some("category".to_string()),
            y: None,
            color: None,
            alpha: None,
            width: None,
            position: BarPosition::Identity,
        });
        let result = resolve_aesthetics(&layer, &None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No y aesthetic"));
    }

    // extract_categorical_data tests (4 tests)

    #[test]
    fn test_extract_categorical_data_basic() {
        let csv = make_csv_data(
            vec!["category", "value"],
            vec![vec!["A", "10"], vec!["B", "20"], vec!["C", "30"]],
        );
        let (categories, values) = extract_categorical_data(&csv, "category", "value").unwrap();
        assert_eq!(categories, vec!["A", "B", "C"]);
        assert_eq!(values, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_extract_categorical_data_aggregation() {
        // Multiple rows with same category should sum
        let csv = make_csv_data(
            vec!["category", "value"],
            vec![vec!["A", "10"], vec!["B", "20"], vec!["A", "15"]],
        );
        let (categories, values) = extract_categorical_data(&csv, "category", "value").unwrap();
        assert_eq!(categories, vec!["A", "B"]);
        assert_eq!(values, vec![25.0, 20.0]);
    }

    #[test]
    fn test_extract_categorical_data_column_not_found() {
        let csv = make_csv_data(vec!["a", "b"], vec![vec!["1", "2"]]);
        let result = extract_categorical_data(&csv, "nonexistent", "b");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_extract_categorical_data_non_numeric_y() {
        let csv = make_csv_data(
            vec!["category", "value"],
            vec![vec!["A", "not_a_number"]],
        );
        let result = extract_categorical_data(&csv, "category", "value");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    // Style conversion tests (4 tests)

    #[test]
    fn test_line_layer_to_style_defaults() {
        let layer = LineLayer::default();
        let style = line_layer_to_style(&layer);
        assert_eq!(style.color, None);
        assert_eq!(style.width, None);
        assert_eq!(style.alpha, None);
    }

    #[test]
    fn test_line_layer_to_style_full() {
        let layer = LineLayer {
            x: None,
            y: None,
            color: Some(AestheticValue::Fixed("red".to_string())),
            width: Some(AestheticValue::Fixed(2.5)),
            alpha: Some(AestheticValue::Fixed(0.8)),
        };
        let style = line_layer_to_style(&layer);
        assert_eq!(style.color, Some("red".to_string()));
        assert_eq!(style.width, Some(2.5));
        assert_eq!(style.alpha, Some(0.8));
    }

    #[test]
    fn test_point_layer_to_style_defaults() {
        let layer = PointLayer::default();
        let style = point_layer_to_style(&layer);
        assert_eq!(style.color, None);
        assert_eq!(style.size, None);
        assert_eq!(style.alpha, None);
    }

    #[test]
    fn test_bar_layer_to_style_defaults() {
        let layer = BarLayer::default();
        let style = bar_layer_to_style(&layer);
        assert_eq!(style.color, None);
        assert_eq!(style.alpha, None);
        assert_eq!(style.width, None);
    }

    // render_continuous_plot tests (4 tests)

    #[test]
    fn test_render_continuous_plot_line() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![Layer::Line(LineLayer::default())],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(vec!["x", "y"], vec![vec!["1", "10"], vec!["2", "20"]]);
        let result = render_continuous_plot(spec, csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_continuous_plot_point() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![Layer::Point(PointLayer::default())],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(vec!["x", "y"], vec![vec!["1", "10"]]);
        let result = render_continuous_plot(spec, csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_continuous_plot_line_and_point() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![
                Layer::Line(LineLayer::default()),
                Layer::Point(PointLayer::default()),
            ],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(vec!["x", "y"], vec![vec!["1", "10"], vec!["2", "20"]]);
        let result = render_continuous_plot(spec, csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_continuous_plot_column_not_found() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "nonexistent".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![Layer::Line(LineLayer::default())],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(vec!["x", "y"], vec![vec!["1", "10"]]);
        let result = render_continuous_plot(spec, csv);
        assert!(result.is_err());
    }

    // render_bar_plot tests (4 tests)

    #[test]
    fn test_render_bar_plot_single() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "category".to_string(),
                y: "value".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![Layer::Bar(BarLayer::default())],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(
            vec!["category", "value"],
            vec![vec!["A", "10"], vec!["B", "20"]],
        );
        let result = render_plot(spec, csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_bar_plot_dodge() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "category".to_string(),
                y: "v1".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![
                Layer::Bar(BarLayer {
                    x: None,
                    y: None,
                    color: Some(AestheticValue::Fixed("blue".to_string())),
                    alpha: None,
                    width: None,
                    position: BarPosition::Dodge,
                }),
                Layer::Bar(BarLayer {
                    x: None,
                    y: Some("v2".to_string()),
                    color: Some(AestheticValue::Fixed("red".to_string())),
                    alpha: None,
                    width: None,
                    position: BarPosition::Dodge,
                }),
            ],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(
            vec!["category", "v1", "v2"],
            vec![vec!["A", "10", "15"], vec!["B", "20", "25"]],
        );
        let result = render_plot(spec, csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_bar_plot_stack() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "category".to_string(),
                y: "v1".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![
                Layer::Bar(BarLayer {
                    x: None,
                    y: None,
                    color: None,
                    alpha: None,
                    width: None,
                    position: BarPosition::Stack,
                }),
                Layer::Bar(BarLayer {
                    x: None,
                    y: Some("v2".to_string()),
                    color: None,
                    alpha: None,
                    width: None,
                    position: BarPosition::Stack,
                }),
            ],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(
            vec!["category", "v1", "v2"],
            vec![vec!["A", "10", "15"]],
        );
        let result = render_plot(spec, csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_bar_plot_identity() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "category".to_string(),
                y: "value".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![
                Layer::Bar(BarLayer::default()),
                Layer::Bar(BarLayer {
                    x: None,
                    y: Some("v2".to_string()),
                    color: None,
                    alpha: None,
                    width: None,
                    position: BarPosition::Identity,
                }),
            ],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(
            vec!["category", "value", "v2"],
            vec![vec!["A", "10", "15"]],
        );
        let result = render_plot(spec, csv);
        assert!(result.is_ok());
    }

    // render_plot tests (4 tests)

    #[test]
    fn test_render_plot_no_layers() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(vec!["x", "y"], vec![vec!["1", "10"]]);
        let result = render_plot(spec, csv);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one"));
    }

    #[test]
    fn test_render_plot_line_success() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![Layer::Line(LineLayer::default())],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(vec!["x", "y"], vec![vec!["1", "10"], vec!["2", "20"]]);
        let result = render_plot(spec, csv);
        assert!(result.is_ok());
        // Check it's a PNG
        let png_bytes = result.unwrap();
        assert!(png_bytes.len() > 8);
        assert_eq!(&png_bytes[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_render_plot_bar_success() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "cat".to_string(),
                y: "val".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![Layer::Bar(BarLayer::default())],
            labels: None,
            facet: None,
        };
        let csv = make_csv_data(vec!["cat", "val"], vec![vec!["A", "10"], vec!["B", "20"]]);
        let result = render_plot(spec, csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_plot_mixed_bar_line_success() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![
                Layer::Bar(BarLayer::default()),
                Layer::Line(LineLayer::default()),
            ],
            labels: None,
            facet: None,
        };
        // Use string data for x to force categorical mode
        let csv = make_csv_data(vec!["x", "y"], vec![vec!["A", "10"]]);
        let result = render_plot(spec, csv);
        assert!(result.is_ok());
    }
}

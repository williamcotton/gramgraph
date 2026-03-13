use anyhow::{anyhow, Context, Result};
use std::collections::{HashMap, HashSet};
use crate::data::PlotData;
use crate::ir::{RenderData, PanelData, LayerData, GroupData, FacetLayout, RenderStyle};
use crate::ir::{ResolvedSpec, ResolvedLayer, ResolvedAesthetics, ResolvedFacet};
use crate::parser::ast::{Layer, BarPosition, Stat};
use crate::graph::{LineStyle, PointStyle, BarStyle, RibbonStyle, ViolinStyle, DensityStyle};
use crate::palette::{ColorPalette, SizePalette, ShapePalette};

/// Main entry point: Transform resolved spec and CSV data into renderable data
pub fn apply_transformations(spec: &ResolvedSpec, data: &PlotData) -> Result<RenderData> {
    // 1. Partition Data (Faceting)
    let partitions = partition_data(spec, data)?;
    
    // 2. Calculate Layout info
    let (nrow, ncol) = calculate_grid_dimensions(partitions.len(), spec.facet.as_ref());
    let facet_layout = FacetLayout {
        nrow,
        ncol,
        panel_titles: partitions.iter().map(|p| p.title.clone()).collect(),
    };

    // 3. Process each partition into a Panel
    let mut panels = Vec::new();
    for (idx, partition) in partitions.into_iter().enumerate() {
        let panel = process_partition(idx, partition, spec)?;
        panels.push(panel);
    }

    Ok(RenderData {
        panels,
        facet_layout,
    })
}

struct DataPartition {
    title: String,
    data: PlotData,
}

/// Split CSV data based on facet configuration
fn partition_data(spec: &ResolvedSpec, data: &PlotData) -> Result<Vec<DataPartition>> {
    if let Some(facet) = &spec.facet {
        // Find facet column index
        let col_idx = data.headers.iter()
            .position(|h| h.eq_ignore_ascii_case(&facet.col))
            .ok_or_else(|| anyhow!("Facet column '{}' not found", facet.col))?;

        // Group rows
        let mut groups: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        for row in &data.rows {
            if let Some(val) = row.get(col_idx) {
                groups.entry(val.clone()).or_default().push(row.clone());
            }
        }

        // Sort keys
        let mut keys: Vec<String> = groups.keys().cloned().collect();
        keys.sort();

        let mut partitions = Vec::new();
        for key in keys {
            let rows = groups.remove(&key).unwrap();
            partitions.push(DataPartition {
                title: key,
                data: PlotData {
                    headers: data.headers.clone(),
                    rows,
                },
            });
        }
        Ok(partitions)
    } else {
        // No facet, single partition
        Ok(vec![DataPartition {
            title: "".to_string(),
            data: data.clone(), // Clone is expensive but safe for now
        }])
    }
}

fn calculate_grid_dimensions(n_panels: usize, facet: Option<&ResolvedFacet>) -> (usize, usize) {
    if let Some(f) = facet {
        if let Some(cols) = f.ncol {
            let rows = (n_panels as f64 / cols as f64).ceil() as usize;
            return (rows, cols);
        }
    }
    // Default: square-ish
    let cols = (n_panels as f64).sqrt().ceil() as usize;
    let rows = (n_panels as f64 / cols as f64).ceil() as usize;
    (rows, cols)
}

/// Process a single data partition (Panel)
fn process_partition(index: usize, partition: DataPartition, spec: &ResolvedSpec) -> Result<PanelData> {
    let mut layers = Vec::new();

    for layer_spec in &spec.layers {
        let layer_data = process_layer(layer_spec, &partition.data)?;
        layers.push(layer_data);
    }

    Ok(PanelData {
        index,
        layers,
    })
}

/// Process a single layer: Extract, Group, Stack
fn process_layer(layer_spec: &ResolvedLayer, data: &PlotData) -> Result<LayerData> {
    let aes = &layer_spec.aesthetics;
    
    // 1. Identify Grouping Column
    let group_col = aes.color.as_ref()
        .or(aes.size.as_ref())
        .or(aes.shape.as_ref())
        .or(aes.alpha.as_ref());

    // 2. Extract Data (Grouped)
    // We return a map: GroupKey -> (RawX, RawY, RawYMin, RawYMax)
    // RawX is String to handle both numeric and categorical initially
    let mut raw_groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)> = HashMap::new();
    
    // Column Indices
    let x_idx = find_col_index(&data.headers, &aes.x_col)?;
    let y_idx = if let Some(y) = &aes.y_col { Some(find_col_index(&data.headers, y)?) } else { None };
    let ymin_idx = if let Some(col) = &aes.ymin_col { Some(find_col_index(&data.headers, col)?) } else { None };
    let ymax_idx = if let Some(col) = &aes.ymax_col { Some(find_col_index(&data.headers, col)?) } else { None };

    let group_idx = if let Some(g) = group_col {
        Some(find_col_index(&data.headers, g)?)
    } else {
        None
    };

    for row in &data.rows {
        let x_str = row[x_idx].clone();
        let y_val = if let Some(idx) = y_idx { 
            row[idx].parse::<f64>().context(format!("Failed to parse Y value '{}'", row[idx]))?
        } else { 
            0.0 // Default for histogram if not provided
        };
        let ymin_val = if let Some(idx) = ymin_idx { row[idx].parse::<f64>().unwrap_or(0.0) } else { 0.0 };
        let ymax_val = if let Some(idx) = ymax_idx { row[idx].parse::<f64>().unwrap_or(0.0) } else { 0.0 };
        
        let group_key = if let Some(idx) = group_idx {
            row[idx].clone()
        } else {
            "default".to_string()
        };

        let entry = raw_groups.entry(group_key).or_insert_with(|| (Vec::new(), Vec::new(), Vec::new(), Vec::new()));
        entry.0.push(x_str);
        entry.1.push(y_val);
        entry.2.push(ymin_val);
        entry.3.push(ymax_val);
    }

    // Apply Statistics
    let raw_groups = apply_statistics(raw_groups, layer_spec.original_layer.stat())?;

    // 3. Determine X-Axis Type (Numeric vs Categorical)
    // Logic: If ALL x values in this layer can be parsed as float, it's numeric.
    // UNLESS it's a Bar chart, which forces categorical.
    let is_bar = matches!(layer_spec.original_layer, Layer::Bar(_));
    let is_boxplot = matches!(layer_spec.original_layer, Layer::Boxplot(_));
    let is_violin = matches!(layer_spec.original_layer, Layer::Violin(_));

    let all_x_strings: Vec<&String> = raw_groups.values().flat_map(|d| d.x.iter()).collect();
    let all_numeric = all_x_strings.iter().all(|s| s.parse::<f64>().is_ok());

    let use_categorical = is_bar || is_boxplot || is_violin || !all_numeric;

    // 4. Normalize X Values
    // If categorical, we need a unified mapping for stacking/grouping
    let mut x_category_map = HashMap::new();
    let mut category_order = Vec::new();
    
    if use_categorical {
        // Collect all unique categories to assign indices
        let mut unique_cats: HashSet<String> = HashSet::new();
        // Preserve order of appearance if possible, or sort? 
        // GoG usually sorts unless factor provided. Let's sort for determinism.
        for s in &all_x_strings {
            unique_cats.insert((*s).clone());
        }
        category_order = unique_cats.into_iter().collect();
        
        // Try to sort numerically if possible
        let all_numeric_cats = category_order.iter().all(|s| s.parse::<f64>().is_ok());
        if all_numeric_cats {
            category_order.sort_by(|a, b| {
                let fa = a.parse::<f64>().unwrap();
                let fb = b.parse::<f64>().unwrap();
                fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            category_order.sort();
        }
        
        for (i, cat) in category_order.iter().enumerate() {
            x_category_map.insert(cat.clone(), i as f64);
        }
    }

    // 5. Build Groups (Styles & Coordinates)
    let mut groups = Vec::new();
    let sorted_group_keys = get_sorted_keys(&raw_groups);
    
    // Assign Palettes
    let color_map = ColorPalette::category10().assign_colors(&sorted_group_keys);
    let size_map = SizePalette::default_range().assign_sizes(&sorted_group_keys);
    let shape_map = ShapePalette::default_shapes().assign_shapes(&sorted_group_keys);

    // Prepare for Stacking (if needed)
    let mut stack_offsets: HashMap<String, f64> = HashMap::new(); // Map "X_Key" -> Current Height
    let is_stacked = match &layer_spec.original_layer {
        Layer::Bar(b) => matches!(b.position, BarPosition::Stack),
        _ => false,
    };

    // Iterate groups in defined order (important for stacking order)
    for key in sorted_group_keys {
        let stat_data = raw_groups.get(&key).unwrap();
        let raw_x = &stat_data.x;
        let raw_y = &stat_data.y;
        let raw_ymin = &stat_data.ymin;
        let raw_ymax = &stat_data.ymax;
        
        let mut x_floats = Vec::with_capacity(raw_x.len());
        let mut y_starts = Vec::with_capacity(raw_x.len());
        let mut y_ends = Vec::with_capacity(raw_x.len());
        let mut y_mins = Vec::with_capacity(raw_x.len());
        let mut y_maxs = Vec::with_capacity(raw_x.len());
        
        // Boxplot specific
        let mut y_q1s = Vec::new();
        let mut y_medians = Vec::new();
        let mut y_q3s = Vec::new();
        let mut outliers_vec = Vec::new();

        // Violin specific
        let mut violin_density_vec: Vec<Vec<f64>> = Vec::new();
        let mut violin_density_y_vec: Vec<Vec<f64>> = Vec::new();
        let mut violin_quantile_values_vec: Vec<Vec<f64>> = Vec::new();

        for (i, x_s) in raw_x.iter().enumerate() {
            let y_val = raw_y[i];
            let raw_min = raw_ymin[i];
            let raw_max = raw_ymax[i];
            
            // Resolve X
            let x_val = if use_categorical {
                *x_category_map.get(x_s).unwrap() // Should exist
            } else {
                x_s.parse::<f64>().unwrap() // Verified numeric earlier
            };
            x_floats.push(x_val);

            // Resolve Y (Stacking and Min/Max)
            let stack_key = if use_categorical { x_s.clone() } else { x_val.to_string() };
            
            let (y_start, y_end, y_min, y_max) = if is_stacked {
                let start = *stack_offsets.get(&stack_key).unwrap_or(&0.0);
                let end = start + y_val;
                stack_offsets.insert(stack_key, end);
                (start, end, start, end)
            } else if matches!(layer_spec.original_layer, Layer::Ribbon(_)) || matches!(layer_spec.original_layer, Layer::Boxplot(_)) || matches!(layer_spec.original_layer, Layer::Violin(_)) {
                // Ribbon, Boxplot, and Violin use raw ymin/ymax
                (raw_min, raw_max, raw_min, raw_max)
            } else {
                // Line/Point/Bar(unstacked)
                (0.0, y_val, 0.0, y_val)
            };
            
            y_starts.push(y_start);
            y_ends.push(y_end);
            y_mins.push(y_min);
            y_maxs.push(y_max);
            
            // Collect boxplot stats if available
            if let Some(bp) = &stat_data.boxplot {
                y_q1s.push(bp.q1[i]);
                y_medians.push(bp.median[i]);
                y_q3s.push(bp.q3[i]);
                outliers_vec.push(bp.outliers[i].clone());
            } else {
                 // Fill defaults to keep vectors aligned
                 y_q1s.push(0.0);
                 y_medians.push(0.0);
                 y_q3s.push(0.0);
                 outliers_vec.push(vec![]);
            }

            // Collect violin stats if available
            if let Some(vp) = &stat_data.violin {
                violin_density_vec.push(vp.density[i].clone());
                violin_density_y_vec.push(vp.density_y[i].clone());
                violin_quantile_values_vec.push(vp.quantile_values[i].clone());
            } else {
                violin_density_vec.push(vec![]);
                violin_density_y_vec.push(vec![]);
                violin_quantile_values_vec.push(vec![]);
            }
        }

        // Build Style
        let style = build_style(key.clone(), &layer_spec.original_layer, aes, &color_map, &size_map, &shape_map);

        groups.push(GroupData {
            key: key.clone(),
            x: x_floats,
            y: y_ends, // Main value
            y_start: y_starts,
            y_min: y_mins,
            y_max: y_maxs,

            y_q1: y_q1s,
            y_median: y_medians,
            y_q3: y_q3s,
            outliers: outliers_vec,

            violin_density: violin_density_vec,
            violin_density_y: violin_density_y_vec,
            violin_quantile_values: violin_quantile_values_vec,

            x_categories: if use_categorical { Some(category_order.clone()) } else { None },
            style,
        });
    }

    Ok(LayerData { groups })
}
fn find_col_index(headers: &[String], name: &str) -> Result<usize> {
    headers.iter()
        .position(|h| h.eq_ignore_ascii_case(name))
        .ok_or_else(|| anyhow!("Column '{}' not found", name))
}

fn get_sorted_keys<V>(map: &HashMap<String, V>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort();
    keys
}

fn build_style(
    group_key: String,
    layer: &Layer,
    aes: &ResolvedAesthetics,
    color_map: &HashMap<String, String>,
    size_map: &HashMap<String, f64>,
    shape_map: &HashMap<String, String>,
) -> RenderStyle {
    // Helper to pick color: GroupMapped ?? Fixed ?? Default
    let pick_color = |l_color: &Option<crate::parser::ast::AestheticValue<String>>| -> Option<String> {
        if aes.color.is_some() && color_map.contains_key(&group_key) {
             color_map.get(&group_key).cloned()
        } else {
            // Check fixed
            match l_color {
                Some(crate::parser::ast::AestheticValue::Fixed(c)) => Some(c.clone()),
                _ => None,
            }
        }
    };
    
    // Helper to pick size/width
    let pick_size = |l_val: &Option<crate::parser::ast::AestheticValue<f64>>| -> Option<f64> {
         if aes.size.is_some() && size_map.contains_key(&group_key) {
             size_map.get(&group_key).copied()
         } else {
             match l_val {
                 Some(crate::parser::ast::AestheticValue::Fixed(v)) => Some(*v),
                 _ => None,
             }
         }
    };

    // Helper to pick alpha
    let pick_alpha = |l_val: &Option<crate::parser::ast::AestheticValue<f64>>| -> Option<f64> {
        // Alpha mapping not fully implemented in palettes yet, usually fixed
        // If we added alpha palette, we would check aes.alpha.is_some() here
         match l_val {
             Some(crate::parser::ast::AestheticValue::Fixed(v)) => Some(*v),
             _ => None,
         }
    };

    match layer {
        Layer::Line(l) => RenderStyle::Line(LineStyle {
            color: pick_color(&l.color),
            width: pick_size(&l.width),
            alpha: pick_alpha(&l.alpha),
        }),
        Layer::Point(p) => RenderStyle::Point(PointStyle {
            color: pick_color(&p.color),
            size: pick_size(&p.size),
            shape: if aes.shape.is_some() && shape_map.contains_key(&group_key) {
                shape_map.get(&group_key).cloned()
            } else {
                match &p.shape {
                    Some(crate::parser::ast::AestheticValue::Fixed(s)) => Some(s.clone()),
                    _ => None,
                }
            },
            alpha: pick_alpha(&p.alpha),
        }),
        Layer::Bar(b) => RenderStyle::Bar(BarStyle {
            color: pick_color(&b.color),
            width: pick_size(&b.width),
            alpha: pick_alpha(&b.alpha),
        }),
        Layer::Ribbon(r) => RenderStyle::Ribbon(RibbonStyle {
            color: pick_color(&r.color),
            alpha: pick_alpha(&r.alpha),
        }),
        Layer::Boxplot(b) => RenderStyle::Boxplot(crate::graph::BoxplotStyle {
            color: pick_color(&b.color),
            width: pick_size(&b.width),
            alpha: pick_alpha(&b.alpha),
            outlier_color: b.outlier_color.clone(),
            outlier_size: b.outlier_size,
            outlier_shape: b.outlier_shape.clone(),
        }),
        Layer::Violin(v) => RenderStyle::Violin(ViolinStyle {
            color: pick_color(&v.color),
            width: pick_size(&v.width),
            alpha: pick_alpha(&v.alpha),
            draw_quantiles: v.draw_quantiles.clone(),
        }),
        Layer::Density(d) => RenderStyle::Density(DensityStyle {
            color: pick_color(&d.color),
            alpha: pick_alpha(&d.alpha),
        }),
    }
}

#[derive(Debug, Clone)]
struct BoxplotData {
    q1: Vec<f64>,
    median: Vec<f64>,
    q3: Vec<f64>,
    outliers: Vec<Vec<f64>>,
}

#[derive(Debug, Clone)]
struct ViolinData {
    density: Vec<Vec<f64>>,           // Normalized density values (0-1) per x category
    density_y: Vec<Vec<f64>>,         // Y coordinates for density curve per x category
    quantile_values: Vec<Vec<f64>>,   // Computed Y values at requested quantiles per x category
}

#[derive(Debug, Clone)]
struct StatData {
    x: Vec<String>,
    y: Vec<f64>,
    ymin: Vec<f64>,
    ymax: Vec<f64>,
    boxplot: Option<BoxplotData>,
    violin: Option<ViolinData>,
}

impl StatData {
    fn from_tuple(t: (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)) -> Self {
        StatData {
            x: t.0,
            y: t.1,
            ymin: t.2,
            ymax: t.3,
            boxplot: None,
            violin: None,
        }
    }
}

fn compute_boxplot_stat(
    groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)>
) -> Result<HashMap<String, StatData>> {
    let mut new_groups = HashMap::new();

    for (key, (x_strs, y_vals, _, _)) in groups {
        // Group by X value
        let mut x_groups: HashMap<String, Vec<f64>> = HashMap::new();
        for (x, y) in x_strs.iter().zip(y_vals.iter()) {
            x_groups.entry(x.clone()).or_default().push(*y);
        }

        let mut unique_x: Vec<String> = x_groups.keys().cloned().collect();
        // Sort unique X? 
        // We rely on numeric parsing if possible, or string sort.
        // Let's replicate the sorting logic from process_layer loosely or just string sort.
        // process_layer handles final sorting. Here we just need consistent order.
        unique_x.sort();

        let mut res_x = Vec::new();
        let mut res_min = Vec::new();    // Lower whisker
        let mut res_max = Vec::new();    // Upper whisker
        let mut res_q1 = Vec::new();
        let mut res_median = Vec::new();
        let mut res_q3 = Vec::new();
        let mut res_outliers = Vec::new();

        for x_val in unique_x {
            let mut ys = x_groups[&x_val].clone();
            ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            if ys.is_empty() { continue; }

            let q1 = percentile(&ys, 0.25);
            let median = percentile(&ys, 0.50);
            let q3 = percentile(&ys, 0.75);
            let iqr = q3 - q1;

            let lower_fence = q1 - 1.5 * iqr;
            let upper_fence = q3 + 1.5 * iqr;

            // Whiskers: Range of data within fences
            let lower_whisker = ys.iter().filter(|&&v| v >= lower_fence).min_by(|a,b| a.partial_cmp(b).unwrap()).unwrap_or(&q1);
            let upper_whisker = ys.iter().filter(|&&v| v <= upper_fence).max_by(|a,b| a.partial_cmp(b).unwrap()).unwrap_or(&q3);

            // Outliers
            let outliers: Vec<f64> = ys.iter().filter(|&&v| v < lower_fence || v > upper_fence).cloned().collect();

            res_x.push(x_val);
            res_min.push(*lower_whisker);
            res_max.push(*upper_whisker);
            res_q1.push(q1);
            res_median.push(median);
            res_q3.push(q3);
            res_outliers.push(outliers);
        }

        // Y in StatData usually represents the "main" value. For boxplot, maybe median?
        // Or we just ignore Y and use the boxplot specific fields.
        // Let's use median for Y so stacking/etc (if applied) does something sane-ish, though boxplots aren't usually stacked.
        let res_y = res_median.clone();

        new_groups.insert(key, StatData {
            x: res_x,
            y: res_y,
            ymin: res_min,
            ymax: res_max,
            boxplot: Some(BoxplotData {
                q1: res_q1,
                median: res_median,
                q3: res_q3,
                outliers: res_outliers,
            }),
            violin: None,
        });
    }

    Ok(new_groups)
}

/// Silverman's rule of thumb for bandwidth selection
fn silverman_bandwidth(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    if n < 2.0 { return 1.0; }

    let mean = data.iter().sum::<f64>() / n;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    // IQR-based estimate for robustness
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q1 = percentile(&sorted, 0.25);
    let q3 = percentile(&sorted, 0.75);
    let iqr = q3 - q1;

    // Silverman's rule: h = 0.9 * min(std, IQR/1.34) * n^(-1/5)
    let scale = if iqr > 0.0 { std_dev.min(iqr / 1.34) } else { std_dev };
    if scale <= 0.0 { return 1.0; }
    0.9 * scale * n.powf(-0.2)
}

/// Gaussian kernel function
fn gaussian_kernel(u: f64) -> f64 {
    const SQRT_2PI: f64 = 2.5066282746310002;
    (-0.5 * u * u).exp() / SQRT_2PI
}

/// Compute Gaussian KDE at grid points
fn compute_kde(data: &[f64], bandwidth: f64) -> (Vec<f64>, Vec<f64>) {
    const GRID_POINTS: usize = 128;  // Resolution of density curve

    let n = data.len() as f64;
    if n == 0.0 { return (vec![], vec![]); }

    let min_y = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_y = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    // Extend range slightly for smooth edges
    let extend = 3.0 * bandwidth;
    let y_start = min_y - extend;
    let y_end = max_y + extend;

    let range = y_end - y_start;
    if range <= 0.0 { return (vec![min_y], vec![1.0]); }

    let step = range / (GRID_POINTS - 1) as f64;
    let mut grid_y = Vec::with_capacity(GRID_POINTS);
    let mut density = Vec::with_capacity(GRID_POINTS);

    for i in 0..GRID_POINTS {
        let y = y_start + i as f64 * step;
        grid_y.push(y);

        // Gaussian kernel density estimation
        let mut d = 0.0;
        for &xi in data {
            let u = (y - xi) / bandwidth;
            d += gaussian_kernel(u);
        }
        d /= n * bandwidth;
        density.push(d);
    }

    // Normalize density to 0-1 range for rendering
    let max_density = density.iter().fold(0.0f64, |a, &b| a.max(b));
    if max_density > 0.0 {
        for d in &mut density {
            *d /= max_density;
        }
    }

    (grid_y, density)
}

/// Compute violin statistics using KDE
fn compute_violin_stat(
    groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)>,
    draw_quantiles: &[f64],
) -> Result<HashMap<String, StatData>> {
    let mut new_groups = HashMap::new();

    for (key, (x_strs, y_vals, _, _)) in groups {
        // Group by X value (category)
        let mut x_groups: HashMap<String, Vec<f64>> = HashMap::new();
        for (x, y) in x_strs.iter().zip(y_vals.iter()) {
            x_groups.entry(x.clone()).or_default().push(*y);
        }

        let mut unique_x: Vec<String> = x_groups.keys().cloned().collect();
        unique_x.sort();

        let mut res_x = Vec::new();
        let mut res_y = Vec::new();     // Will hold median for y
        let mut res_min = Vec::new();
        let mut res_max = Vec::new();
        let mut density_vec = Vec::new();
        let mut density_y_vec = Vec::new();
        let mut quantile_values_vec = Vec::new();

        for x_val in unique_x {
            let ys = &x_groups[&x_val];
            if ys.is_empty() { continue; }

            let mut sorted_ys = ys.clone();
            sorted_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let min_y = sorted_ys[0];
            let max_y = sorted_ys[sorted_ys.len() - 1];
            let median = percentile(&sorted_ys, 0.5);

            // Compute bandwidth using Silverman's rule
            let bandwidth = silverman_bandwidth(&sorted_ys);

            // Compute KDE
            let (grid_y, density) = compute_kde(&sorted_ys, bandwidth);

            // Compute actual data percentiles for requested quantiles
            let quantile_y_values: Vec<f64> = draw_quantiles
                .iter()
                .map(|&q| percentile(&sorted_ys, q))
                .collect();

            res_x.push(x_val);
            res_y.push(median);
            res_min.push(min_y);
            res_max.push(max_y);
            density_vec.push(density);
            density_y_vec.push(grid_y);
            quantile_values_vec.push(quantile_y_values);
        }

        new_groups.insert(key, StatData {
            x: res_x,
            y: res_y,
            ymin: res_min,
            ymax: res_max,
            boxplot: None,
            violin: Some(ViolinData {
                density: density_vec,
                density_y: density_y_vec,
                quantile_values: quantile_values_vec,
            }),
        });
    }

    Ok(new_groups)
}

/// Compute density statistics using KDE (reuses KDE infrastructure from violin)
fn compute_density_stat(
    groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)>,
    bw_override: Option<f64>,
) -> Result<HashMap<String, StatData>> {
    // First collect all x values across all groups to determine shared range
    let mut all_x_values: Vec<f64> = Vec::new();
    for (x_strs, _, _, _) in groups.values() {
        for s in x_strs {
            let v = s.parse::<f64>().map_err(|_| anyhow!("Stat 'density' requires numeric x data"))?;
            all_x_values.push(v);
        }
    }

    if all_x_values.is_empty() {
        return Ok(groups.into_iter().map(|(k, v)| (k, StatData::from_tuple(v))).collect());
    }

    // Use shared range for all groups so density curves align
    let global_min = all_x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let global_max = all_x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let mut new_groups = HashMap::new();

    for (key, (x_strs, _, _, _)) in groups {
        let mut x_floats: Vec<f64> = Vec::new();
        for s in &x_strs {
            x_floats.push(s.parse::<f64>().unwrap());
        }

        if x_floats.is_empty() { continue; }

        // Compute bandwidth
        let bandwidth = bw_override.unwrap_or_else(|| silverman_bandwidth(&x_floats));

        // Compute KDE on a shared grid
        let grid_points: usize = 256;
        let n = x_floats.len() as f64;

        let extend = 3.0 * bandwidth;
        let x_start = global_min - extend;
        let x_end = global_max + extend;
        let range = x_end - x_start;

        if range <= 0.0 {
            new_groups.insert(key, StatData::from_tuple((
                vec![global_min.to_string()],
                vec![1.0],
                vec![0.0],
                vec![1.0],
            )));
            continue;
        }

        let step = range / (grid_points - 1) as f64;
        let mut grid_x = Vec::with_capacity(grid_points);
        let mut density = Vec::with_capacity(grid_points);

        for i in 0..grid_points {
            let x = x_start + i as f64 * step;
            grid_x.push(x);

            let mut d = 0.0;
            for &xi in &x_floats {
                let u = (x - xi) / bandwidth;
                d += gaussian_kernel(u);
            }
            d /= n * bandwidth;
            density.push(d);
        }

        // Convert to string x and build stat data
        let new_x: Vec<String> = grid_x.iter().map(|x| format!("{}", x)).collect();
        let new_ymin: Vec<f64> = vec![0.0; grid_points];
        let new_ymax: Vec<f64> = density.clone();

        new_groups.insert(key, StatData::from_tuple((new_x, density, new_ymin, new_ymax)));
    }

    Ok(new_groups)
}

fn percentile(sorted_data: &[f64], p: f64) -> f64 {
    let n = sorted_data.len();
    if n == 0 { return 0.0; }
    if n == 1 { return sorted_data[0]; }

    let rank = p * (n - 1) as f64;
    let lower_idx = rank.floor() as usize;
    let upper_idx = rank.ceil() as usize;
    
    if lower_idx == upper_idx {
        sorted_data[lower_idx]
    } else {
        let weight = rank - lower_idx as f64;
        sorted_data[lower_idx] * (1.0 - weight) + sorted_data[upper_idx] * weight
    }
}

fn apply_statistics(
    groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)>,
    stat: &Stat
) -> Result<HashMap<String, StatData>> {
    match stat {
        Stat::Identity => Ok(groups.into_iter().map(|(k, v)| (k, StatData::from_tuple(v))).collect()),
        Stat::Bin { bins } => compute_bin_stat(groups, *bins),
        Stat::Count => compute_count_stat(groups),
        Stat::Smooth { method } => compute_smooth_stat(groups, method),
        Stat::Boxplot => compute_boxplot_stat(groups),
        Stat::Violin { draw_quantiles } => compute_violin_stat(groups, draw_quantiles),
        Stat::Density { bw } => compute_density_stat(groups, *bw),
    }
}

fn compute_count_stat(
    groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)>
) -> Result<HashMap<String, StatData>> {
    let mut new_groups = HashMap::new();
    
    for (key, (x_strs, _, _, _)) in groups {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for s in x_strs {
            *counts.entry(s).or_default() += 1;
        }
        
        let mut keys: Vec<String> = counts.keys().cloned().collect();
        keys.sort();
        
        let mut new_x = Vec::new();
        let mut new_y = Vec::new();
        let mut new_ymin = Vec::new();
        let mut new_ymax = Vec::new();
        
        for k in keys {
            let count = counts[&k] as f64;
            new_x.push(k);
            new_y.push(count);
            new_ymin.push(0.0);
            new_ymax.push(count);
        }
        
        new_groups.insert(key, StatData::from_tuple((new_x, new_y, new_ymin, new_ymax)));
    }
    
    Ok(new_groups)
}

fn compute_smooth_stat(
    groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)>,
    _method: &str
) -> Result<HashMap<String, StatData>> {
    let mut new_groups = HashMap::new();
    
    for (key, (x_strs, y_vals, _, _)) in groups {
        // Simple Linear Regression
        let mut x_floats = Vec::new();
        for s in &x_strs {
            x_floats.push(s.parse::<f64>().map_err(|_| anyhow!("Stat 'smooth' requires numeric x data"))?);
        }
        
        if x_floats.len() < 2 { continue; }
        
        let n = x_floats.len() as f64;
        let sum_x: f64 = x_floats.iter().sum();
        let sum_y: f64 = y_vals.iter().sum();
        let sum_xx: f64 = x_floats.iter().map(|&x| x * x).sum();
        let sum_xy: f64 = x_floats.iter().zip(y_vals.iter()).map(|(&x, &y)| x * y).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;
        
        // Generate trend line points (min and max X)
        let min_x = x_floats.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_x = x_floats.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        let new_x = vec![min_x.to_string(), max_x.to_string()];
        let new_y = vec![slope * min_x + intercept, slope * max_x + intercept];
        let new_ymin = new_y.clone();
        let new_ymax = new_y.clone();
        
        new_groups.insert(key, StatData::from_tuple((new_x, new_y, new_ymin, new_ymax)));
    }
    
    Ok(new_groups)
}

fn compute_bin_stat(
    groups: HashMap<String, (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>)>,
    bin_count: usize
) -> Result<HashMap<String, StatData>> {
    // 1. Collect all X values to determine range
    let mut all_values = Vec::new();
    for (x_strs, _, _, _) in groups.values() {
        for s in x_strs {
            let v = s.parse::<f64>().map_err(|_| anyhow!("Stat 'bin' requires numeric x data"))?;
            all_values.push(v);
        }
    }
    
    if all_values.is_empty() { return Ok(groups.into_iter().map(|(k, v)| (k, StatData::from_tuple(v))).collect()); }

    let min = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    // Add small buffer or handle 0 range
    let range = max - min;
    let width = if range == 0.0 { 1.0 } else { range / bin_count as f64 };
    
    let mut new_groups = HashMap::new();
    
    for (key, (x_strs, _, _, _)) in groups {
        let mut bins: HashMap<isize, usize> = HashMap::new();
        
        for s in x_strs {
             // We already checked they are numeric
             let v = s.parse::<f64>().unwrap();
             let bin_idx = ((v - min) / width).floor() as isize;
             *bins.entry(bin_idx).or_default() += 1;
        }
        
        // Convert bins back to (X, Y)
        let mut bin_indices: Vec<isize> = bins.keys().cloned().collect();
        bin_indices.sort();
        
        let mut new_x = Vec::new();
        let mut new_y = Vec::new();
        let mut new_ymin = Vec::new();
        let mut new_ymax = Vec::new();
        
        for idx in bin_indices {
            let center = min + (idx as f64 * width) + (width / 2.0);
            let count = bins[&idx] as f64;
            new_x.push(format!("{:.2}", center));
            new_y.push(count);
            new_ymin.push(0.0);
            new_ymax.push(count);
        }
        
        new_groups.insert(key, StatData::from_tuple((new_x, new_y, new_ymin, new_ymax)));
    }
    
    Ok(new_groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Layer, LineLayer};

    fn make_data() -> PlotData {
        PlotData {
            headers: vec!["x".to_string(), "y".to_string(), "cat".to_string()],
            rows: vec![
                vec!["1.0".to_string(), "10.0".to_string(), "A".to_string()],
                vec!["2.0".to_string(), "20.0".to_string(), "A".to_string()],
                vec!["1.0".to_string(), "15.0".to_string(), "B".to_string()],
            ],
        }
    }

    fn make_spec() -> ResolvedSpec {
        ResolvedSpec {
            layers: vec![ResolvedLayer {
                original_layer: Layer::Line(LineLayer::default()),
                aesthetics: ResolvedAesthetics {
                    x_col: "x".to_string(),
                    y_col: Some("y".to_string()),
                    ymin_col: None,
                    ymax_col: None,
                    color: Some("cat".to_string()),
                    size: None,
                    shape: None,
                    alpha: None,
                },
            }],
            facet: None,
            coord: None,
            labels: crate::parser::ast::Labels::default(),
            theme: crate::parser::ast::Theme::default(),
            x_scale_spec: None,
            y_scale_spec: None,
        }
    }

    #[test]
    fn test_transform_grouping() {
        let csv = make_data();
        let spec = make_spec();
        let render_data = apply_transformations(&spec, &csv).unwrap();
        
        assert_eq!(render_data.panels.len(), 1);
        let panel = &render_data.panels[0];
        assert_eq!(panel.layers.len(), 1);
        let layer = &panel.layers[0];
        assert_eq!(layer.groups.len(), 2); // A and B

        // Check group A
        let group_a = layer.groups.iter().find(|g| g.key == "A").unwrap();
        assert_eq!(group_a.x.len(), 2);
        assert_eq!(group_a.y, vec![10.0, 20.0]);
    }

    #[test]
    fn test_transform_facet() {
        let mut spec = make_spec();
        spec.facet = Some(ResolvedFacet {
            col: "cat".to_string(),
            ncol: None,
            scales: crate::parser::ast::FacetScales::Fixed,
        });
        
        let csv = make_data();
        let render_data = apply_transformations(&spec, &csv).unwrap();
        
        assert_eq!(render_data.panels.len(), 2); // A and B panels
        assert_eq!(render_data.facet_layout.panel_titles.len(), 2);
        assert!(render_data.facet_layout.panel_titles.contains(&"A".to_string()));
    }
}

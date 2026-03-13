use anyhow::Result;
use crate::ir::{RenderData, ScaleSystem, PanelScales, Scale, ResolvedSpec};
use crate::parser::ast::{FacetScales, ScaleType};

/// Build the scale system for the plot
pub fn build_scales(data: &RenderData, spec: &ResolvedSpec) -> Result<ScaleSystem> {
    // 1. Calculate raw ranges per panel
    let mut panel_raw_ranges = Vec::new();
    for panel in &data.panels {
        let x_mm = calculate_min_max_x(panel);
        let y_mm = calculate_min_max_y(panel);
        panel_raw_ranges.push((x_mm, y_mm));
    }

    // 2. Determine Scale Sharing Logic
    let scales_mode = spec.facet.as_ref().map(|f| &f.scales).unwrap_or(&FacetScales::Fixed);

    // 3. Resolve final domains
    let mut final_scales = Vec::new();
    
    // Pre-calculate globals if needed
    let global_x = if matches!(scales_mode, FacetScales::Fixed | FacetScales::FreeY) {
        merge_ranges(panel_raw_ranges.iter().map(|(x, _)| x))
    } else {
        MinMax::default() // Unused
    };

    let global_y = if matches!(scales_mode, FacetScales::Fixed | FacetScales::FreeX) {
        merge_ranges(panel_raw_ranges.iter().map(|(_, y)| y))
    } else {
        MinMax::default()
    };

    for (x_local, y_local) in &panel_raw_ranges {
        let x_mm = match scales_mode {
            FacetScales::Fixed | FacetScales::FreeY => global_x.clone(),
            _ => x_local.clone(),
        };

        let y_mm = match scales_mode {
            FacetScales::Fixed | FacetScales::FreeX => global_y.clone(),
            _ => y_local.clone(),
        };

        // 4. Construct Scale objects
        // X-Axis
        let x_scale = if x_mm.is_categorical {
            // Categorical Scale
            let n = x_mm.categories.len() as f64;
            Scale {
                domain: (0.0, n),
                range: if let Some(s) = &spec.x_scale_spec {
                    if matches!(s.scale_type, ScaleType::Reverse) { (n - 0.5, -0.5) } else { (-0.5, n - 0.5) }
                } else { (-0.5, n - 0.5) },
                is_categorical: true,
                categories: x_mm.categories,
            }
        } else {
            // Continuous Scale
            let (min, max) = if let Some(s) = &spec.x_scale_spec {
                if let Some((lmin, lmax)) = s.limits { (lmin, lmax) }
                else { pad_range(x_mm.min, x_mm.max) }
            } else { pad_range(x_mm.min, x_mm.max) };

            Scale {
                domain: (min, max),
                range: if let Some(s) = &spec.x_scale_spec {
                    if matches!(s.scale_type, ScaleType::Reverse) { (max, min) } else { (min, max) }
                } else { (min, max) },
                is_categorical: false,
                categories: Vec::new(),
            }
        };

        // Y-Axis
        let y_scale = if y_mm.is_categorical {
            let n = y_mm.categories.len() as f64;
            Scale {
                domain: (0.0, n),
                range: if let Some(s) = &spec.y_scale_spec {
                    if matches!(s.scale_type, ScaleType::Reverse) { (n - 0.5, -0.5) } else { (-0.5, n - 0.5) }
                } else { (-0.5, n - 0.5) },
                is_categorical: true,
                categories: y_mm.categories,
            }
        } else {
            let (min, max) = if let Some(s) = &spec.y_scale_spec {
                if let Some((lmin, lmax)) = s.limits { (lmin, lmax) }
                else { pad_range(y_mm.min, y_mm.max) }
            } else { pad_range(y_mm.min, y_mm.max) };

            Scale {
                domain: (min, max),
                range: if let Some(s) = &spec.y_scale_spec {
                    if matches!(s.scale_type, ScaleType::Reverse) { (max, min) } else { (min, max) }
                } else { (min, max) },
                is_categorical: false,
                categories: Vec::new(),
            }
        };

        final_scales.push(PanelScales {
            x: x_scale,
            y: y_scale,
        });
    }

    Ok(ScaleSystem { panels: final_scales })
}

#[derive(Debug, Clone, Default)]
struct MinMax {
    min: f64,
    max: f64,
    is_categorical: bool,
    categories: Vec<String>,
}

fn calculate_min_max_x(panel: &crate::ir::PanelData) -> MinMax {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut categories = Vec::new();
    let mut is_cat = false;

    for layer in &panel.layers {
        for group in &layer.groups {
            if let Some(cats) = &group.x_categories {
                is_cat = true;
                // Merge categories? For now, assume consistent across layers or take first non-empty
                if categories.is_empty() {
                    categories = cats.clone();
                }
            }
            
            for &val in &group.x {
                if val < min { min = val; }
                if val > max { max = val; }
            }
        }
    }

    if is_cat {
        // For categorical, range is determined by number of categories
        // Indices are 0..N-1
        min = 0.0;
        max = (categories.len().max(1) - 1) as f64;
    }

    MinMax { min, max, is_categorical: is_cat, categories }
}

fn calculate_min_max_y(panel: &crate::ir::PanelData) -> MinMax {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;

    // Helper to include 0 for bar charts
    let mut has_bars = false;
    let mut is_cat = false;
    let mut categories = Vec::new();

    for layer in &panel.layers {
        for group in &layer.groups {
            // Check if bar layer
            if matches!(group.style, crate::ir::RenderStyle::Bar(_)) {
                has_bars = true;
            }

            // Check for heatmap with y categories
            if let Some(y_cats) = &group.y_categories {
                is_cat = true;
                if categories.is_empty() {
                    categories = y_cats.clone();
                }
            }

            // For heatmap, use y_positions for range
            if matches!(group.style, crate::ir::RenderStyle::Heatmap(_)) {
                for &val in &group.heatmap_y_positions {
                    if val < min { min = val; }
                    if val > max { max = val; }
                }
                // Include cell extent
                let half_h = group.heatmap_cell_height / 2.0;
                if !group.heatmap_y_positions.is_empty() {
                    let ext_min = group.heatmap_y_positions.iter().fold(f64::INFINITY, |a, &b| a.min(b)) - half_h;
                    let ext_max = group.heatmap_y_positions.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)) + half_h;
                    if ext_min < min { min = ext_min; }
                    if ext_max > max { max = ext_max; }
                }
                continue;
            }

            // Check y (and y_start for stacked)
            for &val in &group.y {
                if val < min { min = val; }
                if val > max { max = val; }
            }
            for &val in &group.y_start {
                if val < min { min = val; }
                if val > max { max = val; }
            }
            for &val in &group.y_min {
                if val < min { min = val; }
                if val > max { max = val; }
            }
            for &val in &group.y_max {
                if val < min { min = val; }
                if val > max { max = val; }
            }
            for outlier_set in &group.outliers {
                for &val in outlier_set {
                    if val < min { min = val; }
                    if val > max { max = val; }
                }
            }
        }
    }

    if has_bars {
        // Bar charts always include 0
        if min > 0.0 { min = 0.0; }
        if max < 0.0 { max = 0.0; }
    }

    if is_cat {
        min = 0.0;
        max = (categories.len().max(1) - 1) as f64;
    }

    MinMax { min, max, is_categorical: is_cat, categories }
}

fn merge_ranges<'a, I>(iter: I) -> MinMax 
where I: Iterator<Item = &'a MinMax> 
{
    let mut global = MinMax { min: f64::INFINITY, max: f64::NEG_INFINITY, is_categorical: false, categories: Vec::new() };
    
    for local in iter {
        if local.min < global.min { global.min = local.min; }
        if local.max > global.max { global.max = local.max; }
        if local.is_categorical {
            global.is_categorical = true;
            // Naive merge: if one has categories, take them.
            // Ideally should union and sort, but transform layer should ensure consistency.
            if global.categories.is_empty() {
                global.categories = local.categories.clone();
            }
        }
    }
    
    // Handle empty case
    if global.min == f64::INFINITY { global.min = 0.0; global.max = 1.0; }
    
    global
}

fn pad_range(min: f64, max: f64) -> (f64, f64) {
    if min == max {
        (min - 1.0, max + 1.0)
    } else {
        let padding = (max - min) * 0.05;
        (min - padding, max + padding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{PanelData, LayerData, GroupData, FacetLayout, RenderStyle};
    use crate::graph::LineStyle;

    fn make_render_data(x: Vec<f64>, y: Vec<f64>) -> RenderData {
        RenderData {
            panels: vec![PanelData {
                index: 0,
                layers: vec![LayerData {
                    groups: vec![GroupData {
                        key: "A".to_string(),
                        x,
                        y: y,
                        y_start: vec![],
                        y_min: vec![],
                        y_max: vec![],
                        y_q1: vec![],
                        y_median: vec![],
                        y_q3: vec![],
                        outliers: vec![],
                        violin_density: vec![],
                        violin_density_y: vec![],
                        violin_quantile_values: vec![],
                        heatmap_y_positions: vec![],
                        heatmap_fill_values: vec![],
                        heatmap_cell_width: 0.0,
                        heatmap_cell_height: 0.0,
                        x_categories: None,
                        y_categories: None,
                        style: RenderStyle::Line(LineStyle::default()),
                    }],
                }],
            }],
            facet_layout: FacetLayout { nrow: 1, ncol: 1, panel_titles: vec![] },
        }
    }

    fn make_resolved_spec() -> ResolvedSpec {
        ResolvedSpec {
            layers: vec![],
            facet: None,
            coord: None,
            labels: crate::parser::ast::Labels::default(),
            theme: crate::parser::ast::Theme::default(),
            x_scale_spec: None,
            y_scale_spec: None,
        }
    }

    #[test]
    fn test_scale_continuous() {
        let data = make_render_data(vec![0.0, 10.0], vec![0.0, 100.0]);
        let spec = make_resolved_spec();
        let scales = build_scales(&data, &spec).unwrap();
        
        assert_eq!(scales.panels.len(), 1);
        let panel = &scales.panels[0];
        
        // Check padding
        assert!(panel.x.domain.0 < 0.0);
        assert!(panel.x.domain.1 > 10.0);
        assert!(!panel.x.is_categorical);
    }

    #[test]
    fn test_scale_single_point() {
        let data = make_render_data(vec![5.0], vec![5.0]);
        let spec = make_resolved_spec();
        let scales = build_scales(&data, &spec).unwrap();
        
        let panel = &scales.panels[0];
        assert_eq!(panel.x.domain.0, 4.0);
        assert_eq!(panel.x.domain.1, 6.0);
    }
    
    #[test]
    fn test_scale_categorical() {
        let mut data = make_render_data(vec![0.0, 1.0], vec![10.0, 20.0]);
        // Modify to simulate categorical
        data.panels[0].layers[0].groups[0].x_categories = Some(vec!["A".to_string(), "B".to_string()]);
        
        let spec = make_resolved_spec();
        let scales = build_scales(&data, &spec).unwrap();
        let panel = &scales.panels[0];
        
        assert!(panel.x.is_categorical);
        assert_eq!(panel.x.categories, vec!["A", "B"]);
        assert_eq!(panel.x.range, (-0.5, 1.5));
    }
}

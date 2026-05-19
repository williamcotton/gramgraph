use crate::datetime::{parse_datetime_interval_seconds, DEFAULT_DATETIME_FORMAT};
use crate::ir::{
    AxisTransform, DateTimeScale, PanelScales, RenderData, ResolvedSpec, Scale, ScaleSystem,
};
use crate::parser::ast::{AxisScale, FacetScales, ScaleType};
use anyhow::{anyhow, Result};

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
    let scales_mode = spec
        .facet
        .as_ref()
        .map(|f| &f.scales)
        .unwrap_or(&FacetScales::Fixed);

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
                    if matches!(s.scale_type, ScaleType::Reverse) {
                        (n - 0.5, -0.5)
                    } else {
                        (-0.5, n - 0.5)
                    }
                } else {
                    (-0.5, n - 0.5)
                },
                is_categorical: true,
                categories: x_mm.categories,
                tick_positions: vec![],
                datetime: None,
                transform: AxisTransform::Linear,
            }
        } else {
            build_continuous_scale(&x_mm, spec.x_scale_spec.as_ref(), "x")?
        };

        // Y-Axis
        let y_scale = if y_mm.is_categorical {
            let n = y_mm.categories.len() as f64;
            Scale {
                domain: (0.0, n),
                range: if let Some(s) = &spec.y_scale_spec {
                    if matches!(s.scale_type, ScaleType::Reverse) {
                        (n - 0.5, -0.5)
                    } else {
                        (-0.5, n - 0.5)
                    }
                } else {
                    (-0.5, n - 0.5)
                },
                is_categorical: true,
                categories: y_mm.categories,
                tick_positions: vec![],
                datetime: None,
                transform: AxisTransform::Linear,
            }
        } else {
            build_continuous_scale(&y_mm, spec.y_scale_spec.as_ref(), "y")?
        };

        final_scales.push(PanelScales {
            x: x_scale,
            y: y_scale,
        });
    }

    Ok(ScaleSystem {
        panels: final_scales,
    })
}

fn build_continuous_scale(
    mm: &MinMax,
    axis_scale: Option<&AxisScale>,
    axis_name: &str,
) -> Result<Scale> {
    let is_datetime = axis_scale.is_some_and(|s| matches!(s.scale_type, ScaleType::DateTime));
    let transform = axis_transform(axis_scale);
    let reverse = axis_scale.is_some_and(|s| matches!(s.scale_type, ScaleType::Reverse));
    let raw_min = mm.min;
    let raw_max = mm.max;

    if is_datetime {
        let datetime_range = if raw_min == raw_max {
            (raw_min - 1.0, raw_max + 1.0)
        } else {
            (raw_min, raw_max)
        };

        return Ok(Scale {
            domain: (raw_min, raw_max),
            range: datetime_range,
            is_categorical: false,
            categories: Vec::new(),
            tick_positions: vec![],
            datetime: build_datetime_scale(axis_scale)?,
            transform: AxisTransform::Linear,
        });
    }

    let (min, max, ticks) = if let Some(scale) = axis_scale {
        if let Some((lmin, lmax)) = scale.limits {
            transformed_ticks_within(lmin, lmax, transform, 8, axis_name)?
        } else {
            transformed_nice_range(raw_min, raw_max, transform, 8, axis_name)?
        }
    } else {
        transformed_nice_range(raw_min, raw_max, transform, 8, axis_name)?
    };

    Ok(Scale {
        domain: (min, max),
        range: if reverse { (max, min) } else { (min, max) },
        is_categorical: false,
        categories: Vec::new(),
        tick_positions: ticks,
        datetime: None,
        transform,
    })
}

fn axis_transform(axis_scale: Option<&AxisScale>) -> AxisTransform {
    match axis_scale.map(|s| &s.scale_type) {
        Some(ScaleType::Log10) => AxisTransform::Log10,
        Some(ScaleType::Sqrt) => AxisTransform::Sqrt,
        _ => AxisTransform::Linear,
    }
}

fn transformed_nice_range(
    raw_min: f64,
    raw_max: f64,
    transform: AxisTransform,
    target_count: usize,
    axis_name: &str,
) -> Result<(f64, f64, Vec<f64>)> {
    let (raw_min, raw_max) = padded_raw_range(raw_min, raw_max, transform, axis_name)?;

    match transform {
        AxisTransform::Linear => {
            let ((min, max), ticks) = nice_range(raw_min, raw_max, target_count);
            Ok((min, max, ticks))
        }
        AxisTransform::Log10 => {
            ensure_transform_domain(raw_min, raw_max, transform, axis_name)?;
            let min = raw_min.log10().floor();
            let max = raw_max.log10().ceil();
            let ticks = integer_ticks(min, max);
            Ok((min, max, ticks))
        }
        AxisTransform::Sqrt => {
            ensure_transform_domain(raw_min, raw_max, transform, axis_name)?;
            let ((mut nice_min, nice_max), raw_ticks) = nice_range(raw_min, raw_max, target_count);
            if nice_min < 0.0 {
                nice_min = 0.0;
            }
            let min = nice_min.sqrt();
            let max = nice_max.sqrt();
            let ticks = raw_ticks
                .into_iter()
                .filter(|tick| *tick >= 0.0)
                .map(|tick| tick.sqrt())
                .collect();
            Ok((min, max, ticks))
        }
    }
}

fn transformed_ticks_within(
    raw_min: f64,
    raw_max: f64,
    transform: AxisTransform,
    target_count: usize,
    axis_name: &str,
) -> Result<(f64, f64, Vec<f64>)> {
    ensure_transform_domain(raw_min, raw_max, transform, axis_name)?;

    match transform {
        AxisTransform::Linear => {
            let ticks = nice_ticks_within(raw_min, raw_max, target_count);
            Ok((raw_min, raw_max, ticks))
        }
        AxisTransform::Log10 => {
            let min = raw_min.log10();
            let max = raw_max.log10();
            let ticks = integer_ticks(min.ceil(), max.floor());
            Ok((min, max, ticks))
        }
        AxisTransform::Sqrt => {
            let ticks = nice_ticks_within(raw_min, raw_max, target_count)
                .into_iter()
                .filter(|tick| *tick >= 0.0)
                .map(|tick| tick.sqrt())
                .collect();
            Ok((raw_min.sqrt(), raw_max.sqrt(), ticks))
        }
    }
}

fn padded_raw_range(
    raw_min: f64,
    raw_max: f64,
    transform: AxisTransform,
    axis_name: &str,
) -> Result<(f64, f64)> {
    if raw_min != raw_max {
        ensure_transform_domain(raw_min, raw_max, transform, axis_name)?;
        return Ok((raw_min, raw_max));
    }

    let range = match transform {
        AxisTransform::Linear => (raw_min - 1.0, raw_max + 1.0),
        AxisTransform::Log10 => {
            ensure_transform_domain(raw_min, raw_max, transform, axis_name)?;
            (raw_min / 10.0, raw_max * 10.0)
        }
        AxisTransform::Sqrt => {
            ensure_transform_domain(raw_min, raw_max, transform, axis_name)?;
            if raw_min <= 0.0 {
                (0.0, 1.0)
            } else {
                ((raw_min - 1.0).max(0.0), raw_max + 1.0)
            }
        }
    };

    Ok(range)
}

fn ensure_transform_domain(
    raw_min: f64,
    raw_max: f64,
    transform: AxisTransform,
    axis_name: &str,
) -> Result<()> {
    if !raw_min.is_finite() || !raw_max.is_finite() {
        return Err(anyhow!("{} scale requires finite values", axis_name));
    }

    match transform {
        AxisTransform::Linear => Ok(()),
        AxisTransform::Log10 if raw_min <= 0.0 || raw_max <= 0.0 => Err(anyhow!(
            "scale_{}_log10() requires positive {} values",
            axis_name,
            axis_name
        )),
        AxisTransform::Sqrt if raw_min < 0.0 || raw_max < 0.0 => Err(anyhow!(
            "scale_{}_sqrt() requires non-negative {} values",
            axis_name,
            axis_name
        )),
        _ => Ok(()),
    }
}

fn integer_ticks(min: f64, max: f64) -> Vec<f64> {
    let start = min.ceil() as i32;
    let end = max.floor() as i32;
    if start > end {
        return vec![];
    }

    (start..=end).map(|value| value as f64).collect()
}

fn build_datetime_scale(
    axis_scale: Option<&crate::parser::ast::AxisScale>,
) -> Result<Option<DateTimeScale>> {
    let Some(axis_scale) = axis_scale else {
        return Ok(None);
    };

    if !matches!(axis_scale.scale_type, ScaleType::DateTime) {
        return Ok(None);
    }

    let options = axis_scale.datetime.as_ref();
    let interval_seconds = options
        .and_then(|dt| dt.interval.as_ref())
        .map(|value| parse_datetime_interval_seconds(value))
        .transpose()?;
    let label_format = options
        .and_then(|dt| dt.format.clone())
        .unwrap_or_else(|| DEFAULT_DATETIME_FORMAT.to_string());

    Ok(Some(DateTimeScale {
        interval_seconds,
        label_format,
    }))
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

            let x_padding = match &group.style {
                crate::ir::RenderStyle::ErrorBar { width, .. } => width / 2.0,
                _ => 0.0,
            };

            for &val in &group.x {
                let padded_min = val - x_padding;
                let padded_max = val + x_padding;
                if padded_min < min {
                    min = padded_min;
                }
                if padded_max > max {
                    max = padded_max;
                }
            }
        }
    }

    if is_cat {
        // For categorical, range is determined by number of categories
        // Indices are 0..N-1
        min = 0.0;
        max = (categories.len().max(1) - 1) as f64;
    }

    MinMax {
        min,
        max,
        is_categorical: is_cat,
        categories,
    }
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
                    if val < min {
                        min = val;
                    }
                    if val > max {
                        max = val;
                    }
                }
                // Include cell extent
                let half_h = group.heatmap_cell_height / 2.0;
                if !group.heatmap_y_positions.is_empty() {
                    let ext_min = group
                        .heatmap_y_positions
                        .iter()
                        .fold(f64::INFINITY, |a, &b| a.min(b))
                        - half_h;
                    let ext_max = group
                        .heatmap_y_positions
                        .iter()
                        .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
                        + half_h;
                    if ext_min < min {
                        min = ext_min;
                    }
                    if ext_max > max {
                        max = ext_max;
                    }
                }
                continue;
            }

            // Check y (and y_start for stacked)
            for &val in &group.y {
                if val < min {
                    min = val;
                }
                if val > max {
                    max = val;
                }
            }
            for &val in &group.y_start {
                if val < min {
                    min = val;
                }
                if val > max {
                    max = val;
                }
            }
            for &val in &group.y_min {
                if val < min {
                    min = val;
                }
                if val > max {
                    max = val;
                }
            }
            for &val in &group.y_max {
                if val < min {
                    min = val;
                }
                if val > max {
                    max = val;
                }
            }
            for outlier_set in &group.outliers {
                for &val in outlier_set {
                    if val < min {
                        min = val;
                    }
                    if val > max {
                        max = val;
                    }
                }
            }
        }
    }

    if has_bars {
        // Bar charts always include 0
        if min > 0.0 {
            min = 0.0;
        }
        if max < 0.0 {
            max = 0.0;
        }
    }

    if is_cat {
        min = 0.0;
        max = (categories.len().max(1) - 1) as f64;
    }

    MinMax {
        min,
        max,
        is_categorical: is_cat,
        categories,
    }
}

fn merge_ranges<'a, I>(iter: I) -> MinMax
where
    I: Iterator<Item = &'a MinMax>,
{
    let mut global = MinMax {
        min: f64::INFINITY,
        max: f64::NEG_INFINITY,
        is_categorical: false,
        categories: Vec::new(),
    };

    for local in iter {
        if local.min < global.min {
            global.min = local.min;
        }
        if local.max > global.max {
            global.max = local.max;
        }
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
    if global.min == f64::INFINITY {
        global.min = 0.0;
        global.max = 1.0;
    }

    global
}

/// Find the nearest "nice" step size (1, 2, 5 × 10^n) for a given range and target tick count.
fn nice_step(data_range: f64, target_count: usize) -> f64 {
    if data_range <= 0.0 || target_count == 0 {
        return 1.0;
    }
    let rough_step = data_range / target_count as f64;
    let magnitude = 10.0_f64.powf(rough_step.log10().floor());
    let residual = rough_step / magnitude;

    let nice = if residual <= 1.5 {
        1.0
    } else if residual <= 3.5 {
        2.0
    } else if residual <= 7.5 {
        5.0
    } else {
        10.0
    };

    nice * magnitude
}

/// Expand min/max to nice boundaries and compute tick positions.
/// Returns ((nice_min, nice_max), tick_positions).
fn nice_range(data_min: f64, data_max: f64, target_count: usize) -> ((f64, f64), Vec<f64>) {
    if data_min == data_max {
        let ticks = vec![data_min - 1.0, data_min, data_min + 1.0];
        return ((data_min - 1.0, data_max + 1.0), ticks);
    }

    let step = nice_step(data_max - data_min, target_count);
    let nice_min = (data_min / step).floor() * step;
    let nice_max = (data_max / step).ceil() * step;

    let mut ticks = Vec::new();
    let mut v = nice_min;
    // Use a small epsilon to avoid floating-point drift missing the last tick
    while v <= nice_max + step * 1e-9 {
        ticks.push(v);
        v += step;
    }

    ((nice_min, nice_max), ticks)
}

/// Compute nice tick positions within user-specified limits (no domain expansion).
fn nice_ticks_within(min: f64, max: f64, target_count: usize) -> Vec<f64> {
    if min == max {
        return vec![min];
    }
    let step = nice_step(max - min, target_count);
    let start = (min / step).ceil() * step;
    let mut ticks = Vec::new();
    let mut v = start;
    while v <= max + step * 1e-9 {
        ticks.push(v);
        v += step;
    }
    ticks
}

/// Format a tick value cleanly: integer if whole, trimmed trailing zeros otherwise.
pub fn format_nice_number(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 && v.abs() < 1e15 {
        format!("{}", v.round() as i64)
    } else {
        // Use enough precision, then trim trailing zeros
        let s = format!("{:.10}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::LineStyle;
    use crate::ir::{AxisTransform, FacetLayout, GroupData, LayerData, PanelData, RenderStyle};

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
            facet_layout: FacetLayout {
                nrow: 1,
                ncol: 1,
                panel_titles: vec![],
            },
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

        // Nice range should snap to clean boundaries
        assert!(panel.x.domain.0 <= 0.0);
        assert!(panel.x.domain.1 >= 10.0);
        assert!(!panel.x.is_categorical);
        // Should have nice tick positions
        assert!(!panel.x.tick_positions.is_empty());
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
        data.panels[0].layers[0].groups[0].x_categories =
            Some(vec!["A".to_string(), "B".to_string()]);

        let spec = make_resolved_spec();
        let scales = build_scales(&data, &spec).unwrap();
        let panel = &scales.panels[0];

        assert!(panel.x.is_categorical);
        assert_eq!(panel.x.categories, vec!["A", "B"]);
        assert_eq!(panel.x.range, (-0.5, 1.5));
        assert!(panel.x.tick_positions.is_empty());
    }

    #[test]
    fn test_scale_log10_transforms_domain_and_ticks() {
        let data = make_render_data(vec![1.0, 1000.0], vec![1.0, 2.0]);
        let mut spec = make_resolved_spec();
        spec.x_scale_spec = Some(crate::parser::ast::AxisScale {
            scale_type: ScaleType::Log10,
            limits: None,
            datetime: None,
        });

        let scales = build_scales(&data, &spec).unwrap();
        let panel = &scales.panels[0];

        assert_eq!(panel.x.transform, AxisTransform::Log10);
        assert_eq!(panel.x.range, (0.0, 3.0));
        assert_eq!(panel.x.tick_positions, vec![0.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_scale_log10_rejects_non_positive_values() {
        let data = make_render_data(vec![0.0, 10.0], vec![1.0, 2.0]);
        let mut spec = make_resolved_spec();
        spec.x_scale_spec = Some(crate::parser::ast::AxisScale {
            scale_type: ScaleType::Log10,
            limits: None,
            datetime: None,
        });

        let err = build_scales(&data, &spec).unwrap_err();
        assert!(err.to_string().contains("requires positive x values"));
    }

    #[test]
    fn test_scale_sqrt_transforms_domain_and_ticks() {
        let data = make_render_data(vec![0.0, 100.0], vec![1.0, 2.0]);
        let mut spec = make_resolved_spec();
        spec.x_scale_spec = Some(crate::parser::ast::AxisScale {
            scale_type: ScaleType::Sqrt,
            limits: None,
            datetime: None,
        });

        let scales = build_scales(&data, &spec).unwrap();
        let panel = &scales.panels[0];

        assert_eq!(panel.x.transform, AxisTransform::Sqrt);
        assert_eq!(panel.x.range, (0.0, 10.0));
        assert!(panel.x.tick_positions.contains(&0.0));
        assert!(panel.x.tick_positions.contains(&10.0));
    }

    #[test]
    fn test_nice_step_small_range() {
        // Range 10, target 8 => rough_step 1.25 => magnitude 1, residual 1.25 => nice 1 => step 1
        let step = nice_step(10.0, 8);
        assert_eq!(step, 1.0);
    }

    #[test]
    fn test_nice_step_large_range() {
        // Range 1000, target 8 => rough_step 125 => magnitude 100, residual 1.25 => nice 1 => step 100
        let step = nice_step(1000.0, 8);
        assert_eq!(step, 100.0);
    }

    #[test]
    fn test_nice_step_fractional_range() {
        // Range 0.5, target 8 => rough 0.0625 => mag 0.01, res 6.25 => nice 10 => 0.1
        // Actually: log10(0.0625) = -1.204, floor = -2, mag = 0.01, res = 6.25 => nice 10 => 0.1
        // But rough_step 0.0625: log10(0.0625) ≈ -1.204, floor(-1.204) = -2, mag = 0.01, res = 6.25 => nice 5 => 0.05
        let step = nice_step(0.5, 8);
        assert_eq!(step, 0.05);
    }

    #[test]
    fn test_nice_range_zero_to_ten() {
        let ((nmin, nmax), ticks) = nice_range(0.0, 10.0, 8);
        assert_eq!(nmin, 0.0);
        assert_eq!(nmax, 10.0);
        // Step is 1, so ticks: 0..10
        assert_eq!(
            ticks,
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        );
    }

    #[test]
    fn test_nice_range_ugly_boundaries() {
        // Data from -0.385 to 7.585 should snap to clean values
        let ((nmin, nmax), ticks) = nice_range(-0.385, 7.585, 8);
        assert_eq!(nmin, -1.0);
        assert_eq!(nmax, 8.0);
        // All ticks should be integers
        for t in &ticks {
            assert_eq!(*t, t.round(), "tick {} is not a round number", t);
        }
    }

    #[test]
    fn test_nice_range_single_value() {
        let ((nmin, nmax), ticks) = nice_range(5.0, 5.0, 8);
        assert_eq!(nmin, 4.0);
        assert_eq!(nmax, 6.0);
        assert_eq!(ticks.len(), 3);
    }

    #[test]
    fn test_nice_range_negative_values() {
        let ((nmin, nmax), ticks) = nice_range(-15.0, -3.0, 8);
        assert!(nmin <= -15.0);
        assert!(nmax >= -3.0);
        for t in &ticks {
            assert_eq!(*t, t.round(), "tick {} is not a round number", t);
        }
    }

    #[test]
    fn test_nice_ticks_within() {
        let ticks = nice_ticks_within(0.0, 100.0, 8);
        assert!(!ticks.is_empty());
        assert!(*ticks.first().unwrap() >= 0.0);
        assert!(*ticks.last().unwrap() <= 100.0 + 1e-9);
    }

    #[test]
    fn test_format_nice_number_integers() {
        assert_eq!(format_nice_number(0.0), "0");
        assert_eq!(format_nice_number(5.0), "5");
        assert_eq!(format_nice_number(-10.0), "-10");
        assert_eq!(format_nice_number(100.0), "100");
    }

    #[test]
    fn test_format_nice_number_decimals() {
        assert_eq!(format_nice_number(0.5), "0.5");
        assert_eq!(format_nice_number(2.5), "2.5");
        assert_eq!(format_nice_number(0.25), "0.25");
    }
}

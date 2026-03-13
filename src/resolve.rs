use anyhow::Result;
use crate::parser::ast::{PlotSpec, Layer, Aesthetics, AestheticValue};
use crate::data::PlotData;
use crate::ir::{ResolvedSpec, ResolvedLayer, ResolvedAesthetics, ResolvedFacet};

/// Resolve all aesthetic mappings for the entire plot
pub fn resolve_plot_aesthetics(
    spec: &PlotSpec,
    _data: &PlotData,
) -> Result<ResolvedSpec> {
    // 0. Resolve global aesthetics (simple clone now)
    let resolved_aes = spec.aesthetics.clone();

    // 1. Resolve Facet (if any)
    let facet = if let Some(f) = &spec.facet {
        Some(ResolvedFacet {
            col: f.by.clone(),
            ncol: f.ncol,
            scales: f.scales.clone(),
        })
    } else {
        None
    };

    // 2. Resolve layers
    let mut layers = Vec::new();
    for layer in &spec.layers {
        // Layer variables are already resolved by preprocessor
        // Just resolve aesthetics
        let aesthetics = resolve_layer_aesthetics(layer, &resolved_aes)?;
        layers.push(ResolvedLayer {
            original_layer: layer.clone(),
            aesthetics,
        });
    }

    // 3. Resolve labels (simple clone now)
    let labels = spec.labels.clone().unwrap_or_default();

    Ok(ResolvedSpec {
        layers,
        facet,
        coord: spec.coord.clone(),
        labels,
        theme: spec.theme.clone().unwrap_or_default(),
        x_scale_spec: spec.x_scale.clone(),
        y_scale_spec: spec.y_scale.clone(),
    })
}

/// Resolve all aesthetic mappings for a single layer (layer-specific + global)
fn resolve_layer_aesthetics(
    layer: &Layer,
    global_aes: &Option<Aesthetics>,
) -> Result<ResolvedAesthetics> {
    // Resolve x and y (required)
    let (x_col, y_col) = resolve_positional(layer, global_aes)?;

    // Resolve color mapping
    let color = match layer {
        Layer::Line(l) => extract_mapped_string(&l.color),
        Layer::Point(p) => extract_mapped_string(&p.color),
        Layer::Bar(b) => extract_mapped_string(&b.color),
        Layer::Ribbon(r) => extract_mapped_string(&r.color),
        Layer::Boxplot(b) => extract_mapped_string(&b.color),
        Layer::Violin(v) => extract_mapped_string(&v.color),
        Layer::Density(d) => extract_mapped_string(&d.color),
        Layer::Heatmap(_) => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.color.clone()));

    // Resolve size mapping
    let size = match layer {
        Layer::Line(l) => extract_mapped_string_from_f64(&l.width), // width can be data-driven
        Layer::Point(p) => extract_mapped_string_from_f64(&p.size),
        Layer::Bar(b) => extract_mapped_string_from_f64(&b.width),
        Layer::Ribbon(_) => None,
        Layer::Boxplot(b) => extract_mapped_string_from_f64(&b.width),
        Layer::Violin(v) => extract_mapped_string_from_f64(&v.width),
        Layer::Density(_) => None,
        Layer::Heatmap(_) => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.size.clone()));

    // Resolve shape mapping (point only)
    let shape = match layer {
        Layer::Point(p) => extract_mapped_string(&p.shape),
        Layer::Line(_) | Layer::Bar(_) | Layer::Ribbon(_) | Layer::Boxplot(_) | Layer::Violin(_) | Layer::Density(_) | Layer::Heatmap(_) => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.shape.clone()));

    // Resolve alpha mapping
    let alpha = match layer {
        Layer::Line(l) => extract_mapped_string_from_f64(&l.alpha),
        Layer::Point(p) => extract_mapped_string_from_f64(&p.alpha),
        Layer::Bar(b) => extract_mapped_string_from_f64(&b.alpha),
        Layer::Ribbon(r) => extract_mapped_string_from_f64(&r.alpha),
        Layer::Boxplot(b) => extract_mapped_string_from_f64(&b.alpha),
        Layer::Violin(v) => extract_mapped_string_from_f64(&v.alpha),
        Layer::Density(d) => extract_mapped_string_from_f64(&d.alpha),
        Layer::Heatmap(h) => extract_mapped_string_from_f64(&h.alpha),
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.alpha.clone()));

    // Resolve ymin/ymax
    let ymin_col = match layer {
        Layer::Ribbon(r) => r.ymin.clone(),
        _ => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.ymin.clone()));

    let ymax_col = match layer {
        Layer::Ribbon(r) => r.ymax.clone(),
        _ => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.ymax.clone()));

    // Resolve fill column (heatmap value)
    let fill = match layer {
        Layer::Heatmap(h) => h.fill.clone(),
        _ => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.fill.clone()));

    Ok(ResolvedAesthetics {
        x_col,
        y_col,
        ymin_col,
        ymax_col,
        color,
        size,
        shape,
        alpha,
        fill,
    })
}

/// Resolve x and y aesthetics
fn resolve_positional(layer: &Layer, global_aes: &Option<Aesthetics>) -> Result<(String, Option<String>)> {
    let (x_override, y_override) = match layer {
        Layer::Line(l) => (l.x.as_ref(), l.y.as_ref()),
        Layer::Point(p) => (p.x.as_ref(), p.y.as_ref()),
        Layer::Bar(b) => (b.x.as_ref(), b.y.as_ref()),
        Layer::Ribbon(r) => (r.x.as_ref(), None), // Ribbon uses ymin/ymax primarily
        Layer::Boxplot(b) => (b.x.as_ref(), b.y.as_ref()),
        Layer::Violin(v) => (v.x.as_ref(), v.y.as_ref()),
        Layer::Density(d) => (d.x.as_ref(), None), // Density only needs x
        Layer::Heatmap(h) => (h.x.as_ref(), h.y.as_ref()),
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
        Some(y.clone())
    } else if let Some(ref aes) = global_aes {
        aes.y.clone()
    } else {
        // y is optional for some layers (e.g. histogram)
        None
    };
    
    // Validation: Check if y is required but missing
    if y_col.is_none() {
        match layer {
            Layer::Bar(b) if matches!(b.stat, crate::parser::ast::Stat::Bin { .. } | crate::parser::ast::Stat::Count) => {
                // Allowed
            },
            Layer::Ribbon(_) => {
                // Allowed (uses ymin/ymax)
            },
            Layer::Density(_) => {
                // Allowed (density computes y from x via KDE)
            },
            _ => {
                 anyhow::bail!("No y aesthetic specified (use aes(x: ..., y: ...) or layer-level y: ...)");
            }
        }
    }

    Ok((x_col, y_col))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Aesthetics, Layer, LineLayer, PointLayer, PlotSpec};
    use crate::data::PlotData;

    fn make_data() -> PlotData {
        PlotData {
            headers: vec!["x".to_string(), "y".to_string(), "g".to_string()],
            rows: vec![],
        }
    }

    #[test]
    fn test_resolve_simple() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: Some("y".to_string()),
                color: None,
                size: None,
                shape: None,
                alpha: None,
                ymin: None,
                ymax: None,
                fill: None,
            }),
            layers: vec![Layer::Line(LineLayer::default())],
            labels: Some(crate::parser::ast::Labels::default()),
            facet: None,
            coord: None,
            theme: None,
            x_scale: None,
            y_scale: None,
        };
        let data = make_data();
        let resolved = resolve_plot_aesthetics(&spec, &data).unwrap();
        assert_eq!(resolved.layers.len(), 1);
        assert_eq!(resolved.layers[0].aesthetics.x_col, "x");
        assert_eq!(resolved.layers[0].aesthetics.y_col, Some("y".to_string()));
    }

    #[test]
    fn test_resolve_override() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: Some("y".to_string()),
                color: None,
                size: None,
                shape: None,
                alpha: None,
                ymin: None,
                ymax: None,
                fill: None,
            }),
            layers: vec![Layer::Point(PointLayer {
                x: None,
                y: Some("g".to_string()),
                ..Default::default()
            })],
            labels: Some(crate::parser::ast::Labels::default()),
            facet: None,
            coord: None,
            theme: None,
            x_scale: None,
            y_scale: None,
        };
        let data = make_data();
        let resolved = resolve_plot_aesthetics(&spec, &data).unwrap();
        assert_eq!(resolved.layers[0].aesthetics.y_col, Some("g".to_string()));
    }

    #[test]
    fn test_resolve_missing_aes() {
        let spec = PlotSpec {
            aesthetics: None,
            layers: vec![Layer::Line(LineLayer::default())],
            labels: None,
            facet: None,
            coord: None,
            theme: None,
            x_scale: None,
            y_scale: None,
        };
        let data = make_data();
        let res = resolve_plot_aesthetics(&spec, &data);
        assert!(res.is_err());
    }

    #[test]
    fn test_resolve_facet() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: Some("y".to_string()),
                color: None,
                size: None,
                shape: None,
                alpha: None,
                ymin: None,
                ymax: None,
                fill: None,
            }),
            layers: vec![],
            labels: Some(crate::parser::ast::Labels::default()),
            facet: Some(crate::parser::ast::Facet {
                by: "g".to_string(),
                ncol: None,
                scales: crate::parser::ast::FacetScales::Fixed,
            }),
            coord: None,
            theme: None,
            x_scale: None,
            y_scale: None,
        };
        let data = make_data();
        let resolved = resolve_plot_aesthetics(&spec, &data).unwrap();
        assert!(resolved.facet.is_some());
        assert_eq!(resolved.facet.unwrap().col, "g");
    }
}
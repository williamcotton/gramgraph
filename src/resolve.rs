use anyhow::Result;
use crate::parser::ast::{PlotSpec, Layer, Aesthetics, AestheticValue};
use crate::csv_reader::CsvData;
use crate::ir::{ResolvedSpec, ResolvedLayer, ResolvedAesthetics, ResolvedFacet};

/// Resolve all aesthetic mappings for the entire plot
pub fn resolve_plot_aesthetics(spec: &PlotSpec, _csv_data: &CsvData) -> Result<ResolvedSpec> {
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

    // 2. Resolve Layers
    let mut layers = Vec::new();
    for layer in &spec.layers {
        let aesthetics = resolve_layer_aesthetics(layer, &spec.aesthetics)?;
        layers.push(ResolvedLayer {
            original_layer: layer.clone(),
            aesthetics,
        });
    }

    Ok(ResolvedSpec {
        layers,
        facet,
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
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.color.clone()));

    // Resolve size mapping
    let size = match layer {
        Layer::Line(l) => extract_mapped_string_from_f64(&l.width), // width can be data-driven
        Layer::Point(p) => extract_mapped_string_from_f64(&p.size),
        Layer::Bar(b) => extract_mapped_string_from_f64(&b.width),
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.size.clone()));

    // Resolve shape mapping (point only)
    let shape = match layer {
        Layer::Point(p) => extract_mapped_string(&p.shape),
        _ => None,
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.shape.clone()));

    // Resolve alpha mapping
    let alpha = match layer {
        Layer::Line(l) => extract_mapped_string_from_f64(&l.alpha),
        Layer::Point(p) => extract_mapped_string_from_f64(&p.alpha),
        Layer::Bar(b) => extract_mapped_string_from_f64(&b.alpha),
    }
    .or_else(|| global_aes.as_ref().and_then(|a| a.alpha.clone()));

    Ok(ResolvedAesthetics {
        x_col,
        y_col,
        color,
        size,
        shape,
        alpha,
    })
}

/// Resolve x and y aesthetics
fn resolve_positional(layer: &Layer, global_aes: &Option<Aesthetics>) -> Result<(String, String)> {
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
    use crate::parser::ast::{Aesthetics, Layer, LineLayer, PointLayer};
    use crate::csv_reader::CsvData;

    fn make_csv() -> CsvData {
        CsvData {
            headers: vec!["x".to_string(), "y".to_string(), "g".to_string()],
            rows: vec![],
        }
    }

    #[test]
    fn test_resolve_simple() {
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
        let csv = make_csv();
        let resolved = resolve_plot_aesthetics(&spec, &csv).unwrap();
        assert_eq!(resolved.layers.len(), 1);
        assert_eq!(resolved.layers[0].aesthetics.x_col, "x");
        assert_eq!(resolved.layers[0].aesthetics.y_col, "y");
    }

    #[test]
    fn test_resolve_override() {
        let spec = PlotSpec {
            aesthetics: Some(Aesthetics {
                x: "x".to_string(),
                y: "y".to_string(),
                color: None,
                size: None,
                shape: None,
                alpha: None,
            }),
            layers: vec![Layer::Point(PointLayer {
                x: None,
                y: Some("g".to_string()),
                ..Default::default()
            })],
            labels: None,
            facet: None,
        };
        let csv = make_csv();
        let resolved = resolve_plot_aesthetics(&spec, &csv).unwrap();
        assert_eq!(resolved.layers[0].aesthetics.y_col, "g");
    }

    #[test]
    fn test_resolve_missing_aes() {
        let spec = PlotSpec {
            aesthetics: None,
            layers: vec![Layer::Line(LineLayer::default())],
            labels: None,
            facet: None,
        };
        let csv = make_csv();
        let res = resolve_plot_aesthetics(&spec, &csv);
        assert!(res.is_err());
    }

    #[test]
    fn test_resolve_facet() {
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
            facet: Some(crate::parser::ast::Facet {
                by: "g".to_string(),
                ncol: None,
                scales: crate::parser::ast::FacetScales::Fixed,
            }),
        };
        let csv = make_csv();
        let resolved = resolve_plot_aesthetics(&spec, &csv).unwrap();
        assert!(resolved.facet.is_some());
        assert_eq!(resolved.facet.unwrap().col, "g");
    }
}

// Pipeline parser for Grammar of Graphics DSL

use super::aesthetics::parse_aesthetics;
use super::ast::{Aesthetics, AxisScale, CoordSystem, Facet, Labels, Layer, PlotSpec, Theme, ThemeElement};
use super::coord::parse_coord_flip;
use super::facet::parse_facet_wrap;
use super::geom::parse_geom;
use super::labels::parse_labs;
use super::scale::parse_scale_command;
use super::theme::parse_theme_command;
use super::lexer::ws;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{eof, map, opt},
    multi::separated_list0,
    error::{Error, ErrorKind},
    IResult,
};

/// Merge two themes together (ggplot2-style).
/// Fields from `overlay` override `base` unless they are `Inherit`.
fn merge_themes(base: Theme, overlay: Theme) -> Theme {
    Theme {
        line: if overlay.line != ThemeElement::Inherit { overlay.line } else { base.line },
        rect: if overlay.rect != ThemeElement::Inherit { overlay.rect } else { base.rect },
        text: if overlay.text != ThemeElement::Inherit { overlay.text } else { base.text },
        plot_background: if overlay.plot_background != ThemeElement::Inherit { overlay.plot_background } else { base.plot_background },
        plot_title: if overlay.plot_title != ThemeElement::Inherit { overlay.plot_title } else { base.plot_title },
        panel_background: if overlay.panel_background != ThemeElement::Inherit { overlay.panel_background } else { base.panel_background },
        panel_grid_major: if overlay.panel_grid_major != ThemeElement::Inherit { overlay.panel_grid_major } else { base.panel_grid_major },
        panel_grid_minor: if overlay.panel_grid_minor != ThemeElement::Inherit { overlay.panel_grid_minor } else { base.panel_grid_minor },
        axis_text: if overlay.axis_text != ThemeElement::Inherit { overlay.axis_text } else { base.axis_text },
        axis_line: if overlay.axis_line != ThemeElement::Inherit { overlay.axis_line } else { base.axis_line },
        axis_ticks: if overlay.axis_ticks != ThemeElement::Inherit { overlay.axis_ticks } else { base.axis_ticks },
        legend_position: overlay.legend_position.or(base.legend_position),
        legend_background: if overlay.legend_background != ThemeElement::Inherit { overlay.legend_background } else { base.legend_background },
        legend_text: if overlay.legend_text != ThemeElement::Inherit { overlay.legend_text } else { base.legend_text },
        legend_margin: overlay.legend_margin.or(base.legend_margin),
        legend_key_size: overlay.legend_key_size.or(base.legend_key_size),
    }
}

#[derive(Debug)]
enum PipelineComponent {
    Aes(Aesthetics),
    Layer(Layer),
    Facet(Facet),
    Coord(CoordSystem),
    Labels(Labels),
    Theme(Theme),
    Scale(bool, AxisScale), // is_x, scale
}

fn parse_pipeline_component(input: &str) -> IResult<&str, PipelineComponent> {
    alt((
        map(parse_aesthetics, PipelineComponent::Aes),
        map(parse_geom, PipelineComponent::Layer),
        map(parse_facet_wrap, PipelineComponent::Facet),
        map(parse_coord_flip, PipelineComponent::Coord),
        map(parse_labs, PipelineComponent::Labels),
        map(parse_theme_command, PipelineComponent::Theme),
        map(parse_scale_command, |(is_x, s)| PipelineComponent::Scale(is_x, s)),
    ))(input)
}

/// Parse a complete plot specification
/// Format: component | component | ...
pub fn parse_plot_spec(input: &str) -> IResult<&str, PlotSpec> {
    // Optional: consume leading "df"
    let (input, _) = opt(ws(tag("df")))(input)?;

    // If input starts with "|", consume it
    let (input, _) = opt(ws(tag("|")))(input)?;

    // Parse list of components separated by "|"
    let (input, components) = separated_list0(
        ws(tag("|")),
        parse_pipeline_component
    )(input)?;

    // Consume trailing whitespace and ensure end of input
    let (input, _) = ws(eof)(input)?;

    // Aggregate components into PlotSpec
    let mut aesthetics = None;
    let mut layers = Vec::new();
    let mut facet = None;
    let mut coord = None;
    let mut labels = None;
    let mut theme = None;
    let mut x_scale = None;
    let mut y_scale = None;

    for comp in components {
        match comp {
            PipelineComponent::Aes(a) => aesthetics = Some(a),
            PipelineComponent::Layer(l) => layers.push(l),
            PipelineComponent::Facet(f) => facet = Some(f),
            PipelineComponent::Coord(c) => coord = Some(c),
            PipelineComponent::Labels(l) => {
                // Merge or override? Let's override for simplicity, or merge fields if needed.
                // For now, simple override.
                labels = Some(l);
            }
            PipelineComponent::Theme(t) => {
                // Merge themes (ggplot2-style: later values override earlier)
                theme = Some(match theme {
                    Some(base) => merge_themes(base, t),
                    None => t,
                });
            }
            PipelineComponent::Scale(is_x, s) => {
                if is_x { x_scale = Some(s); } else { y_scale = Some(s); }
            }
        }
    }

    // Validation: Must have at least one layer
    if layers.is_empty() {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Verify)));
    }

    Ok((
        input,
        PlotSpec {
            aesthetics,
            layers,
            labels,
            facet,
            coord,
            theme,
            x_scale,
            y_scale,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aes_and_line() {
        let result = parse_plot_spec("aes(x: time, y: temp) | line()");
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.aesthetics.is_some());
        assert_eq!(spec.layers.len(), 1);
    }

    #[test]
    fn test_parse_multiple_layers() {
        let result = parse_plot_spec(r#"aes(x: one, y: two) | line(color: "red") | point(size: 5)"#);
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.aesthetics.is_some());
        assert_eq!(spec.layers.len(), 2);
    }

    #[test]
    fn test_parse_no_aesthetics() {
        // Allow geoms without explicit aes for backward compat / convenience
        let result = parse_plot_spec("line()");
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.aesthetics.is_none());
        assert_eq!(spec.layers.len(), 1);
    }

    #[test]
    fn test_parse_with_df_prefix() {
        let result = parse_plot_spec("df | aes(x: a, y: b) | line()");
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.aesthetics.is_some());
        assert_eq!(spec.layers.len(), 1);
    }

    #[test]
    fn test_parse_plot_spec_trailing_pipe() {
        // Trailing pipe should fail (nothing after last pipe)
        assert!(parse_plot_spec("aes(x: a, y: b) | line() |").is_err());
    }

    #[test]
    fn test_parse_plot_spec_missing_geom() {
        // Aesthetics without any geometry should fail (needs at least one geom)
        assert!(parse_plot_spec("aes(x: a, y: b)").is_err());
    }

    #[test]
    fn test_parse_plot_spec_empty_input() {
        // Empty input should fail
        assert!(parse_plot_spec("").is_err());
    }

    #[test]
    fn test_parse_plot_spec_three_layers() {
        // Three layers: line + point + bar
        let result = parse_plot_spec(r#"aes(x: a, y: b) | line() | point() | bar()"#);
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert_eq!(spec.layers.len(), 3);
    }

    #[test]
    fn test_parse_plot_spec_df_without_aes() {
        // df prefix without aesthetics should succeed
        let result = parse_plot_spec("df | line()");
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.aesthetics.is_none());
        assert_eq!(spec.layers.len(), 1);
    }

    #[test]
    fn test_parse_plot_spec_with_facet_wrap() {
        let result = parse_plot_spec("aes(x: time, y: sales) | line() | facet_wrap(by: region)");
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.facet.is_some());
        let facet = spec.facet.unwrap();
        assert_eq!(facet.by, "region");
    }

    #[test]
    fn test_parse_plot_spec_with_facet_wrap_full() {
        let result = parse_plot_spec(r#"aes(x: time, y: sales) | line() | facet_wrap(by: region, ncol: 2, scales: "free_x")"#);
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.facet.is_some());
        let facet = spec.facet.unwrap();
        assert_eq!(facet.by, "region");
        assert_eq!(facet.ncol, Some(2));
    }

    #[test]
    fn test_parse_plot_spec_without_facet() {
        let result = parse_plot_spec("aes(x: time, y: sales) | line()");
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.facet.is_none());
    }

    #[test]
    fn test_parse_plot_spec_with_labs_and_theme() {
        let result = parse_plot_spec(r#"aes(x: x, y: y) | line() | labs(title: "My Plot", x: "Time") | theme(legend_position: "none")"#);
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert_eq!(spec.labels.as_ref().unwrap().title, Some("My Plot".to_string()));
        assert_eq!(spec.labels.as_ref().unwrap().x, Some("Time".to_string()));
        assert_eq!(spec.theme.as_ref().unwrap().legend_position, Some(crate::parser::ast::LegendPosition::None));
    }

    #[test]
    fn test_parse_histogram_pipeline() {
        let input = r#"aes(x: value) | histogram(bins: 5) | labs(title: "Distribution", x: "Value", y: "Count") | theme_minimal()"#;
        let result = parse_plot_spec(input);
        match &result {
            Ok(_) => println!("Parsed successfully"),
            Err(e) => println!("Parse error: {:?}", e),
        }
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert!(spec.aesthetics.is_some());
        assert_eq!(spec.layers.len(), 1);
        if let crate::parser::ast::Layer::Bar(b) = &spec.layers[0] {
             match b.stat {
                 crate::parser::ast::Stat::Bin { bins } => assert_eq!(bins, 5),
                 _ => panic!("Expected Bin stat"),
             }
        } else {
            panic!("Expected Bar layer (histogram)");
        }
    }
}

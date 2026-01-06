// Pipeline parser for Grammar of Graphics DSL

use super::aesthetics::parse_aesthetics;
use super::ast::PlotSpec;
use super::facet::parse_facet_wrap;
use super::geom::parse_geom;
use super::lexer::ws;
use nom::{
    bytes::complete::tag,
    combinator::{eof, opt},
    multi::many0,
    IResult,
};

/// Parse a complete plot specification
/// Format: [aes(...) |] geom() | geom() | ...
pub fn parse_plot_spec(input: &str) -> IResult<&str, PlotSpec> {
    // Optional: consume leading "df"
    let (input, _) = opt(ws(tag("df")))(input)?;

    // If input starts with "|", consume it
    let (input, _) = opt(ws(tag("|")))(input)?;

    // Try to parse aesthetics (optional but recommended)
    let (input, aesthetics) = opt(parse_aesthetics)(input)?;

    // If we parsed aesthetics, consume the pipe separator
    let (input, _) = if aesthetics.is_some() {
        let (input, _) = ws(tag("|"))(input)?;
        (input, ())
    } else {
        (input, ())
    };

    // Parse first geometry (required)
    let (input, first_geom) = parse_geom(input)?;

    // Parse additional geometries
    let (input, mut remaining_geoms) = many0(|input| {
        let (input, _) = ws(tag("|"))(input)?;
        parse_geom(input)
    })(input)?;

    // Parse optional facet_wrap at the end
    let (input, facet) = opt(|input| {
        let (input, _) = ws(tag("|"))(input)?;
        parse_facet_wrap(input)
    })(input)?;

    // Consume trailing whitespace and ensure end of input
    let (input, _) = ws(eof)(input)?;

    // Build layers vec
    let mut layers = vec![first_geom];
    layers.append(&mut remaining_geoms);

    Ok((
        input,
        PlotSpec {
            aesthetics,
            layers,
            labels: None, // Future: parse labs() command
            facet,
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
        let result = parse_plot_spec(r#"aes(x: time, y: sales) | line() | facet_wrap(by: region, ncol: Some(2), scales: "free_x")"#);
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
}

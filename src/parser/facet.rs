// Facet parser for facet_wrap() syntax

use super::ast::{Facet, FacetScales};
use super::lexer::{identifier, ws};
use nom::{
    bytes::complete::tag,
    character::complete::char,
    multi::separated_list0,
    IResult,
};

/// Parse facet_wrap specification
/// Format: facet_wrap(by: column_name, ncol: 2, scales: "free_x")
/// - by: required (column name to facet by)
/// - ncol: optional (number of columns in grid)
/// - scales: optional (axis sharing mode: "fixed", "free_x", "free_y", "free")
pub fn parse_facet_wrap(input: &str) -> IResult<&str, Facet> {
    // Parse function name
    let (input, _) = ws(tag("facet_wrap"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    // Parse named arguments
    let (input, args) = separated_list0(
        ws(char(',')),
        parse_facet_argument
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    // Extract arguments
    let mut by = None;
    let mut ncol = None;
    let mut scales = FacetScales::default();

    for (key, value) in args {
        match key.as_str() {
            "by" => by = Some(value.column),
            "ncol" => ncol = value.ncol,
            "scales" => scales = value.scales.unwrap_or_default(),
            _ => {}
        }
    }

    // Validate: "by" is required
    let by = by.ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))
    })?;

    Ok((input, Facet { by, ncol, scales }))
}

/// Parse a single facet argument (key: value pair)
fn parse_facet_argument(input: &str) -> IResult<&str, (String, FacetArgValue)> {
    let (input, key) = ws(identifier)(input)?;
    let (input, _) = ws(char(':'))(input)?;

    let value = match key.as_str() {
        "by" => {
            let (input, col) = ws(identifier)(input)?;
            (input, FacetArgValue::column(col))
        }
        "ncol" => {
            let (input, _) = ws(tag("Some"))(input)?;
            let (input, _) = ws(char('('))(input)?;
            let (input, n) = nom::character::complete::u32(input)?;
            let (input, _) = ws(char(')'))(input)?;
            (input, FacetArgValue::ncol(n as usize))
        }
        "scales" => {
            let (input, _) = ws(char('"'))(input)?;
            let (input, scale_str) = nom::bytes::complete::take_while(|c: char| c != '"')(input)?;
            let (input, _) = ws(char('"'))(input)?;
            let scales = match scale_str {
                "free_x" => FacetScales::FreeX,
                "free_y" => FacetScales::FreeY,
                "free" => FacetScales::Free,
                "fixed" => FacetScales::Fixed,
                _ => FacetScales::Fixed,
            };
            (input, FacetArgValue::scales(scales))
        }
        _ => {
            // Unknown argument, skip it
            let (input, col) = ws(identifier)(input)?;
            (input, FacetArgValue::column(col))
        }
    };

    Ok((value.0, (key, value.1)))
}

/// Intermediate representation for facet argument values
#[derive(Debug)]
struct FacetArgValue {
    column: String,
    ncol: Option<usize>,
    scales: Option<FacetScales>,
}

impl FacetArgValue {
    fn column(s: String) -> Self {
        Self {
            column: s,
            ncol: None,
            scales: None,
        }
    }

    fn ncol(n: usize) -> Self {
        Self {
            column: String::new(),
            ncol: Some(n),
            scales: None,
        }
    }

    fn scales(s: FacetScales) -> Self {
        Self {
            column: String::new(),
            ncol: None,
            scales: Some(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_facet_wrap_by_only() {
        let result = parse_facet_wrap("facet_wrap(by: region)");
        assert!(result.is_ok());
        let (_, facet) = result.unwrap();
        assert_eq!(facet.by, "region");
        assert_eq!(facet.ncol, None);
        assert_eq!(facet.scales, FacetScales::Fixed);
    }

    #[test]
    fn test_parse_facet_wrap_with_ncol() {
        let result = parse_facet_wrap("facet_wrap(by: region, ncol: Some(2))");
        assert!(result.is_ok());
        let (_, facet) = result.unwrap();
        assert_eq!(facet.by, "region");
        assert_eq!(facet.ncol, Some(2));
    }

    #[test]
    fn test_parse_facet_wrap_with_scales_free_x() {
        let result = parse_facet_wrap(r#"facet_wrap(by: region, scales: "free_x")"#);
        assert!(result.is_ok());
        let (_, facet) = result.unwrap();
        assert_eq!(facet.by, "region");
        assert_eq!(facet.scales, FacetScales::FreeX);
    }

    #[test]
    fn test_parse_facet_wrap_with_scales_free_y() {
        let result = parse_facet_wrap(r#"facet_wrap(by: product, scales: "free_y")"#);
        assert!(result.is_ok());
        let (_, facet) = result.unwrap();
        assert_eq!(facet.scales, FacetScales::FreeY);
    }

    #[test]
    fn test_parse_facet_wrap_with_scales_free() {
        let result = parse_facet_wrap(r#"facet_wrap(by: category, scales: "free")"#);
        assert!(result.is_ok());
        let (_, facet) = result.unwrap();
        assert_eq!(facet.scales, FacetScales::Free);
    }

    #[test]
    fn test_parse_facet_wrap_all_args() {
        let result = parse_facet_wrap(r#"facet_wrap(by: region, ncol: Some(3), scales: "free_x")"#);
        assert!(result.is_ok());
        let (_, facet) = result.unwrap();
        assert_eq!(facet.by, "region");
        assert_eq!(facet.ncol, Some(3));
        assert_eq!(facet.scales, FacetScales::FreeX);
    }

    #[test]
    fn test_parse_facet_wrap_missing_by() {
        // Missing required "by" argument should fail
        let result = parse_facet_wrap(r#"facet_wrap(ncol: Some(2))"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_facet_wrap_with_whitespace() {
        let result = parse_facet_wrap(r#"facet_wrap( by : region , ncol : Some(2) )"#);
        assert!(result.is_ok());
        let (_, facet) = result.unwrap();
        assert_eq!(facet.by, "region");
        assert_eq!(facet.ncol, Some(2));
    }
}

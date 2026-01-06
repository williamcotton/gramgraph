// Aesthetics parser for Grammar of Graphics DSL

use super::ast::Aesthetics;
use super::lexer::{identifier, ws};
use nom::{
    bytes::complete::tag,
    character::complete::char,
    multi::separated_list0,
    IResult,
};

/// Parse aesthetics specification
/// Format: aes(x: col, y: col[, color: col2][, size: col3][, shape: col4][, alpha: col5])
pub fn parse_aesthetics(input: &str) -> IResult<&str, Aesthetics> {
    let (input, _) = ws(tag("aes"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    // Parse named arguments (key: value pairs)
    let (input, args) = separated_list0(
        ws(char(',')),
        parse_aesthetic_argument
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    // Extract arguments
    let mut x = None;
    let mut y = None;
    let mut color = None;
    let mut size = None;
    let mut shape = None;
    let mut alpha = None;

    for (key, value) in args {
        match key.as_str() {
            "x" => x = Some(value),
            "y" => y = Some(value),
            "color" => color = Some(value),
            "size" => size = Some(value),
            "shape" => shape = Some(value),
            "alpha" => alpha = Some(value),
            _ => {} // Ignore unknown keys
        }
    }

    // Validate required fields
    let x = x.ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))
    })?;
    let y = y.ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))
    })?;

    Ok((input, Aesthetics { x, y, color, size, shape, alpha }))
}

/// Parse a single aesthetic argument (key: value)
fn parse_aesthetic_argument(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = ws(identifier)(input)?;
    let (input, _) = ws(char(':'))(input)?;
    let (input, value) = ws(identifier)(input)?;
    Ok((input, (key, value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aesthetics() {
        let result = parse_aesthetics("aes(x: time, y: temp)");
        assert!(result.is_ok());
        let (_, aes) = result.unwrap();
        assert_eq!(aes.x, "time");
        assert_eq!(aes.y, "temp");
    }

    #[test]
    fn test_parse_aesthetics_with_whitespace() {
        let result = parse_aesthetics("  aes( x: time , y: temp )  ");
        assert!(result.is_ok());
        let (_, aes) = result.unwrap();
        assert_eq!(aes.x, "time");
        assert_eq!(aes.y, "temp");
    }

    #[test]
    fn test_parse_aesthetics_missing_x() {
        // Missing x parameter should fail
        assert!(parse_aesthetics("aes(y: temp)").is_err());
    }

    #[test]
    fn test_parse_aesthetics_missing_comma() {
        // Missing comma between x and y should fail
        assert!(parse_aesthetics("aes(x: time y: temp)").is_err());
    }

    #[test]
    fn test_parse_aesthetics_extra_comma() {
        // Extra comma should fail
        assert!(parse_aesthetics("aes(x: time,, y: temp)").is_err());
    }

    #[test]
    fn test_parse_aesthetics_unclosed_paren() {
        // Unclosed parenthesis should fail
        assert!(parse_aesthetics("aes(x: time, y: temp").is_err());
    }
}

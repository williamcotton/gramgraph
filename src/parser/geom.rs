// Geometry (geom) parser for Grammar of Graphics DSL

use super::ast::{AestheticValue, BarLayer, BarPosition, Layer, LineLayer, PointLayer};
use super::lexer::{identifier, number_literal, string_literal, ws};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::map,
    multi::separated_list0,
    sequence::preceded,
    IResult,
};

/// Argument value type for geometry parsers
enum ArgValue {
    ColumnName(String),        // x, y aesthetic overrides
    ColorFixed(String),        // color: "red" (literal)
    ColorMapped(String),       // color: region (column)
    NumericFixed(f64),         // width: 2, alpha: 0.5
    NumericMapped(String),     // width: size_col, alpha: alpha_col
}

/// Parse a line geometry
/// Format: line() or line(color: "red", width: 2, ...) or line(color: region)
pub fn parse_line(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("line"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    // Parse optional named arguments
    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            map(
                preceded(ws(tag("x:")), ws(identifier)),
                |x| ("x", ArgValue::ColumnName(x)),
            ),
            map(
                preceded(ws(tag("y:")), ws(identifier)),
                |y| ("y", ArgValue::ColumnName(y)),
            ),
            // color: can be "red" (literal) or region (identifier/column)
            map(
                preceded(ws(tag("color:")), ws(string_literal)),
                |c| ("color", ArgValue::ColorFixed(c)),
            ),
            map(
                preceded(ws(tag("color:")), ws(identifier)),
                |c| ("color", ArgValue::ColorMapped(c)),
            ),
            // width: can be 2.0 (literal) or width_col (identifier/column)
            map(
                preceded(ws(tag("width:")), ws(number_literal)),
                |w| ("width", ArgValue::NumericFixed(w)),
            ),
            map(
                preceded(ws(tag("width:")), ws(identifier)),
                |w| ("width", ArgValue::NumericMapped(w)),
            ),
            // alpha: can be 0.5 (literal) or alpha_col (identifier/column)
            map(
                preceded(ws(tag("alpha:")), ws(number_literal)),
                |a| ("alpha", ArgValue::NumericFixed(a)),
            ),
            map(
                preceded(ws(tag("alpha:")), ws(identifier)),
                |a| ("alpha", ArgValue::NumericMapped(a)),
            ),
        )),
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = LineLayer::default();

    for (key, val) in args {
        match (key, val) {
            ("x", ArgValue::ColumnName(x)) => layer.x = Some(x),
            ("y", ArgValue::ColumnName(y)) => layer.y = Some(y),
            ("color", ArgValue::ColorFixed(c)) => layer.color = Some(AestheticValue::Fixed(c)),
            ("color", ArgValue::ColorMapped(c)) => layer.color = Some(AestheticValue::Mapped(c)),
            ("width", ArgValue::NumericFixed(w)) => layer.width = Some(AestheticValue::Fixed(w)),
            ("width", ArgValue::NumericMapped(w)) => layer.width = Some(AestheticValue::Mapped(w)),
            ("alpha", ArgValue::NumericFixed(a)) => layer.alpha = Some(AestheticValue::Fixed(a)),
            ("alpha", ArgValue::NumericMapped(a)) => layer.alpha = Some(AestheticValue::Mapped(a)),
            _ => {}
        }
    }

    Ok((input, Layer::Line(layer)))
}

/// Parse a point geometry
/// Format: point() or point(size: 5, color: "blue", ...) or point(color: region, size: metric)
pub fn parse_point(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("point"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    // Parse optional named arguments
    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            map(
                preceded(ws(tag("x:")), ws(identifier)),
                |x| ("x", ArgValue::ColumnName(x)),
            ),
            map(
                preceded(ws(tag("y:")), ws(identifier)),
                |y| ("y", ArgValue::ColumnName(y)),
            ),
            // color: can be "blue" (literal) or region (column)
            map(
                preceded(ws(tag("color:")), ws(string_literal)),
                |c| ("color", ArgValue::ColorFixed(c)),
            ),
            map(
                preceded(ws(tag("color:")), ws(identifier)),
                |c| ("color", ArgValue::ColorMapped(c)),
            ),
            // size: can be 5.0 (literal) or size_col (column)
            map(
                preceded(ws(tag("size:")), ws(number_literal)),
                |s| ("size", ArgValue::NumericFixed(s)),
            ),
            map(
                preceded(ws(tag("size:")), ws(identifier)),
                |s| ("size", ArgValue::NumericMapped(s)),
            ),
            // shape: can be "circle" (literal) or shape_col (column)
            map(
                preceded(ws(tag("shape:")), ws(string_literal)),
                |sh| ("shape", ArgValue::ColorFixed(sh)),
            ),
            map(
                preceded(ws(tag("shape:")), ws(identifier)),
                |sh| ("shape", ArgValue::ColorMapped(sh)),
            ),
            // alpha: can be 0.8 (literal) or alpha_col (column)
            map(
                preceded(ws(tag("alpha:")), ws(number_literal)),
                |a| ("alpha", ArgValue::NumericFixed(a)),
            ),
            map(
                preceded(ws(tag("alpha:")), ws(identifier)),
                |a| ("alpha", ArgValue::NumericMapped(a)),
            ),
        )),
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = PointLayer::default();

    for (key, val) in args {
        match (key, val) {
            ("x", ArgValue::ColumnName(x)) => layer.x = Some(x),
            ("y", ArgValue::ColumnName(y)) => layer.y = Some(y),
            ("color", ArgValue::ColorFixed(c)) => layer.color = Some(AestheticValue::Fixed(c)),
            ("color", ArgValue::ColorMapped(c)) => layer.color = Some(AestheticValue::Mapped(c)),
            ("size", ArgValue::NumericFixed(s)) => layer.size = Some(AestheticValue::Fixed(s)),
            ("size", ArgValue::NumericMapped(s)) => layer.size = Some(AestheticValue::Mapped(s)),
            ("shape", ArgValue::ColorFixed(sh)) => layer.shape = Some(AestheticValue::Fixed(sh)),
            ("shape", ArgValue::ColorMapped(sh)) => layer.shape = Some(AestheticValue::Mapped(sh)),
            ("alpha", ArgValue::NumericFixed(a)) => layer.alpha = Some(AestheticValue::Fixed(a)),
            ("alpha", ArgValue::NumericMapped(a)) => layer.alpha = Some(AestheticValue::Mapped(a)),
            _ => {}
        }
    }

    Ok((input, Layer::Point(layer)))
}

/// Argument value for position (special handling)
enum PositionArg {
    Position(String),  // position: "dodge"/"stack"/"identity"
}

/// Parse a bar geometry
/// Format: bar() or bar(color: "red", position: "dodge", ...) or bar(color: region)
pub fn parse_bar(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("bar"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    // Parse optional named arguments
    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            map(
                preceded(ws(tag("x:")), ws(identifier)),
                |x| ("x", ArgValue::ColumnName(x)),
            ),
            map(
                preceded(ws(tag("y:")), ws(identifier)),
                |y| ("y", ArgValue::ColumnName(y)),
            ),
            // color: can be "red" (literal) or region (column)
            map(
                preceded(ws(tag("color:")), ws(string_literal)),
                |c| ("color", ArgValue::ColorFixed(c)),
            ),
            map(
                preceded(ws(tag("color:")), ws(identifier)),
                |c| ("color", ArgValue::ColorMapped(c)),
            ),
            // width: can be 0.8 (literal) or width_col (column)
            map(
                preceded(ws(tag("width:")), ws(number_literal)),
                |w| ("width", ArgValue::NumericFixed(w)),
            ),
            map(
                preceded(ws(tag("width:")), ws(identifier)),
                |w| ("width", ArgValue::NumericMapped(w)),
            ),
            // alpha: can be 0.7 (literal) or alpha_col (column)
            map(
                preceded(ws(tag("alpha:")), ws(number_literal)),
                |a| ("alpha", ArgValue::NumericFixed(a)),
            ),
            map(
                preceded(ws(tag("alpha:")), ws(identifier)),
                |a| ("alpha", ArgValue::NumericMapped(a)),
            ),
            // position: always a string literal
            map(
                preceded(ws(tag("position:")), ws(string_literal)),
                |p| ("position", ArgValue::ColorFixed(p)),
            ),
        )),
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = BarLayer::default();

    for (key, val) in args {
        match (key, val) {
            ("x", ArgValue::ColumnName(x)) => layer.x = Some(x),
            ("y", ArgValue::ColumnName(y)) => layer.y = Some(y),
            ("color", ArgValue::ColorFixed(c)) => layer.color = Some(AestheticValue::Fixed(c)),
            ("color", ArgValue::ColorMapped(c)) => layer.color = Some(AestheticValue::Mapped(c)),
            ("width", ArgValue::NumericFixed(w)) => layer.width = Some(AestheticValue::Fixed(w)),
            ("width", ArgValue::NumericMapped(w)) => layer.width = Some(AestheticValue::Mapped(w)),
            ("alpha", ArgValue::NumericFixed(a)) => layer.alpha = Some(AestheticValue::Fixed(a)),
            ("alpha", ArgValue::NumericMapped(a)) => layer.alpha = Some(AestheticValue::Mapped(a)),
            ("position", ArgValue::ColorFixed(p)) => {
                layer.position = match p.as_str() {
                    "dodge" => BarPosition::Dodge,
                    "stack" => BarPosition::Stack,
                    "identity" => BarPosition::Identity,
                    _ => BarPosition::Identity, // default for unknown values
                };
            }
            _ => {}
        }
    }

    Ok((input, Layer::Bar(layer)))
}

/// Parse any geometry layer
pub fn parse_geom(input: &str) -> IResult<&str, Layer> {
    alt((parse_line, parse_point, parse_bar))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line_empty() {
        let result = parse_line("line()");
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Line(l) => {
                assert_eq!(l.color, None);
                assert_eq!(l.width, None);
            }
            _ => panic!("Expected Line layer"),
        }
    }

    #[test]
    fn test_parse_line_with_color() {
        let result = parse_line(r#"line(color: "red")"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Line(l) => {
                assert_eq!(l.color, Some(AestheticValue::Fixed("red".to_string())));
            }
            _ => panic!("Expected Line layer"),
        }
    }

    #[test]
    fn test_parse_point_with_size() {
        let result = parse_point("point(size: 5)");
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Point(p) => {
                assert_eq!(p.size, Some(AestheticValue::Fixed(5.0)));
            }
            _ => panic!("Expected Point layer"),
        }
    }

    #[test]
    fn test_parse_bar_empty() {
        let result = parse_bar("bar()");
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Bar(b) => {
                assert_eq!(b.color, None);
                assert_eq!(b.alpha, None);
                assert_eq!(b.position, BarPosition::Identity);
            }
            _ => panic!("Expected Bar layer"),
        }
    }

    #[test]
    fn test_parse_bar_with_position() {
        let result = parse_bar(r#"bar(position: "dodge")"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Bar(b) => {
                assert_eq!(b.position, BarPosition::Dodge);
            }
            _ => panic!("Expected Bar layer"),
        }
    }

    #[test]
    fn test_parse_bar_with_stack_position() {
        let result = parse_bar(r#"bar(position: "stack")"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Bar(b) => {
                assert_eq!(b.position, BarPosition::Stack);
            }
            _ => panic!("Expected Bar layer"),
        }
    }

    #[test]
    fn test_parse_bar_with_color() {
        let result = parse_bar(r#"bar(color: "red")"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Bar(b) => {
                assert_eq!(b.color, Some(AestheticValue::Fixed("red".to_string())));
            }
            _ => panic!("Expected Bar layer"),
        }
    }

    #[test]
    fn test_parse_bar_full() {
        let result = parse_bar(r#"bar(position: "stack", color: "blue", alpha: 0.7, width: 0.6)"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Bar(b) => {
                assert_eq!(b.position, BarPosition::Stack);
                assert_eq!(b.color, Some(AestheticValue::Fixed("blue".to_string())));
                assert_eq!(b.alpha, Some(AestheticValue::Fixed(0.7)));
                assert_eq!(b.width, Some(AestheticValue::Fixed(0.6)));
            }
            _ => panic!("Expected Bar layer"),
        }
    }

    #[test]
    fn test_parse_line_all_params() {
        // Test line with all parameters: x, y, color, width, alpha
        let result = parse_line(r#"line(x: col1, y: col2, color: "red", width: 2, alpha: 0.5)"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Line(l) => {
                assert_eq!(l.x, Some("col1".to_string()));
                assert_eq!(l.y, Some("col2".to_string()));
                assert_eq!(l.color, Some(AestheticValue::Fixed("red".to_string())));
                assert_eq!(l.width, Some(AestheticValue::Fixed(2.0)));
                assert_eq!(l.alpha, Some(AestheticValue::Fixed(0.5)));
            }
            _ => panic!("Expected Line layer"),
        }
    }

    #[test]
    fn test_parse_point_all_params() {
        // Test point with all parameters: x, y, color, size, alpha
        let result = parse_point(r#"point(x: col1, y: col2, color: "blue", size: 10, alpha: 0.8)"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Point(p) => {
                assert_eq!(p.x, Some("col1".to_string()));
                assert_eq!(p.y, Some("col2".to_string()));
                assert_eq!(p.color, Some(AestheticValue::Fixed("blue".to_string())));
                assert_eq!(p.size, Some(AestheticValue::Fixed(10.0)));
                assert_eq!(p.alpha, Some(AestheticValue::Fixed(0.8)));
            }
            _ => panic!("Expected Point layer"),
        }
    }

    #[test]
    fn test_parse_geom_whitespace_variations() {
        // Extra spaces around parentheses and commas should be handled
        let result = parse_line(r#"  line ( color: "red" , width: 2 )  "#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Line(l) => {
                assert_eq!(l.color, Some(AestheticValue::Fixed("red".to_string())));
                assert_eq!(l.width, Some(AestheticValue::Fixed(2.0)));
            }
            _ => panic!("Expected Line layer"),
        }
    }

    #[test]
    fn test_parse_bar_invalid_position() {
        // Unknown position value defaults to identity
        let result = parse_bar(r#"bar(position: "unknown")"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Bar(b) => {
                assert_eq!(b.position, BarPosition::Identity); // Should default
            }
            _ => panic!("Expected Bar layer"),
        }
    }

    #[test]
    fn test_parse_geom_multiple_params() {
        // Test that multiple parameters work correctly
        let result = parse_line(r#"line(color: "red", width: 2)"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Line(l) => {
                assert_eq!(l.color, Some(AestheticValue::Fixed("red".to_string())));
                assert_eq!(l.width, Some(AestheticValue::Fixed(2.0)));
            }
            _ => panic!("Expected Line layer"),
        }
    }
}

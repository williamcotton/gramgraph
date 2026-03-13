// Geometry (geom) parser for Grammar of Graphics DSL

use super::ast::{AestheticValue, BarLayer, BarPosition, BoxplotLayer, DensityLayer, HeatmapLayer, Layer, LineLayer, PointLayer, RibbonLayer, ViolinLayer};
use super::lexer::{identifier, number_literal, string_literal, ws};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
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
    NumberArray(Vec<f64>),     // draw_quantiles: [0.25, 0.5, 0.75]
}

/// Parse a number array like [0.25, 0.5, 0.75]
fn parse_number_array(input: &str) -> IResult<&str, Vec<f64>> {
    let (input, _) = ws(char('['))(input)?;
    let (input, nums) = separated_list0(ws(char(',')), ws(number_literal))(input)?;
    let (input, _) = ws(char(']'))(input)?;
    Ok((input, nums))
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
            // x: can be column
            map(
                preceded(ws(tag("x:")), ws(identifier)),
                |x| ("x", ArgValue::ColumnName(x)),
            ),
            // y: can be column
            map(
                preceded(ws(tag("y:")), ws(identifier)),
                |y| ("y", ArgValue::ColumnName(y)),
            ),
            // color: can be "red" (literal), region (column)
            map(
                preceded(ws(tag("color:")), ws(string_literal)),
                |c| ("color", ArgValue::ColorFixed(c)),
            ),
            map(
                preceded(ws(tag("color:")), ws(identifier)),
                |c| ("color", ArgValue::ColorMapped(c)),
            ),
            // width: can be 2.0 (literal), width_col (column)
            map(
                preceded(ws(tag("width:")), ws(number_literal)),
                |w| ("width", ArgValue::NumericFixed(w)),
            ),
            map(
                preceded(ws(tag("width:")), ws(identifier)),
                |w| ("width", ArgValue::NumericMapped(w)),
            ),
            // alpha: can be 0.5 (literal), alpha_col (column)
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
            // x: can be column
            map(
                preceded(ws(tag("x:")), ws(identifier)),
                |x| ("x", ArgValue::ColumnName(x)),
            ),
            // y: can be column
            map(
                preceded(ws(tag("y:")), ws(identifier)),
                |y| ("y", ArgValue::ColumnName(y)),
            ),
            // color: can be "blue" (literal), region (column)
            map(
                preceded(ws(tag("color:")), ws(string_literal)),
                |c| ("color", ArgValue::ColorFixed(c)),
            ),
            map(
                preceded(ws(tag("color:")), ws(identifier)),
                |c| ("color", ArgValue::ColorMapped(c)),
            ),
            // size: can be 5.0 (literal), size_col (column)
            map(
                preceded(ws(tag("size:")), ws(number_literal)),
                |s| ("size", ArgValue::NumericFixed(s)),
            ),
            map(
                preceded(ws(tag("size:")), ws(identifier)),
                |s| ("size", ArgValue::NumericMapped(s)),
            ),
            // shape: can be "circle" (literal), shape_col (column)
            map(
                preceded(ws(tag("shape:")), ws(string_literal)),
                |sh| ("shape", ArgValue::ColorFixed(sh)),
            ),
            map(
                preceded(ws(tag("shape:")), ws(identifier)),
                |sh| ("shape", ArgValue::ColorMapped(sh)),
            ),
            // alpha: can be 0.8 (literal), alpha_col (column)
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

/// Parse a bar geometry
/// Format: bar() or bar(color: "red", position: "dodge", ...) or bar(color: region)
pub fn parse_bar(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("bar"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    // Parse optional named arguments
    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            // x: can be column
            map(
                preceded(ws(tag("x:")), ws(identifier)),
                |x| ("x", ArgValue::ColumnName(x)),
            ),
            // y: can be column
            map(
                preceded(ws(tag("y:")), ws(identifier)),
                |y| ("y", ArgValue::ColumnName(y)),
            ),
            // color: can be "red" (literal), region (column)
            map(
                preceded(ws(tag("color:")), ws(string_literal)),
                |c| ("color", ArgValue::ColorFixed(c)),
            ),
            map(
                preceded(ws(tag("color:")), ws(identifier)),
                |c| ("color", ArgValue::ColorMapped(c)),
            ),
            // width: can be 0.8 (literal), width_col (column)
            map(
                preceded(ws(tag("width:")), ws(number_literal)),
                |w| ("width", ArgValue::NumericFixed(w)),
            ),
            map(
                preceded(ws(tag("width:")), ws(identifier)),
                |w| ("width", ArgValue::NumericMapped(w)),
            ),
            // alpha: can be 0.7 (literal), alpha_col (column)
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

/// Parse a ribbon geometry
pub fn parse_ribbon(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("ribbon"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            // x: can be column
            map(preceded(ws(tag("x:")), ws(identifier)), |x| ("x", ArgValue::ColumnName(x))),
            // ymin: can be column
            map(preceded(ws(tag("ymin:")), ws(identifier)), |y| ("ymin", ArgValue::ColumnName(y))),
            // ymax: can be column
            map(preceded(ws(tag("ymax:")), ws(identifier)), |y| ("ymax", ArgValue::ColumnName(y))),

            // color: can be "literal", column
            map(preceded(ws(tag("color:")), ws(string_literal)), |c| ("color", ArgValue::ColorFixed(c))),
            map(preceded(ws(tag("color:")), ws(identifier)), |c| ("color", ArgValue::ColorMapped(c))),

            // alpha: can be number, column
            map(preceded(ws(tag("alpha:")), ws(number_literal)), |a| ("alpha", ArgValue::NumericFixed(a))),
            map(preceded(ws(tag("alpha:")), ws(identifier)), |a| ("alpha", ArgValue::NumericMapped(a))),
        ))
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = RibbonLayer::default();

    for (key, val) in args {
        match (key, val) {
            ("x", ArgValue::ColumnName(x)) => layer.x = Some(x),
            ("ymin", ArgValue::ColumnName(y)) => layer.ymin = Some(y),
            ("ymax", ArgValue::ColumnName(y)) => layer.ymax = Some(y),
            ("color", ArgValue::ColorFixed(c)) => layer.color = Some(AestheticValue::Fixed(c)),
            ("color", ArgValue::ColorMapped(c)) => layer.color = Some(AestheticValue::Mapped(c)),
            ("alpha", ArgValue::NumericFixed(a)) => layer.alpha = Some(AestheticValue::Fixed(a)),
            ("alpha", ArgValue::NumericMapped(a)) => layer.alpha = Some(AestheticValue::Mapped(a)),
            _ => {}
        }
    }

    Ok((input, Layer::Ribbon(layer)))
}

/// Parse a histogram geometry (sugar for bar(stat: "bin"))
pub fn parse_histogram(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("histogram"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    // Optional bins argument
    let (input, bins) = opt(preceded(ws(tag("bins:")), ws(number_literal)))(input)?;
    let (input, _) = ws(char(')'))(input)?;

    let mut layer = BarLayer::default();
    layer.stat = crate::parser::ast::Stat::Bin { bins: bins.unwrap_or(30.0) as usize };
    Ok((input, Layer::Bar(layer)))
}

/// Parse a smooth geometry (sugar for line(stat: "smooth"))
pub fn parse_smooth(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("smooth"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, _) = ws(char(')'))(input)?;

    let mut layer = LineLayer::default();
    layer.stat = crate::parser::ast::Stat::Smooth { method: "lm".to_string() };
    Ok((input, Layer::Line(layer)))
}

/// Parse a boxplot geometry
pub fn parse_boxplot(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("boxplot"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            // x: can be column
            map(preceded(ws(tag("x:")), ws(identifier)), |x| ("x", ArgValue::ColumnName(x))),
            // y: can be column
            map(preceded(ws(tag("y:")), ws(identifier)), |y| ("y", ArgValue::ColumnName(y))),

            // color: can be "literal", column
            map(preceded(ws(tag("color:")), ws(string_literal)), |c| ("color", ArgValue::ColorFixed(c))),
            map(preceded(ws(tag("color:")), ws(identifier)), |c| ("color", ArgValue::ColorMapped(c))),

            // width: can be number, column
            map(preceded(ws(tag("width:")), ws(number_literal)), |w| ("width", ArgValue::NumericFixed(w))),
            map(preceded(ws(tag("width:")), ws(identifier)), |w| ("width", ArgValue::NumericMapped(w))),

            // alpha: can be number, column
            map(preceded(ws(tag("alpha:")), ws(number_literal)), |a| ("alpha", ArgValue::NumericFixed(a))),
            map(preceded(ws(tag("alpha:")), ws(identifier)), |a| ("alpha", ArgValue::NumericMapped(a))),

            // Outlier specific args (keep as fixed for simplicity)
            map(preceded(ws(tag("outlier_color:")), ws(string_literal)), |c| ("outlier_color", ArgValue::ColorFixed(c))),
            map(preceded(ws(tag("outlier_size:")), ws(number_literal)), |s| ("outlier_size", ArgValue::NumericFixed(s))),
            map(preceded(ws(tag("outlier_shape:")), ws(string_literal)), |sh| ("outlier_shape", ArgValue::ColorFixed(sh))),
        ))
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = BoxplotLayer::default();
    layer.stat = crate::parser::ast::Stat::Boxplot;

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
            ("outlier_color", ArgValue::ColorFixed(c)) => layer.outlier_color = Some(c),
            ("outlier_size", ArgValue::NumericFixed(s)) => layer.outlier_size = Some(s),
            ("outlier_shape", ArgValue::ColorFixed(sh)) => layer.outlier_shape = Some(sh),
            _ => {}
        }
    }

    Ok((input, Layer::Boxplot(layer)))
}

/// Parse a violin geometry
/// Format: violin() or violin(color: "blue", alpha: 0.7, width: 0.8, draw_quantiles: [0.25, 0.5, 0.75])
pub fn parse_violin(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("violin"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            // x: can be column
            map(preceded(ws(tag("x:")), ws(identifier)), |x| ("x", ArgValue::ColumnName(x))),
            // y: can be column
            map(preceded(ws(tag("y:")), ws(identifier)), |y| ("y", ArgValue::ColumnName(y))),

            // color: can be "literal", column
            map(preceded(ws(tag("color:")), ws(string_literal)), |c| ("color", ArgValue::ColorFixed(c))),
            map(preceded(ws(tag("color:")), ws(identifier)), |c| ("color", ArgValue::ColorMapped(c))),

            // width: can be number, column
            map(preceded(ws(tag("width:")), ws(number_literal)), |w| ("width", ArgValue::NumericFixed(w))),
            map(preceded(ws(tag("width:")), ws(identifier)), |w| ("width", ArgValue::NumericMapped(w))),

            // alpha: can be number, column
            map(preceded(ws(tag("alpha:")), ws(number_literal)), |a| ("alpha", ArgValue::NumericFixed(a))),
            map(preceded(ws(tag("alpha:")), ws(identifier)), |a| ("alpha", ArgValue::NumericMapped(a))),

            // Violin-specific: draw_quantiles array
            map(preceded(ws(tag("draw_quantiles:")), ws(parse_number_array)), |q| ("draw_quantiles", ArgValue::NumberArray(q))),
        ))
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = ViolinLayer::default();

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
            ("draw_quantiles", ArgValue::NumberArray(q)) => layer.draw_quantiles = q,
            _ => {}
        }
    }

    // Set stat with draw_quantiles for transform phase
    layer.stat = crate::parser::ast::Stat::Violin { draw_quantiles: layer.draw_quantiles.clone() };

    Ok((input, Layer::Violin(layer)))
}

/// Parse a density geometry
/// Format: density() or density(color: "blue", alpha: 0.3, bw: 1.5)
pub fn parse_density(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("density"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            // x: can be column
            map(preceded(ws(tag("x:")), ws(identifier)), |x| ("x", ArgValue::ColumnName(x))),

            // color: can be "literal", column
            map(preceded(ws(tag("color:")), ws(string_literal)), |c| ("color", ArgValue::ColorFixed(c))),
            map(preceded(ws(tag("color:")), ws(identifier)), |c| ("color", ArgValue::ColorMapped(c))),

            // alpha: can be number, column
            map(preceded(ws(tag("alpha:")), ws(number_literal)), |a| ("alpha", ArgValue::NumericFixed(a))),
            map(preceded(ws(tag("alpha:")), ws(identifier)), |a| ("alpha", ArgValue::NumericMapped(a))),

            // bw: bandwidth (number only)
            map(preceded(ws(tag("bw:")), ws(number_literal)), |b| ("bw", ArgValue::NumericFixed(b))),
        ))
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = DensityLayer::default();

    for (key, val) in args {
        match (key, val) {
            ("x", ArgValue::ColumnName(x)) => layer.x = Some(x),
            ("color", ArgValue::ColorFixed(c)) => layer.color = Some(AestheticValue::Fixed(c)),
            ("color", ArgValue::ColorMapped(c)) => layer.color = Some(AestheticValue::Mapped(c)),
            ("alpha", ArgValue::NumericFixed(a)) => layer.alpha = Some(AestheticValue::Fixed(a)),
            ("alpha", ArgValue::NumericMapped(a)) => layer.alpha = Some(AestheticValue::Mapped(a)),
            ("bw", ArgValue::NumericFixed(b)) => layer.bw = Some(b),
            _ => {}
        }
    }

    // Set stat with bandwidth for transform phase
    layer.stat = crate::parser::ast::Stat::Density { bw: layer.bw };

    Ok((input, Layer::Density(layer)))
}

/// Parse a heatmap geometry
/// Format: heatmap() or heatmap(bins: 20, alpha: 0.9, fill: value_col)
pub fn parse_heatmap(input: &str) -> IResult<&str, Layer> {
    let (input, _) = ws(tag("heatmap"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            // x: can be column
            map(preceded(ws(tag("x:")), ws(identifier)), |x| ("x", ArgValue::ColumnName(x))),
            // y: can be column
            map(preceded(ws(tag("y:")), ws(identifier)), |y| ("y", ArgValue::ColumnName(y))),
            // fill: column name for fill values
            map(preceded(ws(tag("fill:")), ws(identifier)), |f| ("fill", ArgValue::ColumnName(f))),
            // bins: number of bins for 2D binning
            map(preceded(ws(tag("bins:")), ws(number_literal)), |b| ("bins", ArgValue::NumericFixed(b))),
            // alpha: can be number
            map(preceded(ws(tag("alpha:")), ws(number_literal)), |a| ("alpha", ArgValue::NumericFixed(a))),
            map(preceded(ws(tag("alpha:")), ws(identifier)), |a| ("alpha", ArgValue::NumericMapped(a))),
        ))
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut layer = HeatmapLayer::default();
    let mut bins = None;

    for (key, val) in args {
        match (key, val) {
            ("x", ArgValue::ColumnName(x)) => layer.x = Some(x),
            ("y", ArgValue::ColumnName(y)) => layer.y = Some(y),
            ("fill", ArgValue::ColumnName(f)) => layer.fill = Some(f),
            ("bins", ArgValue::NumericFixed(b)) => bins = Some(b as usize),
            ("alpha", ArgValue::NumericFixed(a)) => layer.alpha = Some(AestheticValue::Fixed(a)),
            ("alpha", ArgValue::NumericMapped(a)) => layer.alpha = Some(AestheticValue::Mapped(a)),
            _ => {}
        }
    }

    layer.stat = crate::parser::ast::Stat::Heatmap { bins };

    Ok((input, Layer::Heatmap(layer)))
}

/// Parse any geometry layer
pub fn parse_geom(input: &str) -> IResult<&str, Layer> {
    alt((parse_line, parse_point, parse_bar, parse_ribbon, parse_histogram, parse_smooth, parse_boxplot, parse_violin, parse_density, parse_heatmap))(input)
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

    #[test]
    fn test_parse_density_empty() {
        let result = parse_density("density()");
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Density(d) => {
                assert_eq!(d.color, None);
                assert_eq!(d.alpha, None);
                assert_eq!(d.bw, None);
                assert!(matches!(d.stat, crate::parser::ast::Stat::Density { bw: None }));
            }
            _ => panic!("Expected Density layer"),
        }
    }

    #[test]
    fn test_parse_density_with_params() {
        let result = parse_density(r#"density(color: "blue", alpha: 0.5, bw: 1.5)"#);
        assert!(result.is_ok());
        let (_, layer) = result.unwrap();
        match layer {
            Layer::Density(d) => {
                assert_eq!(d.color, Some(AestheticValue::Fixed("blue".to_string())));
                assert_eq!(d.alpha, Some(AestheticValue::Fixed(0.5)));
                assert_eq!(d.bw, Some(1.5));
                assert!(matches!(d.stat, crate::parser::ast::Stat::Density { bw: Some(b) } if b == 1.5));
            }
            _ => panic!("Expected Density layer"),
        }
    }

    #[test]
    fn test_parse_density_in_pipeline() {
        use crate::parser::pipeline::parse_plot_spec;
        let result = parse_plot_spec(r#"aes(x: value) | density()"#);
        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert_eq!(spec.layers.len(), 1);
        assert!(matches!(spec.layers[0], Layer::Density(_)));
    }
}
use crate::parser::ast::{
    ElementLine, ElementRect, ElementText, LegendPosition, Theme, ThemeElement,
};
use crate::parser::lexer::{number_literal, string_literal, ws};
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::map,
    multi::separated_list0, sequence::preceded, IResult,
};

// === Element Parsers ===

/// Parse element_text(size: 20, face: "bold", color: "#333333", ...)
fn parse_element_text(input: &str) -> IResult<&str, ThemeElement> {
    let (input, _) = ws(tag("element_text"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            map(preceded(ws(tag("size:")), ws(number_literal)), |v| {
                ("size", ArgValue::Number(v))
            }),
            map(preceded(ws(tag("color:")), ws(string_literal)), |v| {
                ("color", ArgValue::String(v))
            }),
            map(preceded(ws(tag("family:")), ws(string_literal)), |v| {
                ("family", ArgValue::String(v))
            }),
            map(preceded(ws(tag("face:")), ws(string_literal)), |v| {
                ("face", ArgValue::String(v))
            }),
            map(preceded(ws(tag("angle:")), ws(number_literal)), |v| {
                ("angle", ArgValue::Number(v))
            }),
            map(preceded(ws(tag("hjust:")), ws(number_literal)), |v| {
                ("hjust", ArgValue::Number(v))
            }),
            map(preceded(ws(tag("vjust:")), ws(number_literal)), |v| {
                ("vjust", ArgValue::Number(v))
            }),
        )),
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut elem = ElementText::default();
    for (key, val) in args {
        match (key, val) {
            ("size", ArgValue::Number(v)) => elem.size = Some(v),
            ("color", ArgValue::String(v)) => elem.color = Some(v),
            ("family", ArgValue::String(v)) => elem.family = Some(v),
            ("face", ArgValue::String(v)) => elem.face = Some(v),
            ("angle", ArgValue::Number(v)) => elem.angle = Some(v),
            ("hjust", ArgValue::Number(v)) => elem.hjust = Some(v),
            ("vjust", ArgValue::Number(v)) => elem.vjust = Some(v),
            _ => {}
        }
    }

    Ok((input, ThemeElement::Text(elem)))
}

/// Parse element_line(color: "gray", width: 0.5, linetype: "dashed")
fn parse_element_line(input: &str) -> IResult<&str, ThemeElement> {
    let (input, _) = ws(tag("element_line"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            map(preceded(ws(tag("color:")), ws(string_literal)), |v| {
                ("color", ArgValue::String(v))
            }),
            map(preceded(ws(tag("width:")), ws(number_literal)), |v| {
                ("width", ArgValue::Number(v))
            }),
            map(preceded(ws(tag("linetype:")), ws(string_literal)), |v| {
                ("linetype", ArgValue::String(v))
            }),
        )),
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut elem = ElementLine::default();
    for (key, val) in args {
        match (key, val) {
            ("color", ArgValue::String(v)) => elem.color = Some(v),
            ("width", ArgValue::Number(v)) => elem.width = Some(v),
            ("linetype", ArgValue::String(v)) => elem.linetype = Some(v),
            _ => {}
        }
    }

    Ok((input, ThemeElement::Line(elem)))
}

/// Parse element_rect(fill: "white", color: "black", width: 1.0)
fn parse_element_rect(input: &str) -> IResult<&str, ThemeElement> {
    let (input, _) = ws(tag("element_rect"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            map(preceded(ws(tag("fill:")), ws(string_literal)), |v| {
                ("fill", ArgValue::String(v))
            }),
            map(preceded(ws(tag("color:")), ws(string_literal)), |v| {
                ("color", ArgValue::String(v))
            }),
            map(preceded(ws(tag("width:")), ws(number_literal)), |v| {
                ("width", ArgValue::Number(v))
            }),
        )),
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut elem = ElementRect::default();
    for (key, val) in args {
        match (key, val) {
            ("fill", ArgValue::String(v)) => elem.fill = Some(v),
            ("color", ArgValue::String(v)) => elem.color = Some(v),
            ("width", ArgValue::Number(v)) => elem.width = Some(v),
            _ => {}
        }
    }

    Ok((input, ThemeElement::Rect(elem)))
}

/// Parse element_blank()
fn parse_element_blank(input: &str) -> IResult<&str, ThemeElement> {
    let (input, _) = ws(tag("element_blank"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, _) = ws(char(')'))(input)?;
    Ok((input, ThemeElement::Blank))
}

/// Parse any theme element value
fn parse_theme_element(input: &str) -> IResult<&str, ThemeElement> {
    alt((
        parse_element_text,
        parse_element_line,
        parse_element_rect,
        parse_element_blank,
    ))(input)
}

// Helper enum for argument values
#[derive(Debug, Clone)]
enum ArgValue {
    String(String),
    Number(f64),
}

// === Theme Argument Parsing ===

#[derive(Debug)]
enum ThemeArg {
    LegendPosition(LegendPosition),
    LegendBackground(ThemeElement),
    LegendText(ThemeElement),
    LegendMargin(f64),
    LegendKeySize(f64),
    PlotBackground(ThemeElement),
    PlotTitle(ThemeElement),
    PanelBackground(ThemeElement),
    PanelGridMajor(ThemeElement),
    PanelGridMinor(ThemeElement),
    AxisText(ThemeElement),
    AxisLine(ThemeElement),
    AxisTicks(ThemeElement),
    Line(ThemeElement),
    Rect(ThemeElement),
    Text(ThemeElement),
}

fn parse_legend_position_arg(input: &str) -> IResult<&str, ThemeArg> {
    let (input, _) = ws(tag("legend_position:"))(input)?;
    let (input, val) = ws(string_literal)(input)?;
    let pos = match val.as_str() {
        "upper-left" => LegendPosition::UpperLeft,
        "upper-middle" | "top" => LegendPosition::UpperMiddle,
        "upper-right" => LegendPosition::UpperRight,
        "middle-left" | "left" => LegendPosition::MiddleLeft,
        "middle-middle" | "center" => LegendPosition::MiddleMiddle,
        "middle-right" | "right" => LegendPosition::MiddleRight,
        "lower-left" => LegendPosition::LowerLeft,
        "lower-middle" | "bottom" => LegendPosition::LowerMiddle,
        "lower-right" => LegendPosition::LowerRight,
        "none" => LegendPosition::None,
        _ => LegendPosition::UpperRight,
    };
    Ok((input, ThemeArg::LegendPosition(pos)))
}

fn parse_theme_arg(input: &str) -> IResult<&str, ThemeArg> {
    alt((
        parse_legend_position_arg,
        map(
            preceded(ws(tag("plot_background:")), ws(parse_theme_element)),
            ThemeArg::PlotBackground,
        ),
        map(
            preceded(ws(tag("plot_title:")), ws(parse_theme_element)),
            ThemeArg::PlotTitle,
        ),
        map(
            preceded(ws(tag("panel_background:")), ws(parse_theme_element)),
            ThemeArg::PanelBackground,
        ),
        map(
            preceded(ws(tag("panel_grid_major:")), ws(parse_theme_element)),
            ThemeArg::PanelGridMajor,
        ),
        map(
            preceded(ws(tag("panel_grid_minor:")), ws(parse_theme_element)),
            ThemeArg::PanelGridMinor,
        ),
        map(
            preceded(ws(tag("axis_text:")), ws(parse_theme_element)),
            ThemeArg::AxisText,
        ),
        map(
            preceded(ws(tag("axis_line:")), ws(parse_theme_element)),
            ThemeArg::AxisLine,
        ),
        map(
            preceded(ws(tag("axis_ticks:")), ws(parse_theme_element)),
            ThemeArg::AxisTicks,
        ),
        map(
            preceded(ws(tag("legend_background:")), ws(parse_theme_element)),
            ThemeArg::LegendBackground,
        ),
        map(
            preceded(ws(tag("legend_text:")), ws(parse_theme_element)),
            ThemeArg::LegendText,
        ),
        map(
            preceded(ws(tag("legend_margin:")), ws(number_literal)),
            ThemeArg::LegendMargin,
        ),
        map(
            preceded(ws(tag("legend_key_size:")), ws(number_literal)),
            ThemeArg::LegendKeySize,
        ),
        map(
            preceded(ws(tag("line:")), ws(parse_theme_element)),
            ThemeArg::Line,
        ),
        map(
            preceded(ws(tag("rect:")), ws(parse_theme_element)),
            ThemeArg::Rect,
        ),
        map(
            preceded(ws(tag("text:")), ws(parse_theme_element)),
            ThemeArg::Text,
        ),
    ))(input)
}

// === Main Theme Parsers ===

/// Parse theme_minimal() - returns a preset minimal theme
pub fn parse_theme_minimal(input: &str) -> IResult<&str, Theme> {
    let (input, _) = ws(tag("theme_minimal"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, _) = ws(char(')'))(input)?;

    Ok((
        input,
        Theme {
            line: ThemeElement::Inherit,
            rect: ThemeElement::Inherit,
            text: ThemeElement::Inherit,
            plot_background: ThemeElement::Rect(ElementRect {
                fill: Some("white".to_string()),
                ..Default::default()
            }),
            plot_title: ThemeElement::Inherit,
            panel_background: ThemeElement::Rect(ElementRect {
                fill: Some("white".to_string()),
                ..Default::default()
            }),
            panel_grid_major: ThemeElement::Line(ElementLine {
                color: Some("#CCCCCC".to_string()),
                width: Some(0.5),
                ..Default::default()
            }),
            panel_grid_minor: ThemeElement::Blank,
            axis_text: ThemeElement::Inherit,
            axis_line: ThemeElement::Blank,
            axis_ticks: ThemeElement::Blank,
            legend_position: None,
            legend_background: ThemeElement::Inherit,
            legend_text: ThemeElement::Inherit,
            legend_margin: None,
            legend_key_size: None,
        },
    ))
}

/// Parse theme_dark() - dark background with light foreground elements
pub fn parse_theme_dark(input: &str) -> IResult<&str, Theme> {
    let (input, _) = ws(tag("theme_dark"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, _) = ws(char(')'))(input)?;

    Ok((
        input,
        Theme {
            line: ThemeElement::Inherit,
            rect: ThemeElement::Inherit,
            text: ThemeElement::Text(ElementText {
                color: Some("#f2f2f2".to_string()),
                ..Default::default()
            }),
            plot_background: ThemeElement::Rect(ElementRect {
                fill: Some("#1f1f1f".to_string()),
                ..Default::default()
            }),
            plot_title: ThemeElement::Inherit,
            panel_background: ThemeElement::Rect(ElementRect {
                fill: Some("#2b2b2b".to_string()),
                ..Default::default()
            }),
            panel_grid_major: ThemeElement::Line(ElementLine {
                color: Some("#555555".to_string()),
                width: Some(0.5),
                ..Default::default()
            }),
            panel_grid_minor: ThemeElement::Line(ElementLine {
                color: Some("#3f3f3f".to_string()),
                width: Some(0.25),
                ..Default::default()
            }),
            axis_text: ThemeElement::Text(ElementText {
                color: Some("#d8d8d8".to_string()),
                ..Default::default()
            }),
            axis_line: ThemeElement::Line(ElementLine {
                color: Some("#d8d8d8".to_string()),
                width: Some(1.0),
                ..Default::default()
            }),
            axis_ticks: ThemeElement::Line(ElementLine {
                color: Some("#d8d8d8".to_string()),
                width: Some(1.0),
                ..Default::default()
            }),
            legend_position: None,
            legend_background: ThemeElement::Rect(ElementRect {
                fill: Some("#2b2b2b".to_string()),
                color: Some("#d8d8d8".to_string()),
                width: Some(1.0),
            }),
            legend_text: ThemeElement::Text(ElementText {
                color: Some("#f2f2f2".to_string()),
                ..Default::default()
            }),
            legend_margin: None,
            legend_key_size: None,
        },
    ))
}

/// Parse theme_classic() - white background, axis lines, and no grid
pub fn parse_theme_classic(input: &str) -> IResult<&str, Theme> {
    let (input, _) = ws(tag("theme_classic"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, _) = ws(char(')'))(input)?;

    Ok((
        input,
        Theme {
            line: ThemeElement::Inherit,
            rect: ThemeElement::Inherit,
            text: ThemeElement::Inherit,
            plot_background: ThemeElement::Rect(ElementRect {
                fill: Some("white".to_string()),
                ..Default::default()
            }),
            plot_title: ThemeElement::Inherit,
            panel_background: ThemeElement::Rect(ElementRect {
                fill: Some("white".to_string()),
                ..Default::default()
            }),
            panel_grid_major: ThemeElement::Blank,
            panel_grid_minor: ThemeElement::Blank,
            axis_text: ThemeElement::Inherit,
            axis_line: ThemeElement::Line(ElementLine {
                color: Some("black".to_string()),
                width: Some(1.0),
                ..Default::default()
            }),
            axis_ticks: ThemeElement::Line(ElementLine {
                color: Some("black".to_string()),
                width: Some(1.0),
                ..Default::default()
            }),
            legend_position: None,
            legend_background: ThemeElement::Rect(ElementRect {
                fill: Some("white".to_string()),
                color: Some("black".to_string()),
                width: Some(1.0),
            }),
            legend_text: ThemeElement::Inherit,
            legend_margin: None,
            legend_key_size: None,
        },
    ))
}

/// Parse theme_void() - blank panel with no axes, ticks, grid, or legend.
pub fn parse_theme_void(input: &str) -> IResult<&str, Theme> {
    let (input, _) = ws(tag("theme_void"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, _) = ws(char(')'))(input)?;

    Ok((
        input,
        Theme {
            line: ThemeElement::Inherit,
            rect: ThemeElement::Inherit,
            text: ThemeElement::Inherit,
            plot_background: ThemeElement::Rect(ElementRect {
                fill: Some("white".to_string()),
                ..Default::default()
            }),
            plot_title: ThemeElement::Inherit,
            panel_background: ThemeElement::Rect(ElementRect {
                fill: Some("white".to_string()),
                ..Default::default()
            }),
            panel_grid_major: ThemeElement::Blank,
            panel_grid_minor: ThemeElement::Blank,
            axis_text: ThemeElement::Text(ElementText {
                color: Some("white".to_string()),
                ..Default::default()
            }),
            axis_line: ThemeElement::Blank,
            axis_ticks: ThemeElement::Blank,
            legend_position: Some(LegendPosition::None),
            legend_background: ThemeElement::Blank,
            legend_text: ThemeElement::Inherit,
            legend_margin: None,
            legend_key_size: None,
        },
    ))
}

/// Parse theme(...) with hierarchical element arguments
pub fn parse_theme(input: &str) -> IResult<&str, Theme> {
    let (input, _) = ws(tag("theme"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(ws(char(',')), parse_theme_arg)(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut theme = Theme::default();
    for arg in args {
        match arg {
            ThemeArg::LegendPosition(pos) => theme.legend_position = Some(pos),
            ThemeArg::LegendBackground(elem) => theme.legend_background = elem,
            ThemeArg::LegendText(elem) => theme.legend_text = elem,
            ThemeArg::LegendMargin(margin) => theme.legend_margin = Some(margin),
            ThemeArg::LegendKeySize(size) => theme.legend_key_size = Some(size),
            ThemeArg::PlotBackground(elem) => theme.plot_background = elem,
            ThemeArg::PlotTitle(elem) => theme.plot_title = elem,
            ThemeArg::PanelBackground(elem) => theme.panel_background = elem,
            ThemeArg::PanelGridMajor(elem) => theme.panel_grid_major = elem,
            ThemeArg::PanelGridMinor(elem) => theme.panel_grid_minor = elem,
            ThemeArg::AxisText(elem) => theme.axis_text = elem,
            ThemeArg::AxisLine(elem) => theme.axis_line = elem,
            ThemeArg::AxisTicks(elem) => theme.axis_ticks = elem,
            ThemeArg::Line(elem) => theme.line = elem,
            ThemeArg::Rect(elem) => theme.rect = elem,
            ThemeArg::Text(elem) => theme.text = elem,
        }
    }

    Ok((input, theme))
}

/// Parse any theme command (theme_minimal or theme)
pub fn parse_theme_command(input: &str) -> IResult<&str, Theme> {
    alt((
        parse_theme_minimal,
        parse_theme_dark,
        parse_theme_classic,
        parse_theme_void,
        parse_theme,
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_element_blank() {
        let result = parse_element_blank("element_blank()");
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        assert_eq!(elem, ThemeElement::Blank);
    }

    #[test]
    fn test_parse_element_text() {
        let result = parse_element_text("element_text(size: 20, face: \"bold\")");
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        if let ThemeElement::Text(t) = elem {
            assert_eq!(t.size, Some(20.0));
            assert_eq!(t.face, Some("bold".to_string()));
        } else {
            panic!("Expected Text element");
        }
    }

    #[test]
    fn test_parse_element_line() {
        let result = parse_element_line("element_line(color: \"gray\", width: 0.5)");
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        if let ThemeElement::Line(l) = elem {
            assert_eq!(l.color, Some("gray".to_string()));
            assert_eq!(l.width, Some(0.5));
        } else {
            panic!("Expected Line element");
        }
    }

    #[test]
    fn test_parse_element_rect() {
        let result = parse_element_rect("element_rect(fill: \"white\")");
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        if let ThemeElement::Rect(r) = elem {
            assert_eq!(r.fill, Some("white".to_string()));
        } else {
            panic!("Expected Rect element");
        }
    }

    #[test]
    fn test_parse_theme_minimal() {
        let result = parse_theme_minimal("theme_minimal()");
        assert!(result.is_ok());
        let (_, theme) = result.unwrap();
        assert_eq!(theme.panel_grid_minor, ThemeElement::Blank);
        assert_eq!(theme.axis_line, ThemeElement::Blank);
    }

    #[test]
    fn test_parse_theme_void() {
        let result = parse_theme_void("theme_void()");
        assert!(result.is_ok());
        let (_, theme) = result.unwrap();
        assert_eq!(theme.panel_grid_major, ThemeElement::Blank);
        assert_eq!(theme.axis_line, ThemeElement::Blank);
        assert_eq!(theme.axis_ticks, ThemeElement::Blank);
        assert_eq!(theme.legend_position, Some(LegendPosition::None));
    }

    #[test]
    fn test_parse_theme_with_elements() {
        let result = parse_theme(
            "theme(plot_title: element_text(size: 24), panel_grid_minor: element_blank())",
        );
        assert!(result.is_ok());
        let (_, theme) = result.unwrap();
        if let ThemeElement::Text(t) = &theme.plot_title {
            assert_eq!(t.size, Some(24.0));
        } else {
            panic!("Expected Text element for plot_title");
        }
        assert_eq!(theme.panel_grid_minor, ThemeElement::Blank);
    }

    #[test]
    fn test_parse_theme_legend_position() {
        let result = parse_theme("theme(legend_position: \"bottom\")");
        assert!(result.is_ok());
        let (_, theme) = result.unwrap();
        assert_eq!(theme.legend_position, Some(LegendPosition::LowerMiddle));
    }

    #[test]
    fn test_parse_theme_legend_config() {
        let result = parse_theme(
            r##"theme(legend_text: element_text(size: 14, color: "#333333"), legend_background: element_rect(fill: "white", color: "black"), legend_margin: 6, legend_key_size: 18)"##,
        );
        assert!(result.is_ok());
        let (_, theme) = result.unwrap();

        if let ThemeElement::Text(t) = &theme.legend_text {
            assert_eq!(t.size, Some(14.0));
            assert_eq!(t.color, Some("#333333".to_string()));
        } else {
            panic!("Expected Text element for legend_text");
        }

        if let ThemeElement::Rect(r) = &theme.legend_background {
            assert_eq!(r.fill, Some("white".to_string()));
            assert_eq!(r.color, Some("black".to_string()));
        } else {
            panic!("Expected Rect element for legend_background");
        }

        assert_eq!(theme.legend_margin, Some(6.0));
        assert_eq!(theme.legend_key_size, Some(18.0));
    }

    #[test]
    fn test_parse_preset_themes() {
        assert!(parse_theme_command("theme_dark()").is_ok());
        assert!(parse_theme_command("theme_classic()").is_ok());
    }

    #[test]
    fn test_parse_theme_hex_color() {
        let result = parse_theme("theme(axis_text: element_text(color: \"#FF0000\"))");
        assert!(result.is_ok());
        let (_, theme) = result.unwrap();
        if let ThemeElement::Text(t) = &theme.axis_text {
            assert_eq!(t.color, Some("#FF0000".to_string()));
        } else {
            panic!("Expected Text element for axis_text");
        }
    }
}

use crate::parser::ast::Labels;
use crate::parser::lexer::{string_literal, ws};
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::map,
    multi::separated_list0, sequence::preceded, IResult,
};

pub fn parse_labs(input: &str) -> IResult<&str, Labels> {
    let (input, _) = ws(tag("labs"))(input)?;
    let (input, _) = ws(char('('))(input)?;

    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            // title: always string literal
            map(preceded(ws(tag("title:")), ws(string_literal)), |v| {
                ("title", v)
            }),
            // subtitle: always string literal
            map(preceded(ws(tag("subtitle:")), ws(string_literal)), |v| {
                ("subtitle", v)
            }),
            // x: always string literal
            map(preceded(ws(tag("x:")), ws(string_literal)), |v| ("x", v)),
            // y: always string literal
            map(preceded(ws(tag("y:")), ws(string_literal)), |v| ("y", v)),
            // caption: always string literal
            map(preceded(ws(tag("caption:")), ws(string_literal)), |v| {
                ("caption", v)
            }),
        )),
    )(input)?;

    let (input, _) = ws(char(')'))(input)?;

    let mut labels = Labels::default();
    for (key, val) in args {
        match key {
            "title" => labels.title = Some(val),
            "subtitle" => labels.subtitle = Some(val),
            "x" => labels.x = Some(val),
            "y" => labels.y = Some(val),
            "caption" => labels.caption = Some(val),
            _ => {}
        }
    }

    Ok((input, labels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_labs() {
        let result = parse_labs(r#"labs(title: "My Chart", x: "X Axis")"#);
        assert!(result.is_ok());
        let (_, labels) = result.unwrap();
        assert_eq!(labels.title, Some("My Chart".to_string()));
        assert_eq!(labels.x, Some("X Axis".to_string()));
    }
}

use nom::{
    bytes::complete::tag,
    character::complete::char,
    branch::alt,
    combinator::{map},
    multi::separated_list0,
    sequence::{delimited, preceded},
    IResult,
};
use crate::parser::ast::{AxisScale, DateTimeScaleOptions, ScaleType};
use crate::parser::lexer::{number_literal, string_literal, ws};

fn axis_scale(scale_type: ScaleType, limits: Option<(f64, f64)>) -> AxisScale {
    AxisScale {
        scale_type,
        limits,
        datetime: None,
    }
}

pub fn parse_scale_x_log10(input: &str) -> IResult<&str, AxisScale> {
    let (input, _) = ws(tag("scale_x_log10"))(input)?;
    let (input, _) = delimited(tag("("), ws(tag("")), tag(")"))(input)?;
    Ok((input, axis_scale(ScaleType::Log10, None)))
}

pub fn parse_scale_y_log10(input: &str) -> IResult<&str, AxisScale> {
    let (input, _) = ws(tag("scale_y_log10"))(input)?;
    let (input, _) = delimited(tag("("), ws(tag("")), tag(")"))(input)?;
    Ok((input, axis_scale(ScaleType::Log10, None)))
}

pub fn parse_scale_x_reverse(input: &str) -> IResult<&str, AxisScale> {
    let (input, _) = ws(tag("scale_x_reverse"))(input)?;
    let (input, _) = delimited(tag("("), ws(tag("")), tag(")"))(input)?;
    Ok((input, axis_scale(ScaleType::Reverse, None)))
}

pub fn parse_scale_y_reverse(input: &str) -> IResult<&str, AxisScale> {
    let (input, _) = ws(tag("scale_y_reverse"))(input)?;
    let (input, _) = delimited(tag("("), ws(tag("")), tag(")"))(input)?;
    Ok((input, axis_scale(ScaleType::Reverse, None)))
}

pub fn parse_xlim(input: &str) -> IResult<&str, AxisScale> {
    let (input, _) = ws(tag("xlim"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, min) = ws(number_literal)(input)?;
    let (input, _) = ws(char(','))(input)?;
    let (input, max) = ws(number_literal)(input)?;
    let (input, _) = ws(char(')'))(input)?;
    Ok((input, axis_scale(ScaleType::Linear, Some((min, max)))))
}

pub fn parse_ylim(input: &str) -> IResult<&str, AxisScale> {
    let (input, _) = ws(tag("ylim"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, min) = ws(number_literal)(input)?;
    let (input, _) = ws(char(','))(input)?;
    let (input, max) = ws(number_literal)(input)?;
    let (input, _) = ws(char(')'))(input)?;
    Ok((input, axis_scale(ScaleType::Linear, Some((min, max)))))
}

#[derive(Debug)]
enum DateTimeScaleArg {
    Interval(String),
    Format(String),
}

pub fn parse_scale_x_datetime(input: &str) -> IResult<&str, AxisScale> {
    let (input, _) = ws(tag("scale_x_datetime"))(input)?;
    let (input, _) = ws(char('('))(input)?;
    let (input, args) = separated_list0(
        ws(char(',')),
        alt((
            map(preceded(ws(tag("interval:")), ws(string_literal)), DateTimeScaleArg::Interval),
            map(preceded(ws(tag("format:")), ws(string_literal)), DateTimeScaleArg::Format),
        )),
    )(input)?;
    let (input, _) = ws(char(')'))(input)?;

    let mut datetime = DateTimeScaleOptions {
        interval: None,
        format: None,
    };

    for arg in args {
        match arg {
            DateTimeScaleArg::Interval(value) => datetime.interval = Some(value),
            DateTimeScaleArg::Format(value) => datetime.format = Some(value),
        }
    }

    Ok((
        input,
        AxisScale {
            scale_type: ScaleType::DateTime,
            limits: None,
            datetime: Some(datetime),
        },
    ))
}

pub fn parse_scale_command(input: &str) -> IResult<&str, (bool, AxisScale)> {
    alt((
        map(parse_scale_x_datetime, |s| (true, s)),
        map(parse_scale_x_log10, |s| (true, s)),
        map(parse_scale_y_log10, |s| (false, s)),
        map(parse_scale_x_reverse, |s| (true, s)),
        map(parse_scale_y_reverse, |s| (false, s)),
        map(parse_xlim, |s| (true, s)),
        map(parse_ylim, |s| (false, s)),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scale_x_datetime_with_interval_and_format() {
        let (_, scale) = parse_scale_x_datetime(
            r#"scale_x_datetime(interval: "20h", format: "%b %-d %H:%M")"#,
        ).unwrap();

        assert_eq!(scale.scale_type, ScaleType::DateTime);
        let datetime = scale.datetime.unwrap();
        assert_eq!(datetime.interval, Some("20h".to_string()));
        assert_eq!(datetime.format, Some("%b %-d %H:%M".to_string()));
    }

    #[test]
    fn parse_scale_x_datetime_without_args() {
        let (_, scale) = parse_scale_x_datetime("scale_x_datetime()").unwrap();

        assert_eq!(scale.scale_type, ScaleType::DateTime);
        assert_eq!(
            scale.datetime,
            Some(DateTimeScaleOptions { interval: None, format: None })
        );
    }
}

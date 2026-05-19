use nom::{
    bytes::complete::take_while1,
    character::complete::{char, multispace0},
    combinator::recognize,
    number::complete::double,
    sequence::delimited,
    IResult,
};

pub fn ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn identifier(input: &str) -> IResult<&str, String> {
    let (input, ident) = recognize(take_while1(|c: char| c.is_alphanumeric() || c == '_'))(input)?;

    if let Some(first) = ident.chars().next() {
        if !first.is_alphabetic() && first != '_' {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Alpha,
            )));
        }
    }

    Ok((input, ident.to_string()))
}

pub fn string_literal(input: &str) -> IResult<&str, String> {
    let (input, content) = delimited(char('\"'), take_while1(|c| c != '\"'), char('\"'))(input)?;

    Ok((input, content.to_string()))
}

pub fn number_literal(input: &str) -> IResult<&str, f64> {
    double(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::bytes::complete::tag;

    #[test]
    fn test_identifier() {
        assert_eq!(identifier("foo"), Ok(("", "foo".to_string())));
        assert_eq!(identifier("foo123"), Ok(("", "foo123".to_string())));
        assert_eq!(identifier("_bar"), Ok(("", "_bar".to_string())));
        assert_eq!(
            identifier("foo_bar_123"),
            Ok(("", "foo_bar_123".to_string()))
        );
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(string_literal(r#""hello""#), Ok(("", "hello".to_string())));
        assert_eq!(string_literal(r#""US""#), Ok(("", "US".to_string())));
    }

    #[test]
    fn test_number_literal() {
        assert_eq!(number_literal("42"), Ok(("", 42.0)));
        assert_eq!(number_literal("3.5"), Ok(("", 3.5)));
        assert_eq!(number_literal("2020"), Ok(("", 2020.0)));
    }

    #[test]
    fn test_ws() {
        let mut parser = ws(tag("foo"));
        assert_eq!(parser("  foo  "), Ok(("", "foo")));
        assert_eq!(parser("foo"), Ok(("", "foo")));
        assert_eq!(parser("\n\tfoo\t\n"), Ok(("", "foo")));
    }

    #[test]
    fn test_identifier_invalid_start_with_number() {
        assert!(identifier("123abc").is_err());
        assert!(identifier("1test").is_err());
    }

    #[test]
    fn test_identifier_underscore_only() {
        assert_eq!(identifier("_"), Ok(("", "_".to_string())));
        assert_eq!(identifier("__"), Ok(("", "__".to_string())));
    }

    #[test]
    fn test_string_literal_empty() {
        assert!(string_literal(r#""#).is_err());
    }

    #[test]
    fn test_string_literal_unclosed() {
        assert!(string_literal(r#""hello"#).is_err());
    }

    #[test]
    fn test_number_literal_negative() {
        assert_eq!(number_literal("-42"), Ok(("", -42.0)));
        assert_eq!(number_literal("-3.5"), Ok(("", -3.5)));
        assert_eq!(number_literal("-0.1"), Ok(("", -0.1)));
    }
}

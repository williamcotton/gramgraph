use crate::parser::ast::CoordSystem;
use nom::{bytes::complete::tag, character::complete::multispace0, sequence::delimited, IResult};

pub fn parse_coord_flip(input: &str) -> IResult<&str, CoordSystem> {
    let (input, _) = tag("coord_flip")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = delimited(tag("("), multispace0, tag(")"))(input)?;

    Ok((input, CoordSystem::Flip))
}

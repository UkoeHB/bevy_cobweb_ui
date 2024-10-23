use nom::branch::alt;
use nom::character::complete::{digit1, one_of};
use nom::combinator::recognize;
use nom::error::ErrorKind;
use nom::multi::many0_count;
use nom::sequence::tuple;
use nom::{IResult, InputTakeAtPosition, Parser};

use super::Span;

//-------------------------------------------------------------------------------------------------------------------

/// Recognizes lowercase letters.
fn lowercase_alpha1(input: Span) -> IResult<Span, Span>
{
    input.split_at_position1_complete(|item| !item.is_ascii_lowercase(), ErrorKind::Alpha)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses a snake-case identifier from the input.
///
/// The identifier must contain only lower-case letters, numbers, and underscores, and must start with a letter.
pub(crate) fn snake_identifier(input: Span) -> IResult<Span, Span>
{
    recognize(tuple((
        lowercase_alpha1,
        many0_count(alt((lowercase_alpha1, digit1, recognize(one_of("_"))))),
    )))
    .parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

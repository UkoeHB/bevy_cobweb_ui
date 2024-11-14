use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric0, char, digit1};
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

/// Recognizes uppercase letters.
fn uppercase_alpha1(input: Span) -> IResult<Span, Span>
{
    input.split_at_position1_complete(|item| !item.is_ascii_uppercase(), ErrorKind::Alpha)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses a snake-case identifier from the input.
///
/// The identifier must contain only lower-case letters, numbers, and underscores, and must start with a letter.
pub(crate) fn snake_identifier(input: Span) -> IResult<Span, Span>
{
    recognize(tuple((
        lowercase_alpha1,
        many0_count(alt((lowercase_alpha1, digit1, recognize(char('_'))))),
    )))
    .parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses a snake-case identifier from the input.
///
/// The identifier must contain only lower-case letters, numbers, and underscores, and must start with a letter or
/// number.
pub(crate) fn numerical_snake_identifier(input: Span) -> IResult<Span, Span>
{
    recognize(tuple((
        alt((lowercase_alpha1, digit1)),
        many0_count(alt((lowercase_alpha1, digit1, recognize(char('_'))))),
    )))
    .parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses a camel-case identifier from the input.
///
/// The identifier must contain only upper-case and lower-case letters and numbers, and must start with an
/// upper-case letter.
pub(crate) fn camel_identifier(input: Span) -> IResult<Span, Span>
{
    recognize(tuple((uppercase_alpha1, alphanumeric0))).parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses an identifier from the input.
///
/// The identifier must contain only upper and lower-case letters, numbers, and underscores. It must start with
/// a letter or number.
pub(crate) fn anything_identifier(input: Span) -> IResult<Span, Span>
{
    recognize(tuple((
        alt((alpha1, digit1)),
        many0_count(alt((alpha1, digit1, recognize(char('_'))))),
    )))
    .parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

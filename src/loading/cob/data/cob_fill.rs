use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::one_of;
use nom::combinator::{map, recognize, rest};
use nom::error::ErrorKind;
use nom::multi::many0_count;
use nom::sequence::{preceded, terminated};
use nom::{AsChar, Parser};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Special characters that can appear after a fill sequence.
fn is_allowed_special_char(c: char) -> bool
{
    match c {
        '"' | ':' | '#' | '@' | '+' | '-' | '=' | '$' | '?' | '{' | '}' | '(' | ')' | '[' | ']' | '<' | '>'
        | '.' | '\'' | '\\' | '_' | '^' | '!' => true,
        _ => false,
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns true if the next character is banned immediately after a fill sequence.
fn invalid_postfill(c: char) -> bool
{
    !c.is_alphanum() && !is_allowed_special_char(c)
}

//-------------------------------------------------------------------------------------------------------------------

/// Section of a COB file containing filler charaters.
///
/// Includes whitespace (spaces and newlines), comments (line and block comments), and ignored characters (commas
/// and semicolons).
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CobFill
{
    // TODO: replace with Cow of string slice
    pub string: SmolStr,
}

impl CobFill
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.string.as_bytes())?;
        Ok(())
    }

    pub fn write_to_or_else(
        &self,
        writer: &mut impl RawSerializer,
        fallback: impl AsRef<str>,
    ) -> Result<(), std::io::Error>
    {
        if self.string.len() == 0 {
            writer.write_bytes(fallback.as_ref().as_bytes())?;
        } else {
            self.write_to(writer)?;
        }
        Ok(())
    }

    /// Parses a fill sequence from the input string. The fill may be empty.
    pub fn parse(input: Span) -> (CobFill, Span)
    {
        // NOTE: recursion not tested here (not vulnerable)

        // Get fill.
        let (remaining, fill) = map(
            recognize(many0_count(alt((
                // collect whitespace and special characters
                recognize(one_of::<_, _, (Span, ErrorKind)>(" \n,;")),
                // collect line comments until \n or end of file
                preceded(tag("//"), alt((terminated(take_until("\n"), tag("\n")), rest))),
                // collect block comments until block terminator or end of file
                preceded(tag("/*"), alt((terminated(take_until("*/"), tag("*/")), rest))),
            )))),
            |r| -> CobFill { CobFill::new(*r.fragment()) },
        )
        .parse(input)
        .expect("parsing fill should never fail");

        // Cleanup illegal characters.
        let remaining = if let Ok((after_invalid, invalid_span)) =
            take_while1::<_, _, ()>(invalid_postfill).parse(remaining)
        {
            // Note: use CobStringSegment to escape all weird characters in the invalid span
            tracing::warn!(
                "discarding banned character sequence \"{}\" at {}",
                String::from_utf8(CobStringSegment::from(*invalid_span.fragment()).original).unwrap().as_str(),
                get_location(remaining).as_str()
            );
            after_invalid
        } else {
            remaining
        };

        (fill, remaining)
    }

    pub fn new(string: impl Into<SmolStr>) -> Self
    {
        Self { string: string.into() }
    }

    pub fn len(&self) -> usize
    {
        self.string.len()
    }

    /// Number of space characters at the end of the filler if it ends in '\n   '.
    ///
    /// Used to calibrate scene tree depth of scene nodes.
    pub fn ends_newline_then_num_spaces(&self) -> Option<usize>
    {
        let trimmed = self.string.as_str().trim_end_matches(' ');
        if !trimmed.ends_with('\n') {
            return None;
        }
        Some(self.len() - trimmed.len())
    }

    /// Checks if the filler ends in a newline.
    pub fn ends_with_newline(&self) -> bool
    {
        self.string.as_str().ends_with('\n')
    }

    /// If `self.len() == 0` then clone the other's fill value.
    pub fn recover(&mut self, other: &Self)
    {
        if self.len() == 0 {
            *self = other.clone();
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

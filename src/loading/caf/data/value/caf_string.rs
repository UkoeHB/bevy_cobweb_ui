//! String parsing adapted from the nom crate's `string.rs` example.
//!
//! Rules:
//! - Enclosed by double quotes
//! - Can contain any raw unescaped code point besides \ and "
//! - Matches the following escape sequences: \b, \f, \n, \r, \t, \", \\,
//! - Matches code points like Rust: \u{XXXX}, where XXXX can be 1 to 6 hex characters
//! - an escape followed by whitespace consumes all whitespace between the escape and the next non-whitespace
//!   character, then creates a new string segment

use bevy::prelude::default;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::streaming::{is_not, take_while_m_n};
use nom::character::streaming::char;
use nom::combinator::{map, map_opt, map_res, peek, value, verify};
use nom::multi::{fold_many0, many0_count};
use nom::sequence::{delimited, preceded};
use nom::{IResult, Parser};
use smallvec::SmallVec;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Parses a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals.
fn parse_unicode(input: Span) -> IResult<Span, char>
{
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), delimited(char('{'), parse_hex, char('}')));
    let parse_u32 = map_res(parse_delimited_hex, move |hex: Span| {
        u32::from_str_radix(*hex.fragment(), 16)
    });
    map_opt(parse_u32, std::char::from_u32).parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal(input: Span) -> IResult<Span, Span>
{
    verify(is_not("\"\\"), |s: &Span| !s.fragment().is_empty()).parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char(input: Span) -> IResult<Span, char>
{
    preceded(
        char('\\'),
        alt((
            parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('"', char('"')),
        )),
    )
    .parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses a backslash, followed by a newline then any amount of spaces.
///
/// Must be called after `parse_escaped_char`.
///
/// Returns the original input, the remaining input, and the number of spaces counted.
fn parse_new_section(input: Span) -> IResult<Span, (Span, usize, Span)>
{
    preceded(tag("\\\n"), many0_count(char(' ')))
        .parse(input)
        .map(|(r, c)| (r, (input, c, r)))
}

//-------------------------------------------------------------------------------------------------------------------

/// Parses a fragment of a string: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped fill characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a>
{
    Literal(Span<'a>),
    EscapedChar(char),
    /// (remaining input at and including '\' symbol, num spaces after '\\n', remaining input after spaces)
    EscapedSpaces((Span<'a>, usize, Span<'a>)),
}

//-------------------------------------------------------------------------------------------------------------------

fn parse_fragment(input: Span) -> IResult<Span, StringFragment>
{
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        map(parse_new_section, StringFragment::EscapedSpaces),
        map_res(char('\\'), |_| -> Result<StringFragment, SpanError> {
            tracing::warn!("failed parsing string at {}; invalid escape sequence \
                (supported: \\n, \\r, \\t, \\b, \\f, \\\\, \\\", \\u{{<unicode hex>}}, \\<newline><spaces>)",
                get_location(input).as_str());
            Err(span_verify_error(input))
        }),
    ))
    .parse(input)
}

//-------------------------------------------------------------------------------------------------------------------

fn parse_string(input: Span) -> IResult<Span, SmallVec<[CafStringSegment; 1]>>
{
    let build_string = fold_many0(
        parse_fragment,
        || (input, SmallVec::from_elem(CafStringSegment::default(), 1)),
        |(mut input, mut segments): (_, SmallVec<[CafStringSegment; 1]>), fragment| {
            match fragment {
                StringFragment::Literal(s) => segments.last_mut().unwrap().segment.push_str(*s.fragment()),
                StringFragment::EscapedChar(c) => segments.last_mut().unwrap().segment.push(c),
                StringFragment::EscapedSpaces((after_segment, leading_spaces, next_segment)) => {
                    let input_bytes = input.fragment().as_bytes();
                    let after_bytes = after_segment.fragment().as_bytes();
                    let len = input_bytes.len().saturating_sub(after_bytes.len());
                    segments.last_mut().unwrap().original = Vec::from(&input_bytes[..len]);
                    segments.push(CafStringSegment { leading_spaces, ..default() });
                    input = next_segment;
                }
            }
            (input, segments)
        },
    );

    // Note: assumes no internal parsers match directly on '"'.
    delimited(char('"'), build_string, char('"'))
        .parse(input)
        // Collect original bytes of the last segment.
        .map(|(remaining, (input, mut segments))| {
            let input_bytes = input.fragment().as_bytes();
            let remaining_bytes = remaining.fragment().as_bytes();
            let len = input_bytes.len().saturating_sub(remaining_bytes.len() + 1); // + 1 for the terminal '"'
            segments.last_mut().unwrap().original = Vec::from(&input_bytes[..len]);
            (remaining, segments)
        })
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafStringSegment
{
    /// Spaces at the start of a segment for multiline text.
    pub leading_spaces: usize,
    /// The originally-parsed bytes, cached for writing back to raw bytes.
    ///
    /// We do this because bytes -> string -> bytes is potentially lossy, since the -> bytes step will convert
    /// newlines and Unicode characters to escape sequences even if they were literals in the original bytes.
    /// Note that it's possible editors can't support
    /// a mix of explicit and implicit newlines, and they will just have to aggressively replace implicit newlines
    /// with explicit newlines.
    //todo: replace this with a shared memory structure like Bytes or Cow<[u8]>?
    pub original: Vec<u8>,
    pub segment: String,
}

impl CafStringSegment
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        for _ in 0..self.leading_spaces {
            writer.write_bytes(" ".as_bytes())?;
        }
        // Here we write raw bytes.
        writer.write_bytes(&self.original)?;
        Ok(())
    }
}

impl From<&str> for CafStringSegment
{
    fn from(string: &str) -> Self
    {
        Self::from(String::from(string))
    }
}

impl From<char> for CafStringSegment
{
    fn from(character: char) -> Self
    {
        Self::from(String::from(character))
    }
}

impl From<String> for CafStringSegment
{
    /// Note that `Self::write_to` -> `Self::from` is lossy because String has no awareness of
    /// multi-line string formatting. The string contents are preserved, but not their presentation in CAF.
    fn from(segment: String) -> Self
    {
        let mut original = Vec::with_capacity(segment.len());
        // escape_default will insert escapes for all ASCII control characters (e.g. \n) and Unicode characters.
        for c in segment.chars().flat_map(|c| c.escape_default()) {
            let len = original.len();
            let size = c.len_utf8();
            original.resize(len + size, 0u8);
            c.encode_utf8(&mut original[len..(len + size)]);
        }

        Self { leading_spaces: 0, original, segment }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafString
{
    pub fill: CafFill,
    /// Note: if you manually insert segments then you need to manually construct the cached aggregate string.
    pub segments: SmallVec<[CafStringSegment; 1]>,
    /// Caches the full string if there are multiple segments.
    pub cached: Option<String>,
}

impl CafString
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(
        &self,
        writer: &mut impl RawSerializer,
        space: impl AsRef<str>,
    ) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write_bytes("\"".as_bytes())?;
        let num_segments = self.segments.len();
        for (idx, segment) in self.segments.iter().enumerate() {
            segment.write_to(writer)?;
            if num_segments > 1 && idx + 1 < num_segments {
                writer.write_bytes("\\\n".as_bytes())?;
            }
        }
        writer.write_bytes("\"".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        if peek(char::<_, ()>('"')).parse(content).is_err() {
            return Ok((None, fill, content));
        }

        // Parse segments
        let (remaining, mut segments) = parse_string(content)?;

        // Cache all segments concatenated
        if segments.len() == 0 {
            segments.push(CafStringSegment::default());
        }
        let cached = if segments.len() > 1 {
            Some(String::from_iter(segments.iter().map(|s| s.segment.as_str())))
        } else {
            None
        };

        let (next_fill, remaining) = CafFill::parse(remaining);
        Ok((Some(Self { fill, segments, cached }), next_fill, remaining))
    }

    pub fn as_str(&self) -> &str
    {
        if self.segments.len() == 1 {
            self.segments[0].segment.as_str()
        } else if let Some(cached) = &self.cached {
            cached.as_str()
        } else {
            ""
        }
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
        for (idx, (segment, other_segment)) in self
            .segments
            .iter_mut()
            .zip(other.segments.iter())
            .enumerate()
        {
            if idx == 0 {
                segment.leading_spaces = 0;
            } else {
                segment.leading_spaces = other_segment.leading_spaces;
            }
        }
    }
}

impl From<char> for CafString
{
    fn from(character: char) -> Self
    {
        Self {
            fill: CafFill::default(),
            segments: SmallVec::from_elem(CafStringSegment::from(character), 1),
            cached: None,
        }
    }
}

impl From<&str> for CafString
{
    fn from(string: &str) -> Self
    {
        Self {
            fill: CafFill::default(),
            segments: SmallVec::from_elem(CafStringSegment::from(string), 1),
            cached: None,
        }
    }
}

impl From<String> for CafString
{
    fn from(string: String) -> Self
    {
        Self {
            fill: CafFill::default(),
            segments: SmallVec::from_elem(CafStringSegment::from(string), 1),
            cached: None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

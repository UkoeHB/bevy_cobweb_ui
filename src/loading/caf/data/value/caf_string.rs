use smallvec::SmallVec;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
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

/*
Parsing:
- copy serde_json string deserialization logic to read string segments
    - see: parse_str_bytes, is_escape, as_str -> from_utf8(), parse_escape
    - need to test if this will properly handle string segments that contain raw newlines
    - should error if something unknown is escaped (such as whitespace or a comment)
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafString
{
    pub fill: CafFill,
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

    // TODO: recover leading spaces for multi-line text? what if the lines change?
    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
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

/*
Parsing:
- segments end in [\\][\n] or non-escaped-"
- if multiple segments, they must be collected into the cached string
*/

//-------------------------------------------------------------------------------------------------------------------

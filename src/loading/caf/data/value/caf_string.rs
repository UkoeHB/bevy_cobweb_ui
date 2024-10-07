use std::io::Cursor;

use bevy::reflect::{TypeInfo, TypeRegistry};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafStringSegment
{
    /// Spaces at the start of a segment for multiline text.
    pub leading_spaces: usize,
    /// The originally-parsed bytes, cached for writing back to raw bytes.
    ///
    /// We do this as a shortcut to avoid extra logic for serializing a normal string. One way would be
    /// using `serde_json::ser::write_to()` to a string, but the problem is needing to crop the first and last
    /// characters which will be `"` (since we are potentially multi-line). There is also a problem for strings
    /// that contain both explicit `\n` newlines and also implicit newlines from being on multiple lines. That
    /// distinction is lost once the string is fully converted. Note that it's possible editors can't support
    /// a mix of explicit and implicit newlines, and they will just have to aggressively replace implicit newlines
    /// with explicit newlines.
    //todo: replace this with a shared memory structure like Bytes or Cow<[u8]>?
    pub original: Vec<u8>,
    pub segment: String,
}

impl CafStringSegment
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        for _ in 0..self.leading_spaces {
            writer.write(' '.as_bytes())?;
        }
        // Here we write raw bytes.
        writer.write(&self.original)?;
        Ok(())
    }

    pub fn write_to_json(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        // Here we write as a deserialized string.
        writer.write(self.segment.as_bytes())?;
        Ok(())
    }

    /// Note that `Self::write_to_json` -> `Self::from_json_string` is lossy because JSON has no awareness of
    /// multi-line string formatting. The string contents are preserved, but not their presentation in CAF.
    pub fn from_json_string(json_str: &str) -> Result<Self, String>
    {
        let mut original = Vec::with_capacity(json_str.len());
        let mut cursor = Cursor::new(&mut original);
        format_escaped_str_contents(&mut cursor, json_str).map_err(|e| format!("{e:?}"))?;

        Ok(Self {
            leading_spaces: 0,
            original,
            segment: String::from(json_str.as_str()),
        })
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
}

impl CafString
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(
        &self,
        writer: &mut impl std::io::Write,
        space: impl AsRef<str>,
    ) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write('"'.as_bytes())?;
        let num_segments = self.segments.len();
        for (idx, segment) in self.segments.iter().enumerate() {
            segment.write_to(writer)?;
            if num_segments > 1 && idx + 1 < num_segments {
                writer.write("\\\n".as_bytes())?;
            }
        }
        writer.write('"'.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut string = String::default();
        let mut cursor = Cursor::new(&mut string);
        for segment in self.segments.iter() {
            segment.write_to_json(&mut cursor)?;
        }
        Ok(serde_json::Value::String(string))
    }

    /// Note that `Self::to_json` -> `Self::from_json_string` is lossy because JSON has no awareness of multi-line
    /// string formatting. The string contents are preserved, but not their presentation in CAF.
    pub fn from_json_string(json_str: &String) -> Result<Self, String>
    {
        Ok(Self {
            fill: CafFill::default(),
            segments: SmallVec::from_elem(CafStringSegment::from_json_string(json_str)?, 1),
        })
    }

    // TODO: recover leading spaces for multi-line text? what if the lines change?
    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

/*
Parsing:
- segments end in [\\][\n] or non-escaped-"
*/

//-------------------------------------------------------------------------------------------------------------------

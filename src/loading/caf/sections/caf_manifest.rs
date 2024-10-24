use std::sync::Arc;

use bevy::prelude::{default, Deref};
use nom::bytes::complete::tag;
use nom::combinator::recognize;
use nom::multi::many0_count;
use nom::sequence::{preceded, tuple};
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafManifestFile
{
    SelfRef,
    File(CafFilePath),
}

impl CafManifestFile
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::SelfRef => {
                writer.write_bytes("self".as_bytes())?;
            }
            Self::File(file) => {
                file.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        // Case: self
        if let Ok((remaining, _)) = tag::<_, _, ()>("self").parse(content) {
            return Ok((Self::SelfRef, remaining));
        }

        // Case: string file path
        let (file, remaining) = CafFilePath::parse(content)?;
        Ok((Self::File(file), remaining))
    }
}

impl Default for CafManifestFile
{
    fn default() -> Self
    {
        Self::File(CafFilePath::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafManifestKey(pub Arc<str>);

impl CafManifestKey
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        recognize(tuple((
            // Base identifier
            snake_identifier,
            // Extensions
            many0_count(preceded(tag("."), snake_identifier)),
        )))
        .parse(content)
        .map(|(r, k)| (Self(Arc::from(*k.fragment())), r))
    }
}

impl Default for CafManifestKey
{
    fn default() -> Self
    {
        Self(Arc::from(""))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// {file} as {key}
#[derive(Debug, Clone, PartialEq)]
pub struct CafManifestEntry
{
    pub entry_fill: CafFill,
    pub file: CafManifestFile,
    pub as_fill: CafFill,
    pub key_fill: CafFill,
    pub key: CafManifestKey,
}

impl CafManifestEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.entry_fill.write_to_or_else(writer, "\n")?;
        self.file.write_to(writer)?;
        self.as_fill.write_to_or_else(writer, " ")?;
        writer.write_bytes("as".as_bytes())?;
        self.key_fill.write_to_or_else(writer, " ")?;
        self.key.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(entry_fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((file, remaining)) = CafManifestFile::parse(content) else {
            return Ok((None, entry_fill, content));
        };
        if !entry_fill.ends_with_newline() {
            tracing::warn!("manifest entry doesn't start on a new line at {}", get_location(content).as_str());
            return Err(span_verify_error(content));
        }
        let (as_fill, remaining) = CafFill::parse(remaining);
        if as_fill.len() == 0 {
            tracing::warn!("no fill/whitespace before manifest 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (remaining, _) = tag("as").parse(remaining)?;
        let (key_fill, remaining) = CafFill::parse(remaining);
        if key_fill.len() == 0 {
            tracing::warn!("no fill/whitespace after manifest 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (key, remaining) = CafManifestKey::parse(remaining)?;
        let (next_fill, remaining) = CafFill::parse(remaining);
        Ok((
            Some(Self { entry_fill, file, as_fill, key_fill, key }),
            next_fill,
            remaining,
        ))
    }

    // Makes a new entry with default spacing.
    pub fn new(file: impl AsRef<str>, key: impl AsRef<str>) -> Self
    {
        Self {
            file: CafManifestFile::File(CafFilePath(Arc::from(file.as_ref()))),
            key: CafManifestKey(Arc::from(key.as_ref())),
            ..default()
        }
    }
}

impl Default for CafManifestEntry
{
    fn default() -> Self
    {
        Self {
            entry_fill: CafFill::new("\n"),
            file: Default::default(),
            as_fill: CafFill::new(" "),
            key_fill: CafFill::new(" "),
            key: CafManifestKey(Arc::from("")),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafManifest
{
    pub start_fill: CafFill,
    pub entries: Vec<CafManifestEntry>,
}

impl CafManifest
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("#manifest".as_bytes())?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    pub fn try_parse(start_fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#manifest").parse(content) else {
            return Ok((None, start_fill, content));
        };

        let (mut item_fill, mut remaining) = CafFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            match CafManifestEntry::try_parse(item_fill, remaining)? {
                (Some(entry), next_fill, after_entry) => {
                    entries.push(entry);
                    item_fill = next_fill;
                    remaining = after_entry;
                }
                (None, end_fill, after_end) => {
                    remaining = after_end;
                    break end_fill;
                }
            }
        };

        let manifest = Self { start_fill, entries };
        Ok((Some(manifest), end_fill, remaining))
    }
}

impl Default for CafManifest
{
    fn default() -> Self
    {
        Self { start_fill: CafFill::default(), entries: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

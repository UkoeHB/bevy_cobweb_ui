use std::sync::Arc;

use bevy::prelude::Deref;
use nom::bytes::complete::tag;
use nom::combinator::recognize;
use nom::multi::many0_count;
use nom::sequence::{preceded, tuple};
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobManifestFile
{
    SelfRef,
    File(CobFile),
}

impl CobManifestFile
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
        let (file, remaining) = CobFile::parse(content)?;
        Ok((Self::File(file), remaining))
    }
}

impl Default for CobManifestFile
{
    fn default() -> Self
    {
        Self::File(CobFile::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents a manifest key pointing to a cobweb asset file. Manifest keys must be registered with a `#manifest`
/// section in a cobweb asset file.
///
/// Example: `builtin.widgets.radio_button` for a pre-registered radio button COB file.
#[derive(Debug, Clone, Eq, PartialEq, Deref, Hash)]
pub struct ManifestKey(pub Arc<str>);

impl ManifestKey
{
    /// Creates a new COB manifest key.
    pub fn new(key: impl AsRef<str>) -> Self
    {
        Self(Arc::from(key.as_ref()))
    }

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

    pub fn as_str(&self) -> &str
    {
        &self.0
    }
}

impl Default for ManifestKey
{
    fn default() -> Self
    {
        Self(Arc::from(""))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// {file} as {key}
#[derive(Debug, Clone, PartialEq)]
pub struct CobManifestEntry
{
    pub entry_fill: CobFill,
    pub file: CobManifestFile,
    pub as_fill: CobFill,
    pub key_fill: CobFill,
    pub key: ManifestKey,
}

impl CobManifestEntry
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

    pub fn try_parse(entry_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((file, remaining)) = CobManifestFile::parse(content) else {
            return Ok((None, entry_fill, content));
        };
        if !entry_fill.ends_with_newline() {
            tracing::warn!("manifest entry doesn't start on a new line at {}", get_location(content).as_str());
            return Err(span_verify_error(content));
        }
        let (as_fill, remaining) = CobFill::parse(remaining);
        if as_fill.len() == 0 {
            tracing::warn!("no fill/whitespace before manifest 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (remaining, _) = tag("as").parse(remaining)?;
        let (key_fill, remaining) = CobFill::parse(remaining);
        if key_fill.len() == 0 {
            tracing::warn!("no fill/whitespace after manifest 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (key, remaining) = ManifestKey::parse(remaining)?;
        let (next_fill, remaining) = CobFill::parse(remaining);
        Ok((
            Some(Self { entry_fill, file, as_fill, key_fill, key }),
            next_fill,
            remaining,
        ))
    }
}

impl Default for CobManifestEntry
{
    fn default() -> Self
    {
        Self {
            entry_fill: CobFill::new("\n"),
            file: Default::default(),
            as_fill: CobFill::new(" "),
            key_fill: CobFill::new(" "),
            key: ManifestKey(Arc::from("")),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobManifest
{
    pub start_fill: CobFill,
    pub entries: Vec<CobManifestEntry>,
}

impl CobManifest
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

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#manifest").parse(content) else {
            return Ok((None, start_fill, content));
        };

        if start_fill.len() != 0 && !start_fill.ends_with_newline() {
            tracing::warn!("failed parsing manifest section at {} that doesn't start on newline",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            match CobManifestEntry::try_parse(item_fill, remaining)? {
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

impl Default for CobManifest
{
    fn default() -> Self
    {
        Self { start_fill: CobFill::default(), entries: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

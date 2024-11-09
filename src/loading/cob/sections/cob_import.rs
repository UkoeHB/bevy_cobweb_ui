use std::sync::Arc;

use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::recognize;
use nom::multi::many0_count;
use nom::sequence::{preceded, tuple};
use nom::Parser;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobImportAlias
{
    None,
    Alias(SmolStr),
}

impl CobImportAlias
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::None => {
                writer.write_bytes("_".as_bytes())?;
            }
            Self::Alias(alias) => {
                writer.write_bytes(alias.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        // Case: no alias
        if let Ok((remaining, _)) = char::<_, ()>('_').parse(content) {
            return Ok((Self::None, remaining));
        }

        // Case: alias
        recognize(tuple((
            // Base identifier
            snake_identifier,
            // Extensions
            many0_count(preceded(tag("::"), snake_identifier)),
        )))
        .parse(content)
        .map(|(r, k)| (Self::Alias(SmolStr::from(*k.fragment())), r))
    }

    pub fn as_str(&self) -> &str
    {
        match self {
            CobImportAlias::None => "",
            CobImportAlias::Alias(alias) => alias.as_str(),
        }
    }
}

impl Default for CobImportAlias
{
    fn default() -> Self
    {
        Self::None
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// {manifest key} as {alias}
#[derive(Debug, Clone, PartialEq)]
pub struct CobImportEntry
{
    pub entry_fill: CobFill,
    pub key: ManifestKey,
    pub as_fill: CobFill,
    pub alias_fill: CobFill,
    pub alias: CobImportAlias,
}

impl CobImportEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.entry_fill.write_to_or_else(writer, "\n")?;
        self.key.write_to(writer)?;
        self.as_fill.write_to_or_else(writer, " ")?;
        writer.write_bytes("as".as_bytes())?;
        self.alias_fill.write_to_or_else(writer, " ")?;
        self.alias.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(entry_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((key, remaining)) = ManifestKey::parse(content) else {
            return Ok((None, entry_fill, content));
        };
        if !entry_fill.ends_with_newline() {
            tracing::warn!("import entry doesn't start on a new line at {}", get_location(content).as_str());
            return Err(span_verify_error(content));
        }
        let (as_fill, remaining) = CobFill::parse(remaining);
        if as_fill.len() == 0 {
            tracing::warn!("no fill/whitespace before import 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (remaining, _) = tag("as").parse(remaining)?;
        let (alias_fill, remaining) = CobFill::parse(remaining);
        if alias_fill.len() == 0 {
            tracing::warn!("no fill/whitespace after import 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (alias, remaining) = CobImportAlias::parse(remaining)?;
        let (next_fill, remaining) = CobFill::parse(remaining);
        Ok((
            Some(Self { entry_fill, key, as_fill, alias_fill, alias }),
            next_fill,
            remaining,
        ))
    }

    // Makes a new entry with default spacing.
    pub fn new(key: impl AsRef<str>, alias: impl AsRef<str>) -> Self
    {
        Self {
            key: ManifestKey(Arc::from(key.as_ref())),
            alias: CobImportAlias::Alias(SmolStr::from(alias.as_ref())),
            ..Default::default()
        }
    }
}

impl Default for CobImportEntry
{
    fn default() -> Self
    {
        Self {
            entry_fill: CobFill::new("\n"),
            key: Default::default(),
            as_fill: CobFill::new(" "),
            alias_fill: CobFill::new(" "),
            alias: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CobImport
{
    pub start_fill: CobFill,
    pub entries: Vec<CobImportEntry>,
}

impl CobImport
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("#import".as_bytes())?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#import").parse(content) else {
            return Ok((None, start_fill, content));
        };

        if start_fill.len() != 0 && !start_fill.ends_with_newline() {
            tracing::warn!("failed parsing import section at {} that doesn't start on newline",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            match CobImportEntry::try_parse(item_fill, remaining)? {
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

        let import = Self { start_fill, entries };
        Ok((Some(import), end_fill, remaining))
    }
}

//-------------------------------------------------------------------------------------------------------------------

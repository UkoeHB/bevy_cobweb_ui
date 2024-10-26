use std::sync::Arc;

use bevy::prelude::Deref;
use nom::bytes::complete::tag;
use nom::combinator::recognize;
use nom::multi::many0_count;
use nom::sequence::terminated;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafUsingTypePath
{
    pub path_prefix: Arc<str>,
    /// Note that only instruction types need to be looked up in the type registry, so there is no need to handle
    /// rust primitives or tuples here.
    pub id: CafInstructionIdentifier,
}

impl CafUsingTypePath
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.path_prefix.as_bytes())?;
        self.id.write_to(writer)?;
        Ok(())
    }

    pub fn to_canonical(&self, scratch: Option<String>) -> String
    {
        let mut canonical = self.id.to_canonical(scratch);
        canonical.insert_str(0, &*self.path_prefix);
        canonical
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (remaining, prefix) =
            recognize(many0_count(terminated(snake_identifier, tag("::")))).parse(content)?;
        let (id, remaining) = CafInstructionIdentifier::parse(remaining)?;
        if !id.is_resolved() {
            tracing::warn!("failed parsing using type path at {}; path generics are not fully resolved {:?}",
                get_location(content).as_str(), id);
            return Err(span_verify_error(content));
        }
        Ok((Self { path_prefix: Arc::from(*prefix.fragment()), id }, remaining))
    }
}

impl Default for CafUsingTypePath
{
    fn default() -> Self
    {
        Self { path_prefix: Arc::from(""), id: Default::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafUsingIdentifier(pub CafInstructionIdentifier);

impl CafUsingIdentifier
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.0.write_to(writer)?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (id, remaining) = CafInstructionIdentifier::parse(content)?;
        if !id.is_resolved() {
            tracing::warn!("failed parsing using identifier at {}; id generics are not fully resolved {:?}",
                get_location(content).as_str(), id);
            return Err(span_verify_error(content));
        }
        Ok((Self(id), remaining))
    }
}

impl Default for CafUsingIdentifier
{
    fn default() -> Self
    {
        Self(Default::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// {file} as {alias}
#[derive(Debug, Clone, PartialEq)]
pub struct CafUsingEntry
{
    pub entry_fill: CafFill,
    pub type_path: CafUsingTypePath,
    pub as_fill: CafFill,
    pub identifier_fill: CafFill,
    pub identifier: CafUsingIdentifier,
}

impl CafUsingEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.entry_fill.write_to_or_else(writer, "\n")?;
        self.type_path.write_to(writer)?;
        self.as_fill.write_to_or_else(writer, " ")?;
        writer.write_bytes("as".as_bytes())?;
        self.identifier_fill.write_to_or_else(writer, " ")?;
        self.identifier.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(entry_fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((type_path, remaining)) = CafUsingTypePath::parse(content) else {
            return Ok((None, entry_fill, content));
        };
        if !entry_fill.ends_with_newline() {
            tracing::warn!("using entry doesn't start on a new line at {}", get_location(content).as_str());
            return Err(span_verify_error(content));
        }
        let (as_fill, remaining) = CafFill::parse(remaining);
        if as_fill.len() == 0 {
            tracing::warn!("no fill/whitespace before using 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (remaining, _) = tag("as").parse(remaining)?;
        let (identifier_fill, remaining) = CafFill::parse(remaining);
        if identifier_fill.len() == 0 {
            tracing::warn!("no fill/whitespace after using 'as' at {}", get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }
        let (identifier, remaining) = CafUsingIdentifier::parse(remaining)?;
        let (next_fill, remaining) = CafFill::parse(remaining);
        Ok((
            Some(Self { entry_fill, type_path, as_fill, identifier_fill, identifier }),
            next_fill,
            remaining,
        ))
    }
}

impl Default for CafUsingEntry
{
    fn default() -> Self
    {
        Self {
            entry_fill: CafFill::new("\n"),
            type_path: Default::default(),
            as_fill: CafFill::new(" "),
            identifier_fill: CafFill::new(" "),
            identifier: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafUsing
{
    pub start_fill: CafFill,
    pub entries: Vec<CafUsingEntry>,
}

impl CafUsing
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("#using".as_bytes())?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    pub fn try_parse(start_fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#using").parse(content) else {
            return Ok((None, start_fill, content));
        };

        if start_fill.len() != 0 && !start_fill.ends_with_newline() {
            tracing::warn!("failed parsing using section at {} that doesn't start on newline",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        let (mut item_fill, mut remaining) = CafFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            match CafUsingEntry::try_parse(item_fill, remaining)? {
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

        let using = Self { start_fill, entries };
        Ok((Some(using), end_fill, remaining))
    }
}

impl Default for CafUsing
{
    fn default() -> Self
    {
        Self { start_fill: CafFill::default(), entries: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::sync::Arc;

use bevy::prelude::{default, Deref};
use nom::bytes::complete::tag;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafUsingTypePath(pub Arc<str>);

impl CafUsingTypePath
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.as_bytes())?;
        Ok(())
    }
}

impl Default for CafUsingTypePath
{
    fn default() -> Self
    {
        Self(Arc::from(""))
    }
}

/*
Parsing: no parsing? how to evaluate this?
*/

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
}

impl Default for CafUsingIdentifier
{
    fn default() -> Self
    {
        Self(Default::default())
    }
}

/*
Parsing:
- The identifier's generics must be fully known (no macro params, constants, or macro calls).
*/

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

    // Makes a new entry with default spacing.
    pub fn new(file: impl AsRef<str>, identifier: CafInstructionIdentifier) -> Self
    {
        Self {
            type_path: CafUsingTypePath(Arc::from(file.as_ref())),
            identifier: CafUsingIdentifier(identifier),
            ..default()
        }
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

/*
Parsing:
- Must start with newline.
- Must be 'file as alias'.
- SceneFile parsing should use CafManifestKey to parse manifest keys.
*/

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

        // TODO

        let using = CafUsing { start_fill, entries: vec![] };
        Ok((Some(using), CafFill::default(), remaining))
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

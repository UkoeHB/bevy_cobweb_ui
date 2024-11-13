use nom::bytes::complete::tag;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobDefEntry
{
    Constant(CobConstantDef),
    DataMacro(CobDataMacroDef),
    LoadableMacro(CobLoadableMacroDef),
    SceneMacro(CobSceneMacroDef),
}

impl CobDefEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Constant(entry) => {
                entry.write_to(writer)?;
            }
            Self::DataMacro(entry) => {
                entry.write_to(writer)?;
            }
            Self::LoadableMacro(entry) => {
                entry.write_to(writer)?;
            }
            Self::SceneMacro(entry) => {
                entry.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let starts_newline = fill.ends_with_newline();
        let check_newline = || -> Result<(), SpanError> {
            if !starts_newline {
                tracing::warn!("def entry doesn't start on a new line at {}", get_location(content).as_str());
                return Err(span_verify_error(content));
            }
            Ok(())
        };
        let fill = match rc(content, move |c| CobConstantDef::try_parse(fill, c))? {
            (Some(def), next_fill, remaining) => {
                (check_newline)()?;
                return Ok((Some(Self::Constant(def)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobDataMacroDef::try_parse(fill, c))? {
            (Some(def), next_fill, remaining) => {
                (check_newline)()?;
                return Ok((Some(Self::DataMacro(def)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobLoadableMacroDef::try_parse(fill, c))? {
            (Some(def), next_fill, remaining) => {
                (check_newline)()?;
                return Ok((Some(Self::LoadableMacro(def)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobSceneMacroDef::try_parse(fill, c))? {
            (Some(def), next_fill, remaining) => {
                (check_newline)()?;
                return Ok((Some(Self::SceneMacro(def)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Includes constants and macros. A constant is equivalent to a macro with no parameters.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CobDefs
{
    pub start_fill: CobFill,
    pub entries: Vec<CobDefEntry>,
}

impl CobDefs
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#defs").parse(content) else {
            return Ok((None, start_fill, content));
        };

        if start_fill.len() != 0 && !start_fill.ends_with_newline() {
            tracing::warn!("failed parsing defs section at {} that doesn't start on newline",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            match rc(remaining, move |rm| CobDefEntry::try_parse(item_fill, rm))? {
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

        let defs = CobDefs { start_fill, entries };
        Ok((Some(defs), end_fill, remaining))
    }
}

//-------------------------------------------------------------------------------------------------------------------

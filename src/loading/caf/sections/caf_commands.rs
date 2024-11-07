use nom::bytes::complete::tag;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Commands are parsed as loadables.
#[derive(Debug, Clone, PartialEq)]
pub enum CafCommandEntry
{
    Loadable(CafLoadable),
    LoadableMacroCall(CafLoadableMacroCall),
}

impl CafCommandEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Loadable(loadable) => {
                loadable.write_to(writer)?;
            }
            Self::LoadableMacroCall(call) => {
                call.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let starts_newline = fill.ends_with_newline();
        let fill = match rc(content, move |c| CafLoadable::try_parse(fill, c))? {
            (Some(loadable), next_fill, remaining) => {
                if !starts_newline {
                    tracing::warn!("command entry doesn't start on a new line at {}", get_location(content).as_str());
                    return Err(span_verify_error(content));
                }
                // NOTE: macro params are not allowed in commands but we don't check here to avoid the perf cost
                // of traversing the structure. Allow errors to be detected downstream (e.g. when deserializing).
                // TODO: re-evaluate if this is useful; the perf cost of traversing everything again is
                // non-negligible
                return Ok((Some(Self::Loadable(loadable)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafLoadableMacroCall::try_parse(fill, c))? {
            (Some(call), next_fill, remaining) => {
                if !starts_newline {
                    tracing::warn!("command entry doesn't start on a new line at {}", get_location(content).as_str());
                    return Err(span_verify_error(content));
                }
                return Ok((Some(Self::LoadableMacroCall(call)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafCommands
{
    pub start_fill: CafFill,
    pub entries: Vec<CafCommandEntry>,
}

impl CafCommands
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("#commands".as_bytes())?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    pub fn try_parse(start_fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#commands").parse(content) else {
            return Ok((None, start_fill, content));
        };

        if start_fill.len() != 0 && !start_fill.ends_with_newline() {
            tracing::warn!("failed parsing commands section at {} that doesn't start on newline",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        let (mut item_fill, mut remaining) = CafFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            match rc(remaining, move |rm| CafCommandEntry::try_parse(item_fill, rm))? {
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

        let command = Self { start_fill, entries };
        Ok((Some(command), end_fill, remaining))
    }
}

impl Default for CafCommands
{
    fn default() -> Self
    {
        Self { start_fill: CafFill::default(), entries: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

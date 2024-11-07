use nom::bytes::complete::tag;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafDefEntry
{
    Constant(CafConstantDef),
    DataMacro(CafDataMacroDef),
    LoadableMacro(CafLoadableMacroDef),
    SceneMacro(CafSceneMacroDef),
}

impl CafDefEntry
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
}

//-------------------------------------------------------------------------------------------------------------------

/// Includes constants and macros. A constant is equivalent to a macro with no parameters.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafDefs
{
    pub start_fill: CafFill,
    pub defs: Vec<CafDefEntry>,
}

impl CafDefs
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        for def in self.defs.iter() {
            def.write_to(writer)?;
        }
        Ok(())
    }

    pub fn try_parse(start_fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#defs").parse(content) else {
            return Ok((None, start_fill, content));
        };

        if start_fill.len() != 0 && !start_fill.ends_with_newline() {
            tracing::warn!("failed parsing defs section at {} that doesn't start on newline",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        // TODO (with recursion testing)

        let defs = CafDefs { start_fill, defs: vec![] };
        Ok((Some(defs), CafFill::default(), remaining))
    }

    pub fn eq_ignore_whitespace(&self, _other: &CafDefs) -> bool
    {
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

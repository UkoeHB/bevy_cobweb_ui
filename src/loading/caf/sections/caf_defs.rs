use nom::bytes::complete::tag;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafDefEntry
{
    Constant(CafConstantDef),
    DataMacro(CafDataMacroDef),
    InstructionMacro(CafInstructionMacroDef),
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
            Self::InstructionMacro(entry) => {
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

    pub fn try_parse(
        content: Span,
        fill: CafFill,
    ) -> Result<(Option<Self>, CafFill, Span), nom::error::Error<Span>>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#defs").parse(content) else { return Ok((None, fill, content)) };

        // TODO

        let defs = CafDefs { start_fill: fill, defs: vec![] };
        Ok((Some(defs), CafFill::default(), remaining))
    }

    pub fn eq_ignore_whitespace(&self, _other: &CafDefs) -> bool
    {
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

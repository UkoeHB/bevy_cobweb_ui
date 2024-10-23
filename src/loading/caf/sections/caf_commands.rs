use nom::bytes::complete::tag;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Full instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum CafCommandEntry
{
    Instruction(CafInstruction),
    InstructionMacroCall(CafInstructionMacroCall),
}

impl CafCommandEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Instruction(instruction) => {
                instruction.write_to(writer)?;
            }
            Self::InstructionMacroCall(call) => {
                call.write_to(writer)?;
            }
        }
        Ok(())
    }
}

/*
Parsing:
- Entry cannot contain macro params.
*/

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

    pub fn try_parse(
        content: Span,
        fill: CafFill,
    ) -> Result<(Option<Self>, CafFill, Span), nom::error::Error<Span>>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#commands").parse(content) else {
            return Ok((None, fill, content));
        };

        // TODO

        let commands = CafCommands { start_fill: fill, entries: vec![] };
        Ok((Some(commands), CafFill::default(), remaining))
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

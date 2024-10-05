use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Full instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum CafCommandEntry
{
    Instruction(CafInstruction),
    InstructionMacro(CafInstructionMacro),
}

impl CafCommandEntry
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Instruction(instruction) => {
                instruction.write_to(writer)?;
            }
            Self::InstructionMacro(instruction) => {
                instruction.write_to(writer)?;
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
    pub fn write_to(&self, first_section: bool, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write("#commands".as_bytes)?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
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

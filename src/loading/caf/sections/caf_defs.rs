use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafDefEntry
{
    Constant(CafConstantDef),
    DataMacro(CafDataMacroDef),
    InstructionMacro(CafInstructionMacroDef),
    SceneMacro(SceneMacroDef),
}

impl CafDefEntry
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
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
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        for def in self.defs.iter() {
            defs.write_to(writer)?;
        }
    }

    pub fn eq_ignore_whitespace(&self, _other: &CafDefs) -> bool
    {
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

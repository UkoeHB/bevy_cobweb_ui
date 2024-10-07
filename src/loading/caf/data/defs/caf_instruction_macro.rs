// CafInstructionMacroCall
// CafInstructionMacroDef
// - Flatten group or single instruction only

// def must start at beginning of line

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafInstructionMacroCall;

impl CafInstructionMacroCall
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafInstructionMacroDef;

impl CafInstructionMacroDef
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

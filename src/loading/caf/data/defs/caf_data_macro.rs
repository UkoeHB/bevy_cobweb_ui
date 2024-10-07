// CafDataMacroCall
// CafDataMacroDef

// def must start at beginning of line

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafDataMacroCall;

impl CafDataMacroCall
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafDataMacroDef;

impl CafDataMacroDef
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

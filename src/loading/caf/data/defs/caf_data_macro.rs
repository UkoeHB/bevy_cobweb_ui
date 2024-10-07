// CafDataMacroCall
// CafDataMacroDef

// def must start at beginning of line

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafDataMacroCall;

impl CafDataMacroCall
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(
        &self,
        _writer: &mut impl std::io::Write,
        _space: &str,
    ) -> Result<(), std::io::Error>
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
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(
        &self,
        _writer: &mut impl std::io::Write,
        _space: &str,
    ) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

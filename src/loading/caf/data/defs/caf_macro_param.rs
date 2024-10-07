// CafMacroParam
// - @, ?, ..
// - Param id potentially nested.
// CafMacroParamDef
// - Unassigned
// - Assigned
// - Nested
// - Catch-all into flatten group
// - type params for generics: use ^param notation without whitespace, cannot be assigned (non-optional)

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafMacroParam;

impl CafMacroParam
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
pub struct CafMacroParamDef;

impl CafMacroParamDef
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

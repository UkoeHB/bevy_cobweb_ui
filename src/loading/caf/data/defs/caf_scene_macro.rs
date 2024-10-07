// CafSceneMacroCall
// CafSceneMacroDef
// - Flatten group only
// - Scene layer
// CafSceneMacroParam
// CafSceneMacroParamDef

// How to capture special catch-all syntax '..*' ?

// def must start at beginning of line

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafSceneMacroCall;

impl CafSceneMacroCall
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafSceneMacroDef;

impl CafSceneMacroDef
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafSceneMacroParam;

impl CafSceneMacroParam
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafSceneMacroParamDef;

impl CafSceneMacroParamDef
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

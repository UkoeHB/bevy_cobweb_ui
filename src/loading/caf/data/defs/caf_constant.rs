// CafConstant
// CafConstantDef

// def must start at beginning of line

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafConstant;

impl CafConstant
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafConstantDef;

impl CafConstantDef
{
    pub fn write_to(&self, _writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------

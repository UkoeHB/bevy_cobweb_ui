use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafScene
{
    pub start_fill: CafFill,
}

impl CafScene
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        Ok(())
    }

    pub fn eq_ignore_whitespace(&self, _other: &CafScene) -> bool
    {
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

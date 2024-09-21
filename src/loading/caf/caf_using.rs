use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafUsing
{
    pub start_fill: CafFill,
}

impl CafUsing
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        Ok(())
    }

    pub fn eq_ignore_whitespace(&self, _other: &CafUsing) -> bool
    {
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

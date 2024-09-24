use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

// Distinguish between instruction defs and data defs.

/// Includes constants and macros. A constant is equivalent to a macro with no parameters.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafDefs
{
    pub start_fill: CafFill,
}

impl CafDefs
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)
    }

    pub fn eq_ignore_whitespace(&self, _other: &CafDefs) -> bool
    {
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

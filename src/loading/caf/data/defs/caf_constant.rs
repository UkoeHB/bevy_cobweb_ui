// CafConstant
// CafConstantDef

// def must start at beginning of line

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafConstant;

impl CafConstant
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, _writer: &mut impl RawSerializer, _space: &str)
        -> Result<(), std::io::Error>
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
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, _writer: &mut impl RawSerializer, _space: &str)
        -> Result<(), std::io::Error>
    {
        Ok(())
    }

    pub fn recover_fill(&mut self, _other: &Self) {}
}

//-------------------------------------------------------------------------------------------------------------------


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafInstruction(pub CafStruct);

impl CafInstruction
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.0.write_to(writer)?;
        Ok(())
    }
}

/*
Parsing:

*/

//-------------------------------------------------------------------------------------------------------------------

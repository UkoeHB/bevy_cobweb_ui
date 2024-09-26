

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafBool
{
    pub fill: CafFill,
    pub value: bool
}

impl CafBool
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.fill.write_to(writer)?;
        let string = match *self.value {
            true => "true",
            false => "false",
        };
        writer.write(string.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        Ok(serde_json::Value::Bool(self.value))
    }
}

/*
Parsing:
- parse as string
*/

//-------------------------------------------------------------------------------------------------------------------

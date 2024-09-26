

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafArray
{
    /// Fill before opening `[`.
    pub start_fill: CafFill,
    pub entries: Vec<CafValue>,
    /// Fill before ending `]`.
    pub end_fill: CafFill
}

impl CafArray
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write('['.as_bytes())?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        self.end_fill.write_to(writer)?;
        writer.write(']'.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut array = Vec::with_capacity(self.entries.len());
        for entry in self.entries.iter() {
            array.push(entry.to_json()?);
        }
        Ok(serde_json::Value::Array(array))
    }
}

/*
Parsing:
*/

//-------------------------------------------------------------------------------------------------------------------

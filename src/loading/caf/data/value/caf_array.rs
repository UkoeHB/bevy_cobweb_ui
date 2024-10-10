use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafArray
{
    /// Fill before opening `[`.
    pub start_fill: CafFill,
    pub entries: Vec<CafValue>,
    /// Fill before ending `]`.
    pub end_fill: CafFill,
}

impl CafArray
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str)
        -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write("[".as_bytes())?;
        for (idx, entry) in self.entries.iter().enumerate() {
            if idx == 0 {
                entry.write_to(writer)?;
            } else {
                entry.write_to_with_space(writer, " ")?;
            }
        }
        self.end_fill.write_to(writer)?;
        writer.write("]".as_bytes())?;
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

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        for (entry, other_entry) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other_entry);
        }
        self.end_fill.recover(&other.end_fill);
    }
}

impl From<Vec<CafValue>> for CafArray
{
    fn from(entries: Vec<CafValue>) -> Self
    {
        Self {
            start_fill: CafFill::default(),
            entries,
            end_fill: CafFill::default(),
        }
    }
}

/*
Parsing:
*/

//-------------------------------------------------------------------------------------------------------------------

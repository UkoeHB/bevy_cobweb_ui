


//-------------------------------------------------------------------------------------------------------------------

/// Stores the original number value string and also the number deserialized from JSON.
///
/// We need to keep the original so it can be reserialized in the correct format.
///
/// We store a JSON value for convenience instead of implementing our own deserialization routine.
#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafNumberValue
{
    pub original: Arc<str>,
    pub deserialized: serde_json::Number,
}

impl CafNumberValue
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.original.write_to(writer)?;
        Ok(())
    }
}

/*
Parsing:
- Use regex to identify number, then parse it using a JSON deserializer with serde_json::Number::from_str().
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafNumber
{
    pub fill: CafFill,
    pub number: CafNumberValue
}

impl CafNumber
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.fill.write_to(writer)?;
        self.number.write_to(writer)?;
        Ok(())
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

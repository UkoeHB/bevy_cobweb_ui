use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Stores the original number value string and also the number deserialized from JSON.
///
/// We need to keep the original so it can be reserialized in the correct format.
///
/// We store a JSON value for convenience instead of implementing our own deserialization routine.
#[derive(Debug, Clone, PartialEq)]
pub struct CafNumberValue
{
    pub original: SmolStr,
    pub deserialized: serde_json::Number,
}

impl CafNumberValue
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write(self.original.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        Ok(serde_json::Value::Number(self.deserialized.clone()))
    }

    pub fn from_json_number(json_num: serde_json::Number) -> Self
    {
        let string = if let Some(value) = json_num.as_u64() {
            let mut buffer = itoa::Buffer::new();
            SmolStr::from(buffer.format(value))
        } else if let Some(value) = json_num.as_f64() {
            let mut buffer = ryu::Buffer::new();
            SmolStr::from(buffer.format_finite(value))
        } else if let Some(value) = json_num.as_i64() {
            let mut buffer = itoa::Buffer::new();
            SmolStr::from(buffer.format(value))
        } else {
            unreachable!();
        };

        Some(Self { original: string, deserialized: json_num })
    }

    pub fn try_from_json_string(json_str: &str) -> Option<Self>
    {
        let deserialized = serde_json::Number::from_str(json_str).ok()?;
        Some(Self { original: SmolStr::from(json_str.as_str()), deserialized })
    }
}

/*
Parsing:
- Use regex to identify number, then parse it using a JSON deserializer with serde_json::Number::from_str() and
check result
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafNumber
{
    pub fill: CafFill,
    pub number: CafNumberValue,
}

impl CafNumber
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(
        &self,
        writer: &mut impl std::io::Write,
        space: impl AsRef<str>,
    ) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        self.number.write_to(writer)?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        self.number.to_json()
    }

    pub fn from_json_number(json_num: serde_json::Number) -> Self
    {
        Self {
            fill: CafFill::default(),
            number: CafNumberValue::from_json_number(json_num),
        }
    }

    pub fn try_from_json_string(json_str: &str) -> Option<Self>
    {
        let number = CafNumberValue::try_from_json_string(json_str)?;
        Some(Self { fill: CafFill::default(), number })
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

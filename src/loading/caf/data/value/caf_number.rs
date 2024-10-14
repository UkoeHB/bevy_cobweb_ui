use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Stores the original number value string and also the number deserialized from JSON.
///
/// We need to keep the original so it can be reserialized in the pre-existing format if possible.
///
/// We store a JSON value for convenience instead of implementing our own deserialization routine.
//todo: include nan and +/-inf
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

    /// Writes floats as ints if they have a negligible fractional component.
    pub fn write_to_simplified(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        if !self.deserialized.is_f64() || !self.deserialized.as_f64().unwrap().is_finite() {
            writer.write(self.original.as_bytes())?;
            return Ok(());
        }

        let value = self.deserialized.as_f64().unwrap();
        if value >= 0.0f64 && value <= (u64::MAX as f64) {
            let converted = value as u64;
            let diff = value - (converted as f64);
            if diff.is_subnormal() || diff == 0.0f64 {
                let mut buffer = itoa::Buffer::new();
                let formatted = buffer.format(converted);
                writer.write(formatted.as_bytes())?;
                return Ok(());
            }
        } else if value >= (i64::MIN as f64) && value <= (i64::MAX as f64) {
            let converted = value as i64;
            let diff = value - (converted as f64);
            if diff.is_subnormal() || diff == 0.0f64 {
                let mut buffer = itoa::Buffer::new();
                let formatted = buffer.format(converted);
                writer.write(formatted.as_bytes())?;
                return Ok(());
            }
        }

        writer.write(self.original.as_bytes())?;
        Ok(())
    }

    // TODO: replace with custom representation
    pub fn from_json_number(json_num: serde_json::Number) -> Self
    {
        let string = if let Some(value) = json_num.as_u64() {
            let mut buffer = itoa::Buffer::new();
            SmolStr::from(buffer.format(value))
        } else if let Some(value) = json_num.as_f64() {
            let mut buffer = ryu::Buffer::new();
            SmolStr::from(buffer.format(value)) // Includes NaN and inf/-inf
        } else if let Some(value) = json_num.as_i64() {
            let mut buffer = itoa::Buffer::new();
            SmolStr::from(buffer.format(value))
        } else {
            unreachable!();
        };

        Self { original: string, deserialized: json_num }
    }
}

impl From<i8> for CafNumberValue
{
    fn from(number: i8) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<i16> for CafNumberValue
{
    fn from(number: i16) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<i32> for CafNumberValue
{
    fn from(number: i32) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<i64> for CafNumberValue
{
    fn from(number: i64) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<i128> for CafNumberValue
{
    fn from(number: i128) -> Self
    {
        let number = i64::try_from(number).expect("i128 not yet fully supported");
        Self::from_json_number(number.into())
    }
}

impl From<u8> for CafNumberValue
{
    fn from(number: u8) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<u16> for CafNumberValue
{
    fn from(number: u16) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<u32> for CafNumberValue
{
    fn from(number: u32) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<u64> for CafNumberValue
{
    fn from(number: u64) -> Self
    {
        Self::from_json_number(number.into())
    }
}

impl From<u128> for CafNumberValue
{
    fn from(number: u128) -> Self
    {
        let number = u64::try_from(number).expect("u128 not yet fully supported");
        Self::from_json_number(number.into())
    }
}

impl From<f32> for CafNumberValue
{
    fn from(number: f32) -> Self
    {
        Self::from_json_number(serde_json::Number::from_f64(number as f64).unwrap())
    }
}

impl From<f64> for CafNumberValue
{
    fn from(number: f64) -> Self
    {
        Self::from_json_number(serde_json::Number::from_f64(number as f64).unwrap())
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

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

impl From<i8> for CafNumber
{
    fn from(number: i8) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<i16> for CafNumber
{
    fn from(number: i16) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<i32> for CafNumber
{
    fn from(number: i32) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<i64> for CafNumber
{
    fn from(number: i64) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<i128> for CafNumber
{
    fn from(number: i128) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<u8> for CafNumber
{
    fn from(number: u8) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<u16> for CafNumber
{
    fn from(number: u16) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<u32> for CafNumber
{
    fn from(number: u32) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<u64> for CafNumber
{
    fn from(number: u64) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<u128> for CafNumber
{
    fn from(number: u128) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<f32> for CafNumber
{
    fn from(number: f32) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

impl From<f64> for CafNumber
{
    fn from(number: f64) -> Self
    {
        Self { fill: CafFill::default(), number: number.into() }
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

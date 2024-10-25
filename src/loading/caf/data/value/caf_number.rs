use nom::character::complete::char;
use nom::error::ErrorKind;
use nom::multi::many0_count;
use nom::number::complete::{double, le_i128, le_u128, recognize_float_parts};
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafNumberValue
{
    Uint(u128),
    Int(i128),
    Float64(f64),
    /// Separate from `Float64` to avoid loss of exactness when going rust -> caf number -> serialized to file.
    Float32(f32),
}

impl CafNumberValue
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Uint(val) => writer.write_u128(val),
            Self::Int(val) => writer.write_i128(val),
            Self::Float64(val) => {
                if val.is_nan() {
                    write!(writer, "nan")
                } else if val.is_infinite() {
                    match val.is_sign_positive() {
                        true => write!(writer, "inf"),
                        false => write!(writer, "-inf"),
                    }
                } else {
                    writer.write_f64(val)
                }
            }
            Self::Float32(val) => {
                if val.is_nan() {
                    write!(writer, "nan")
                } else if val.is_infinite() {
                    match val.is_sign_positive() {
                        true => write!(writer, "inf"),
                        false => write!(writer, "-inf"),
                    }
                } else {
                    writer.write_f32(val)
                }
            }
        }
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        // Sign
        let (remaining, num_minuses) = many0_count(char('-')).parse(content)?;
        let sign = match num_minuses {
            0 => true,
            1 => false,
            _ => {
                tracing::warn!("failed parsing number at {}; encountered multiple '-' in a row",
                    get_location(content));
                return Err(span_verify_error(content));
            }
        };

        // Special float types: nan, inf, -inf
        if let Ok((remaining, text)) = snake_identifier(remaining) {
            let float = match *text.fragment() {
                "nan" => f64::NAN,
                "inf" => match sign {
                    true => f64::INFINITY,
                    false => f64::NEG_INFINITY,
                },
                _ => return Err(span_error(content, ErrorKind::Float)),
            };
            return Ok((Self::Float64(float), remaining));
        }

        // Break apart post-sign content
        let (remaining, (_, integer, decimal, exponent)) = recognize_float_parts(remaining)?;

        if integer.len() == 0 {
            return Err(span_error(content, ErrorKind::Float));
        }

        // Float
        if decimal.len() > 0 || exponent != 0 {
            // Backtrack to re-parse content as a double, incorporating the sign automatically.
            let (remaining, float) = double(content)?;
            return Ok((Self::Float64(float), remaining));
        }

        // Integer
        let number = match sign {
            true => match le_u128::<_, ()>(integer.fragment().as_bytes()).map(|(_, i)| Self::Uint(i)) {
                Ok(num) => num,
                Err(_) => Err(span_error(integer, ErrorKind::Digit))?,
            },
            // Backtrack to incorporate sign automatically.
            false => match le_i128::<_, ()>(content.fragment().as_bytes()).map(|(_, i)| Self::Int(i)) {
                Ok(num) => num,
                Err(_) => Err(span_error(integer, ErrorKind::Digit))?,
            },
        };

        Ok((number, remaining))
    }

    /// Converts the number to `u128` if possible to do so without precision loss.
    pub fn as_u128(&self) -> Option<u128>
    {
        match *self {
            Self::Uint(val) => Some(val),
            Self::Int(val) => {
                if val >= 0i128 {
                    Some(val as u128)
                } else {
                    None
                }
            }
            Self::Float64(val) => {
                if !val.is_finite() {
                    return None;
                }
                let converted = val as u128;
                let diff = val - (converted as f64);
                if diff.is_subnormal() || diff == 0.0 {
                    return Some(converted);
                }
                None
            }
            Self::Float32(val) => {
                if !val.is_finite() {
                    return None;
                }
                let converted = val as u128;
                let diff = val - (converted as f32);
                if diff.is_subnormal() || diff == 0.0 {
                    return Some(converted);
                }
                None
            }
        }
    }

    /// Converts the number to `i128` if possible to do so without precision loss.
    pub fn as_i128(&self) -> Option<i128>
    {
        match *self {
            Self::Uint(val) => {
                if val <= (i128::MAX as u128) {
                    Some(val as i128)
                } else {
                    None
                }
            }
            Self::Int(val) => Some(val),
            Self::Float64(val) => {
                if !val.is_finite() {
                    return None;
                }
                let converted = val as i128;
                let diff = val - (converted as f64);
                if diff.is_subnormal() || diff == 0.0 {
                    return Some(converted);
                }
                None
            }
            Self::Float32(val) => {
                if !val.is_finite() {
                    return None;
                }
                let converted = val as i128;
                let diff = val - (converted as f32);
                if diff.is_subnormal() || diff == 0.0 {
                    return Some(converted);
                }
                None
            }
        }
    }

    /// Converts the number to `f64` if possible to do so without loss of precision.
    pub fn as_f64(&self) -> Option<f64>
    {
        match *self {
            Self::Uint(val) => {
                if (val as f64) as u128 == val {
                    Some(val as f64)
                } else {
                    None
                }
            }
            Self::Int(val) => {
                if (val as f64) as i128 == val {
                    Some(val as f64)
                } else {
                    None
                }
            }
            Self::Float64(val) => Some(val),
            Self::Float32(val) => Some(val as f64),
        }
    }

    /// Converts the number to `f32` if possible to do so without loss of accuracy.
    pub fn as_f32(&self) -> Option<f32>
    {
        match *self {
            Self::Uint(val) => {
                if (val as f32) as u128 == val {
                    Some(val as f32)
                } else {
                    None
                }
            }
            Self::Int(val) => {
                if (val as f32) as i128 == val {
                    Some(val as f32)
                } else {
                    None
                }
            }
            Self::Float64(val) => {
                if (val as f32) as f64 == val {
                    Some(val as f32)
                } else {
                    None
                }
            }
            Self::Float32(val) => Some(val),
        }
    }
}

impl From<i8> for CafNumberValue
{
    fn from(number: i8) -> Self
    {
        Self::Int(number as i128)
    }
}

impl From<i16> for CafNumberValue
{
    fn from(number: i16) -> Self
    {
        Self::Int(number as i128)
    }
}

impl From<i32> for CafNumberValue
{
    fn from(number: i32) -> Self
    {
        Self::Int(number as i128)
    }
}

impl From<i64> for CafNumberValue
{
    fn from(number: i64) -> Self
    {
        Self::Int(number as i128)
    }
}

impl From<i128> for CafNumberValue
{
    fn from(number: i128) -> Self
    {
        Self::Int(number)
    }
}

impl From<u8> for CafNumberValue
{
    fn from(number: u8) -> Self
    {
        Self::Uint(number as u128)
    }
}

impl From<u16> for CafNumberValue
{
    fn from(number: u16) -> Self
    {
        Self::Uint(number as u128)
    }
}

impl From<u32> for CafNumberValue
{
    fn from(number: u32) -> Self
    {
        Self::Uint(number as u128)
    }
}

impl From<u64> for CafNumberValue
{
    fn from(number: u64) -> Self
    {
        Self::Uint(number as u128)
    }
}

impl From<u128> for CafNumberValue
{
    fn from(number: u128) -> Self
    {
        Self::Uint(number as u128)
    }
}

impl From<f32> for CafNumberValue
{
    fn from(number: f32) -> Self
    {
        Self::Float32(number)
    }
}

impl From<f64> for CafNumberValue
{
    fn from(number: f64) -> Self
    {
        Self::Float64(number)
    }
}

/*
Parsing:
- Use regex to identify number, then parse it using a JSON deserializer with serde_json::Number::from_str() and
check result
- See https://docs.rs/lexical-core/1.0.2/lexical_core/
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
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(
        &self,
        writer: &mut impl RawSerializer,
        space: impl AsRef<str>,
    ) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        self.number.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((number, remaining)) = CafNumberValue::parse(content) else { return Ok((None, fill, content)) };
        let (next_fill, remaining) = CafFill::parse(remaining);
        Ok((Some(Self { fill, number }), next_fill, remaining))
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

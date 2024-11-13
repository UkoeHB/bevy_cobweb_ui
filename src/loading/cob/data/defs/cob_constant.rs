use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::recognize;
use nom::multi::many0_count;
use nom::sequence::{preceded, tuple};
use nom::Parser;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CobConstantName
{
    pub name: SmolStr,
}

impl CobConstantName
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.name.as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        recognize(tuple((
            // First segment
            snake_identifier,
            // Extensions
            many0_count(preceded(tag("::"), snake_identifier)),
        )))
        .parse(content)
        .map(|(r, k)| (Self { name: SmolStr::from(*k.fragment()) }, r))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Commands are parsed as loadables.
#[derive(Debug, Clone, PartialEq)]
pub enum CobConstantValue
{
    Value(CobValue),
    /// Used for collections of values that will be inserted to an array/tuple/map.
    ValueGroup(CobValueGroup),
}

impl CobConstantValue
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Value(value) => {
                value.write_to(writer)?;
            }
            Self::ValueGroup(group) => {
                group.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let fill = match rc(content, move |c| CobValue::try_parse(fill, c))? {
            (Some(value), next_fill, remaining) => {
                return Ok((Some(Self::Value(value)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobValueGroup::try_parse(fill, c))? {
            (Some(group), next_fill, remaining) => {
                return Ok((Some(Self::ValueGroup(group)), next_fill, remaining));
            }
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Value(value), Self::Value(other)) => {
                value.recover_fill(&other);
            }
            (Self::ValueGroup(value), Self::ValueGroup(other)) => {
                value.recover_fill(&other);
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobConstant
{
    pub start_fill: CobFill,
    pub name: CobConstantName,
}

impl CobConstant
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        self.name.write_to(writer)?;

        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((name, remaining)) = rc(content, |c| CobConstantName::parse(c)) else {
            return Ok((None, start_fill, content));
        };
        let (end_fill, remaining) = CobFill::parse(remaining);

        let constant = Self { start_fill, name };
        Ok((Some(constant), end_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        // Name doesn't have fill
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobConstantDef
{
    pub start_fill: CobFill,
    pub name: CobConstantName,
    pub pre_eq_fill: CobFill,
    /// The value is expected to handle its own fill.
    pub value: CobConstantValue,
}

impl CobConstantDef
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        self.name.write_to(writer)?;
        self.pre_eq_fill.write_to(writer)?;
        writer.write_bytes("=".as_bytes())?;
        self.value.write_to(writer)?;

        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((name, remaining)) = rc(content, |c| CobConstantName::parse(c)) else {
            return Ok((None, start_fill, content));
        };
        let (pre_eq_fill, remaining) = CobFill::parse(remaining);
        let (remaining, _) = char('=').parse(remaining)?;
        let (value_fill, remaining) = CobFill::parse(remaining);
        let (Some(value), end_fill, remaining) = CobConstantValue::try_parse(value_fill, remaining)? else {
            tracing::warn!("constant definition is invalid at {}", get_location(content).as_str());
            return Err(span_verify_error(content));
        };

        let def = Self { start_fill, name, pre_eq_fill, value };
        Ok((Some(def), end_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        // Name has no fill
        self.pre_eq_fill.recover(&other.pre_eq_fill);
        self.value.recover_fill(&other.value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

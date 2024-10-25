use bevy::prelude::Deref;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafEnumVariantIdentifier(pub SmolStr);

impl CafEnumVariantIdentifier
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.0.as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (remaining, id) = camel_identifier(content)?;
        Ok((Self::from(*id.fragment()), remaining))
    }
}

impl From<&str> for CafEnumVariantIdentifier
{
    fn from(string: &str) -> Self
    {
        Self(SmolStr::from(string))
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafEnumVariant
{
    Unit,
    Tuple(CafTuple),
    /// Shorthand for, and equivalent to, a tuple of array.
    Array(CafArray),
    Map(CafMap),
}

impl CafEnumVariant
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Unit => (),
            Self::Tuple(tuple) => {
                tuple.write_to(writer)?;
            }
            Self::Array(array) => {
                array.write_to(writer)?;
            }
            Self::Map(map) => {
                map.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, CafFill, Span), SpanError>
    {
        if let (Some(tuple), next_fill, remaining) = CafTuple::try_parse(CafFill::default(), content)? {
            return Ok((Self::Tuple(tuple), next_fill, remaining));
        }
        if let (Some(array), next_fill, remaining) = CafArray::try_parse(CafFill::default(), content)? {
            return Ok((Self::Array(array), next_fill, remaining));
        }
        if let (Some(map), next_fill, remaining) = CafMap::try_parse(CafFill::default(), content)? {
            if !map.is_structlike() {
                tracing::warn!("failed parsing enum struct variant at {}, map is not structlike",
                    get_location(remaining));
                return Err(span_verify_error(remaining));
            }
            return Ok((Self::Map(map), next_fill, remaining));
        }

        let (next_fill, remaining) = CafFill::parse(content);
        Ok((Self::Unit, next_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Tuple(tuple), Self::Tuple(other_tuple)) => {
                tuple.recover_fill(other_tuple);
            }
            (Self::Array(array), Self::Array(other_array)) => {
                array.recover_fill(other_array);
            }
            (Self::Map(map), Self::Map(other_map)) => {
                map.recover_fill(other_map);
            }
            _ => (),
        }
    }

    /// Returns `true` if the value has no macro params.
    pub fn no_macro_params(&self) -> bool
    {
        match self {
            Self::Unit => true,
            Self::Tuple(val) => val.no_macro_params(),
            Self::Array(val) => val.no_macro_params(),
            Self::Map(val) => val.no_macro_params(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafEnum
{
    pub fill: CafFill,
    pub id: CafEnumVariantIdentifier,
    pub variant: CafEnumVariant,
}

impl CafEnum
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        self.id.write_to(writer)?;
        self.variant.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((id, remaining)) = CafEnumVariantIdentifier::parse(content) else {
            return Ok((None, fill, content));
        };
        let (variant, next_fill, remaining) = CafEnumVariant::parse(remaining)?;
        Ok((Some(Self { fill, id, variant }), next_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
        self.variant.recover_fill(&other.variant);
    }

    pub fn unit(variant: &str) -> Self
    {
        Self {
            fill: CafFill::default(),
            id: variant.into(),
            variant: CafEnumVariant::Unit,
        }
    }

    pub fn array(variant: &str, array: CafArray) -> Self
    {
        Self {
            fill: CafFill::default(),
            id: variant.into(),
            variant: CafEnumVariant::Array(array),
        }
    }

    pub fn newtype(variant: &str, value: CafValue) -> Self
    {
        Self {
            fill: CafFill::default(),
            id: variant.into(),
            variant: CafEnumVariant::Tuple(CafTuple::single(value)),
        }
    }

    pub fn tuple(variant: &str, tuple: CafTuple) -> Self
    {
        Self {
            fill: CafFill::default(),
            id: variant.into(),
            variant: CafEnumVariant::Tuple(tuple),
        }
    }

    pub fn map(variant: &str, map: CafMap) -> Self
    {
        Self {
            fill: CafFill::default(),
            id: variant.into(),
            variant: CafEnumVariant::Map(map),
        }
    }

    /// Returns `true` if the value has no macro params.
    pub fn no_macro_params(&self) -> bool
    {
        self.variant.no_macro_params()
    }
}

//-------------------------------------------------------------------------------------------------------------------

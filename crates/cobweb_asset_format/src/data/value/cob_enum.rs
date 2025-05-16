use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobEnumVariantIdentifier(pub SmolStr);

impl CobEnumVariantIdentifier
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.0.as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        // NOTE: recursion not tested here (not vulnerable)
        let (remaining, id) = camel_identifier(content)?;
        Ok((Self::from(*id.fragment()), remaining))
    }

    pub fn as_str(&self) -> &str
    {
        self.0.as_str()
    }
}

impl From<&str> for CobEnumVariantIdentifier
{
    fn from(string: &str) -> Self
    {
        Self(SmolStr::from(string))
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobEnumVariant
{
    Unit,
    Tuple(CobTuple),
    /// Shorthand for, and equivalent to, a tuple of array.
    Array(CobArray),
    Map(CobMap),
}

impl CobEnumVariant
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

    pub fn parse(content: Span) -> Result<(Self, CobFill, Span), SpanError>
    {
        if let (Some(tuple), next_fill, remaining) = rc(content, |c| CobTuple::try_parse(CobFill::default(), c))? {
            return Ok((Self::Tuple(tuple), next_fill, remaining));
        }
        if let (Some(array), next_fill, remaining) = rc(content, |c| CobArray::try_parse(CobFill::default(), c))? {
            return Ok((Self::Array(array), next_fill, remaining));
        }
        if let (Some(map), next_fill, remaining) = rc(content, |c| CobMap::try_parse(CobFill::default(), c))? {
            return Ok((Self::Map(map), next_fill, remaining));
        }

        let (next_fill, remaining) = CobFill::parse(content);
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

    #[cfg(feature = "full")]
    pub fn resolve(&mut self, resolver: &CobLoadableResolver) -> Result<(), String>
    {
        match self {
            Self::Unit => Ok(()),
            Self::Array(arr) => arr.resolve(resolver),
            Self::Tuple(tup) => tup.resolve(resolver),
            Self::Map(map) => map.resolve(resolver),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobEnum
{
    pub fill: CobFill,
    pub id: CobEnumVariantIdentifier,
    pub variant: CobEnumVariant,
}

impl CobEnum
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

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((id, remaining)) = rc(content, |c| CobEnumVariantIdentifier::parse(c)) else {
            return Ok((None, fill, content));
        };
        let (variant, next_fill, remaining) = rc(remaining, |rm| CobEnumVariant::parse(rm))?;
        Ok((Some(Self { fill, id, variant }), next_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
        self.variant.recover_fill(&other.variant);
    }

    #[cfg(feature = "full")]
    pub fn resolve(&mut self, resolver: &CobLoadableResolver) -> Result<(), String>
    {
        self.variant.resolve(resolver)
    }

    pub fn unit(variant: &str) -> Self
    {
        Self {
            fill: CobFill::default(),
            id: variant.into(),
            variant: CobEnumVariant::Unit,
        }
    }

    pub fn array(variant: &str, array: CobArray) -> Self
    {
        Self {
            fill: CobFill::default(),
            id: variant.into(),
            variant: CobEnumVariant::Array(array),
        }
    }

    pub fn newtype(variant: &str, value: CobValue) -> Self
    {
        Self {
            fill: CobFill::default(),
            id: variant.into(),
            variant: CobEnumVariant::Tuple(CobTuple::single(value)),
        }
    }

    pub fn tuple(variant: &str, tuple: CobTuple) -> Self
    {
        Self {
            fill: CobFill::default(),
            id: variant.into(),
            variant: CobEnumVariant::Tuple(tuple),
        }
    }

    pub fn map(variant: &str, map: CobMap) -> Self
    {
        Self {
            fill: CobFill::default(),
            id: variant.into(),
            variant: CobEnumVariant::Map(map),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::reflect::serde::TypedReflectSerializer;
use bevy::reflect::{Reflect, TypeRegistry};
use nom::bytes::complete::tag;
use nom::Parser;
use serde::Serialize;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafLoadableIdentifier
{
    pub name: SmolStr,
    pub generics: Option<CafGenerics>,
}

impl CafLoadableIdentifier
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes(self.name.as_bytes())?;
        if let Some(generics) = &self.generics {
            generics.write_to(writer)?;
        }
        Ok(())
    }

    /// The canonical string can be used to access the type in the reflection type registry.
    ///
    /// You can pass a scratch string as input to reuse a string buffer for querying multiple identifiers.
    pub fn to_canonical(&self, scratch: Option<String>) -> String
    {
        let mut buff = scratch.unwrap_or_default();
        buff.clear();
        buff.push_str(self.name.as_str());
        if let Some(generics) = &self.generics {
            generics.write_canonical(&mut buff);
        }
        buff
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (remaining, id) = camel_identifier(content)?;
        let (generics, remaining) = CafGenerics::try_parse(remaining)?;
        Ok((Self { name: SmolStr::from(*id.fragment()), generics }, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        if let (Some(generics), Some(other_generics)) = (&mut self.generics, &other.generics) {
            generics.recover_fill(other_generics);
        }
    }

    pub fn is_resolved(&self) -> bool
    {
        let Some(generics) = &self.generics else { return true };
        generics.is_resolved()
    }

    //todo: resolve_constants
    //todo: resolve_macro
}

impl TryFrom<&'static str> for CafLoadableIdentifier
{
    type Error = CafError;

    /// Fails if generics fail to parse.
    fn try_from(short_path: &'static str) -> CafResult<Self>
    {
        match Self::parse(Span::new_extra(
            short_path,
            CafLocationMetadata { file: "CafLoadableIdentifier::try_from" },
        )) {
            Ok((id, remaining)) => {
                if remaining.fragment().len() == 0 {
                    Ok(id)
                } else {
                    Err(CafError::MalformedLoadableId)
                }
            }
            Err(_) => Err(CafError::MalformedLoadableId),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Variant for [`CafLoadable`].
#[derive(Debug, Clone, PartialEq)]
pub enum CafLoadableVariant
{
    /// Corresponds to a unit struct.
    Unit,
    /// Corresponds to a tuple struct.
    Tuple(CafTuple),
    /// This is a shorthand and equivalent to a tuple struct of an array.
    Array(CafArray),
    /// Corresponds to a plain struct.
    Map(CafMap),
    /// Corresponds to an enum.
    Enum(CafEnum),
}

impl CafLoadableVariant
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
            Self::Enum(variant) => {
                writer.write_bytes("::".as_bytes())?;
                variant.write_to(writer)?;
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
            // Note: we don't test if the map is struct-like here since it may *not* be a struct if a data map
            // was flattened into a newtype loadable.
            return Ok((Self::Map(map), next_fill, remaining));
        }
        if let Ok((remaining, _)) = tag::<_, _, ()>("::").parse(content) {
            match CafEnum::try_parse(CafFill::default(), remaining)? {
                (Some(variant), next_fill, remaining) => return Ok((Self::Enum(variant), next_fill, remaining)),
                _ => {
                    tracing::warn!("failed parsing loadable enum at {}; no valid variant name",
                        get_location(content));
                    return Err(span_verify_error(content));
                }
            }
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
            (Self::Enum(variant), Self::Enum(other_variant)) => {
                variant.recover_fill(other_variant);
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafLoadable
{
    pub fill: CafFill,
    pub id: CafLoadableIdentifier,
    pub variant: CafLoadableVariant,
}

impl CafLoadable
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.fill.write_to(writer)?;
        self.id.write_to(writer)?;
        self.variant.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((id, remaining)) = CafLoadableIdentifier::parse(content) else {
            return Ok((None, fill, content));
        };
        let (variant, post_fill, remaining) = CafLoadableVariant::parse(remaining)?;
        Ok((Some(Self { fill, id, variant }), post_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
        self.id.recover_fill(&other.id);
        self.variant.recover_fill(&other.variant);
    }

    pub fn extract<T: Serialize + 'static>(value: &T, registry: &TypeRegistry) -> CafResult<Self>
    {
        let registration = registry
            .get(std::any::TypeId::of::<T>())
            .ok_or(CafError::LoadableNotRegistered)?;
        let name = registration.type_info().type_path_table().short_path();
        value.serialize(CafLoadableSerializer { name })
    }

    pub fn extract_reflect<T: Reflect + 'static>(value: &T, registry: &TypeRegistry) -> CafResult<Self>
    {
        let registration = registry
            .get(std::any::TypeId::of::<T>())
            .ok_or(CafError::LoadableNotRegistered)?;
        let name = registration.type_info().type_path_table().short_path();
        let wrapper = TypedReflectSerializer::new(value, registry);
        wrapper.serialize(CafLoadableSerializer { name })
    }
}

//-------------------------------------------------------------------------------------------------------------------

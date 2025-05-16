#[cfg(feature = "full")]
use bevy::reflect::serde::TypedReflectSerializer;
#[cfg(feature = "full")]
use bevy::reflect::{PartialReflect, Reflect, TypeRegistry};
use nom::bytes::complete::tag;
use nom::Parser;
use serde::Serialize;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CobLoadableIdentifier
{
    pub name: SmolStr,
    pub generics: Option<CobGenerics>,
}

impl CobLoadableIdentifier
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
        let (generics, remaining) = rc(remaining, |rm| CobGenerics::try_parse(rm))?;
        Ok((Self { name: SmolStr::from(*id.fragment()), generics }, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        if let (Some(generics), Some(other_generics)) = (&mut self.generics, &other.generics) {
            generics.recover_fill(other_generics);
        }
    }

    // pub fn is_resolved(&self) -> bool
    // {
    //     let Some(generics) = &self.generics else { return true };
    //     generics.is_resolved()
    // }
}

impl TryFrom<&'static str> for CobLoadableIdentifier
{
    type Error = CobError;

    /// Fails if generics fail to parse.
    fn try_from(short_path: &'static str) -> CobResult<Self>
    {
        match Self::parse(Span::new_extra(
            short_path,
            CobLocationMetadata { file: "CobLoadableIdentifier::try_from" },
        )) {
            Ok((id, remaining)) => {
                if remaining.fragment().len() == 0 {
                    Ok(id)
                } else {
                    Err(CobError::MalformedLoadableId)
                }
            }
            Err(_) => Err(CobError::MalformedLoadableId),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Variant for [`CobLoadable`].
#[derive(Debug, Clone, PartialEq)]
pub enum CobLoadableVariant
{
    /// Corresponds to a unit struct.
    Unit,
    /// Corresponds to a tuple struct.
    Tuple(CobTuple),
    /// This is a shorthand and equivalent to a tuple struct of an array.
    Array(CobArray),
    /// Corresponds to a plain struct.
    Map(CobMap),
    /// Corresponds to an enum.
    Enum(CobEnum),
}

impl CobLoadableVariant
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

    pub fn parse(content: Span) -> Result<(Self, CobFill, Span), SpanError>
    {
        if let (Some(tuple), next_fill, remaining) = rc(content, |c| CobTuple::try_parse(CobFill::default(), c))? {
            return Ok((Self::Tuple(tuple), next_fill, remaining));
        }
        if let (Some(array), next_fill, remaining) = rc(content, |c| CobArray::try_parse(CobFill::default(), c))? {
            return Ok((Self::Array(array), next_fill, remaining));
        }
        if let (Some(map), next_fill, remaining) = rc(content, |c| CobMap::try_parse(CobFill::default(), c))? {
            // Note: we don't test if the map is struct-like here since it may *not* be a struct if a data map
            // was flattened into a newtype loadable.
            return Ok((Self::Map(map), next_fill, remaining));
        }
        if let Ok((remaining, _)) = tag::<_, _, ()>("::").parse(content) {
            match rc(remaining, |rm| CobEnum::try_parse(CobFill::default(), rm))? {
                (Some(variant), next_fill, remaining) => return Ok((Self::Enum(variant), next_fill, remaining)),
                _ => {
                    tracing::warn!("failed parsing loadable enum at {}; no valid variant name",
                        get_location(content));
                    return Err(span_verify_error(content));
                }
            }
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
            (Self::Enum(variant), Self::Enum(other_variant)) => {
                variant.recover_fill(other_variant);
            }
            _ => (),
        }
    }

    #[cfg(feature = "full")]
    pub fn resolve(&mut self, resolver: &CobLoadableResolver) -> Result<(), String>
    {
        match self {
            Self::Unit => Ok(()),
            Self::Tuple(tuple) => tuple.resolve(resolver),
            Self::Array(array) => array.resolve(resolver),
            Self::Map(map) => map.resolve(resolver),
            Self::Enum(variant) => variant.resolve(resolver),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobLoadable
{
    pub fill: CobFill,
    pub id: CobLoadableIdentifier,
    pub variant: CobLoadableVariant,
}

impl CobLoadable
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.fill.write_to(writer)?;
        self.id.write_to(writer)?;
        self.variant.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((id, remaining)) = rc(content, |c| CobLoadableIdentifier::parse(c)) else {
            return Ok((None, fill, content));
        };
        let (variant, post_fill, remaining) = rc(remaining, |rm| CobLoadableVariant::parse(rm))?;
        Ok((Some(Self { fill, id, variant }), post_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
        self.id.recover_fill(&other.id);
        self.variant.recover_fill(&other.variant);
    }

    #[cfg(feature = "full")]
    pub fn resolve(&mut self, resolver: &CobLoadableResolver) -> Result<(), String>
    {
        self.variant.resolve(resolver)
    }

    pub fn extract<T: Serialize + 'static>(value: &T, name: &'static str) -> CobResult<Self>
    {
        value.serialize(CobLoadableSerializer { name })
    }

    #[cfg(feature = "full")]
    pub fn extract_with_registry<T: Serialize + 'static>(value: &T, registry: &TypeRegistry) -> CobResult<Self>
    {
        let type_info = registry
            .get_type_info(std::any::TypeId::of::<T>())
            .ok_or(CobError::LoadableNotRegistered)?;
        let name = type_info.type_path_table().short_path();
        Self::extract(value, name)
    }

    #[cfg(feature = "full")]
    pub fn extract_reflect<T: Reflect + 'static>(value: &T, registry: &TypeRegistry) -> CobResult<Self>
    {
        let type_info = registry
            .get_type_info(std::any::TypeId::of::<T>())
            .ok_or(CobError::LoadableNotRegistered)?;
        let name = type_info.type_path_table().short_path();
        let wrapper = TypedReflectSerializer::new(value, registry);
        wrapper.serialize(CobLoadableSerializer { name })
    }

    /// This is slightly less efficient than [`Self::extract_reflect`], which should be preferred if possible.
    #[cfg(feature = "full")]
    pub fn extract_partial_reflect(
        value: &(dyn PartialReflect + 'static),
        registry: &TypeRegistry,
    ) -> CobResult<Self>
    {
        let type_info = value
            .get_represented_type_info()
            .ok_or(CobError::LoadableNotRegistered)?;
        let name = type_info.type_path_table().short_path();
        let wrapper = TypedReflectSerializer::new(value, registry);
        wrapper.serialize(CobLoadableSerializer { name })
    }
}

//-------------------------------------------------------------------------------------------------------------------

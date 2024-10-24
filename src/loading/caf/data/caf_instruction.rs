use bevy::reflect::TypeRegistry;
use serde::Serialize;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafInstructionIdentifier
{
    pub name: SmolStr,
    pub generics: Option<CafGenerics>,
}

impl CafInstructionIdentifier
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

impl TryFrom<&'static str> for CafInstructionIdentifier
{
    type Error = CafError;

    fn try_from(short_path: &'static str) -> CafResult<Self>
    {
        Ok(Self { name: SmolStr::new_static(short_path), generics: None })
    }
}

/*
Parsing:
- identifier is camelcase
- generics have no preceding whitespace
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafInstruction
{
    /// Corresponds to a unit struct.
    Unit
    {
        fill: CafFill, id: CafInstructionIdentifier
    },
    /// Corresponds to a tuple struct.
    Tuple
    {
        fill: CafFill, id: CafInstructionIdentifier, tuple: CafTuple
    },
    /// This is a shorthand and equivalent to a tuple struct of an array.
    Array
    {
        fill: CafFill, id: CafInstructionIdentifier, array: CafArray
    },
    /// Corresponds to a plain struct.
    Map
    {
        fill: CafFill, id: CafInstructionIdentifier, map: CafMap
    },
    /// Corresponds to an enum.
    Enum
    {
        fill: CafFill,
        id: CafInstructionIdentifier,
        variant: CafEnumVariant,
    },
}

impl CafInstruction
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Unit { fill, id } => {
                fill.write_to(writer)?;
                id.write_to(writer)?;
            }
            Self::Tuple { fill, id, tuple } => {
                fill.write_to(writer)?;
                id.write_to(writer)?;
                tuple.write_to(writer)?;
            }
            Self::Array { fill, id, array } => {
                fill.write_to(writer)?;
                id.write_to(writer)?;
                array.write_to(writer)?;
            }
            Self::Map { fill, id, map } => {
                fill.write_to(writer)?;
                id.write_to(writer)?;
                map.write_to(writer)?;
            }
            Self::Enum { fill, id, variant } => {
                fill.write_to(writer)?;
                id.write_to(writer)?;
                writer.write_bytes("::".as_bytes())?;
                variant.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Unit { fill, id }, Self::Unit { fill: other_fill, id: other }) => {
                fill.recover(&other_fill);
                id.recover_fill(other);
            }
            (
                Self::Tuple { fill, id, tuple },
                Self::Tuple { fill: other_fill, id: other_id, tuple: other_tuple },
            ) => {
                fill.recover(&other_fill);
                id.recover_fill(other_id);
                tuple.recover_fill(other_tuple);
            }
            (
                Self::Array { fill, id, array },
                Self::Array { fill: other_fill, id: other_id, array: other_array },
            ) => {
                fill.recover(&other_fill);
                id.recover_fill(other_id);
                array.recover_fill(other_array);
            }
            (Self::Map { fill, id, map }, Self::Map { fill: other_fill, id: other_id, map: other_map }) => {
                fill.recover(&other_fill);
                id.recover_fill(other_id);
                map.recover_fill(other_map);
            }
            (
                Self::Enum { fill, id, variant },
                Self::Enum { fill: other_fill, id: other_id, variant: other_variant },
            ) => {
                fill.recover(&other_fill);
                id.recover_fill(other_id);
                variant.recover_fill(other_variant);
            }
            _ => (),
        }
    }

    pub fn id(&self) -> &CafInstructionIdentifier
    {
        match self {
            Self::Unit { id, .. }
            | Self::Tuple { id, .. }
            | Self::Array { id, .. }
            | Self::Map { id, .. }
            | Self::Enum { id, .. } => id,
        }
    }

    pub fn extract<T: Serialize + 'static>(value: &T, registry: &TypeRegistry) -> CafResult<Self>
    {
        let registration = registry
            .get(std::any::TypeId::of::<T>())
            .ok_or(CafError::InstructionNotRegistered)?;
        let name = registration.type_info().type_path_table().short_path();
        value.serialize(CafInstructionSerializer { name })
    }
}

/*
Parsing:
- no whitespace allowed between identifier and value
- map-type instructions can only have field name map keys in the base layer
*/

//-------------------------------------------------------------------------------------------------------------------

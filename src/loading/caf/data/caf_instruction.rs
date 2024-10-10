use bevy::reflect::TypeRegistry;
use serde::Serialize;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafInstructionIdentifier
{
    pub start_fill: CafFill,
    pub name: SmolStr,
    pub generics: Option<CafGenerics>,
}

impl CafInstructionIdentifier
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write(self.name.as_bytes())?;
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

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);

        if let (Some(generics), Some(other_generics)) = (&mut self.generics, &other.generics) {
            generics.recover_fill(other_generics);
        }
    }

    //todo: resolve_constants
    //todo: resolve_macro
}

impl TryFrom<&'static str> for CafInstructionIdentifier
{
    type Error = CafError;

    fn try_from(short_path: &'static str) -> CafResult<Self>
    {
        Ok(Self {
            start_fill: CafFill::default(),
            name: SmolStr::new_static(short_path),
            generics: None,
        })
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafInstruction
{
    /// Corresponds to a unit struct.
    Unit
    {
        id: CafInstructionIdentifier
    },
    /// Corresponds to a tuple struct.
    Tuple
    {
        id: CafInstructionIdentifier, tuple: CafTuple
    },
    /// This is a shorthand and equivalent to a tuple struct of an array.
    Array
    {
        id: CafInstructionIdentifier, array: CafArray
    },
    /// Corresponds to a plain struct.
    Map
    {
        id: CafInstructionIdentifier, map: CafMap
    },
    /// Corresponds to an enum.
    Enum
    {
        id: CafInstructionIdentifier, variant: CafEnumVariant
    },
}

impl CafInstruction
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Unit { id } => {
                id.write_to(writer)?;
            }
            Self::Tuple { id, tuple } => {
                id.write_to(writer)?;
                tuple.write_to(writer)?;
            }
            Self::Array { id, array } => {
                id.write_to(writer)?;
                array.write_to(writer)?;
            }
            Self::Map { id, map } => {
                id.write_to(writer)?;
                map.write_to(writer)?;
            }
            Self::Enum { id, variant } => {
                id.write_to(writer)?;
                writer.write("::".as_bytes())?;
                variant.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        match self {
            Self::Unit { .. } => {
                // {}
                Ok(serde_json::Value::Object(serde_json::Map::default()))
            }
            Self::Tuple { tuple, .. } => {
                // [..tuple items..]
                tuple.to_json_for_type()
            }
            Self::Array { array, .. } => {
                // [[..array items..]]
                Ok(serde_json::Value::Array(vec![array.to_json()?]))
            }
            Self::Map { map, .. } => {
                // {..map items..}
                map.to_json()
            }
            Self::Enum { variant, .. } => {
                // .. enum variant ..
                variant.to_json()
            }
        }
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Unit { id }, Self::Unit { id: other }) => {
                id.recover_fill(other);
            }
            (Self::Tuple { id, tuple }, Self::Tuple { id: other_id, tuple: other_tuple }) => {
                id.recover_fill(other_id);
                tuple.recover_fill(other_tuple);
            }
            (Self::Array { id, array }, Self::Array { id: other_id, array: other_array }) => {
                id.recover_fill(other_id);
                array.recover_fill(other_array);
            }
            (Self::Map { id, map }, Self::Map { id: other_id, map: other_map }) => {
                id.recover_fill(other_id);
                map.recover_fill(other_map);
            }
            (Self::Enum { id, variant }, Self::Enum { id: other_id, variant: other_variant }) => {
                id.recover_fill(other_id);
                variant.recover_fill(other_variant);
            }
            _ => (),
        }
    }

    pub fn id(&self) -> &CafInstructionIdentifier
    {
        match self {
            Self::Unit { id }
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

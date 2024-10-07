use bevy::reflect::{TupleStructInfo, TypeInfo, TypeRegistry};
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

    pub fn from_short_path(short_path: &'static str) -> Result<Self, std::io::Error>
    {
        // TODO: properly parse the path to extract generics so recover_fill can repair them
        Ok(Self {
            start_fill: CafFill::default(),
            name: SmolStr::new_static(short_path),
            generics: None,
        })
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

    // Expect:
    // - JSON: [[..vals..]]
    // - Info: TupleStruct of single element, single element is an array or list
    fn array_from_json(
        val: &serde_json::Value,
        info: &TupleStructInfo,
        registry: &TypeRegistry,
    ) -> Result<Option<Self>, String>
    {
        // Check if JSON is array of array.
        let serde_json::Value::Array(arr) = val else { return Ok(None) };
        if arr.len() != 1 {
            return Ok(None);
        }
        if !matches!(arr[0], serde_json::Value::Array(_)) {
            return Ok(None);
        }

        // Get type info of inner slice.
        if info.field_len() != 1 {
            return Ok(None);
        }
        let Some(registration) = registry.get(info.field_at(0).unwrap().type_id()) else { unreachable!() };
        if !matches!(registration.type_info(), &TypeInfo::Array(_))
            || !matches!(registration.type_info(), &TypeInfo::List(_))
        {
            return Ok(None);
        }

        Ok(Some(Self::Array {
            id: CafInstructionIdentifier::from_short_path(info.type_path_table().short_path())
                .map_err(|e| format!("{e:?}"))?,
            array: CafArray::from_json(&arr[0], registration.type_info(), registry)?,
        }))
    }

    pub fn from_json(
        val: &serde_json::Value,
        type_info: &TypeInfo,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        match type_info {
            TypeInfo::Struct(info) => {
                // Case 1: zero-sized type
                if info.field_len() == 0 {
                    if *val != serde_json::Value::Null
                        && *val != serde_json::Value::Array(vec![])
                        && *val != serde_json::Value::Object(serde_json::Map::default())
                    {
                        tracing::warn!("encountered non-empty JSON value {:?} when converting zero-size-type {:?}",
                            val, type_info.type_path());
                    }

                    return Ok(Self::Unit {
                        id: CafInstructionIdentifier::from_short_path(info.type_path_table().short_path())
                            .map_err(|e| format!("{e:?}"))?,
                    });
                }

                // Case 2: normal struct
                Ok(Self::Map {
                    id: CafInstructionIdentifier::from_short_path(info.type_path_table().short_path())
                        .map_err(|e| format!("{e:?}"))?,
                    map: CafMap::from_json_as_type(val, type_info, registry)?,
                })
            }
            TypeInfo::TupleStruct(info) => {
                // Case 1: tuple of list/array
                if let Some(result) = Self::array_from_json(val, info, registry)? {
                    return Ok(result);
                }

                // Case 2: tuple of anything else
                Ok(Self::Tuple {
                    id: CafInstructionIdentifier::from_short_path(info.type_path_table().short_path())
                        .map_err(|e| format!("{e:?}"))?,
                    tuple: CafTuple::from_json_as_type(val, type_info, registry)?,
                })
            }
            TypeInfo::Enum(info) => {
                // Note: we assume no instruction is ever Option<T>, so there is no need to check here.
                Ok(Self::Enum {
                    id: CafInstructionIdentifier::from_short_path(info.type_path_table().short_path())
                        .map_err(|e| format!("{e:?}"))?,
                    variant: CafEnumVariant::from_json(val, info, registry)?,
                })
            }
            _ => Err(format!(
                "failed converting {:?} from json {:?} as an instruction; type is not a struct/tuplestruct/enum",
                val, type_info.type_path()
            )),
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
}

/*
Parsing:
- no whitespace allowed between identifier and value
- map-type instructions can only have field name map keys in the base layer
*/

//-------------------------------------------------------------------------------------------------------------------

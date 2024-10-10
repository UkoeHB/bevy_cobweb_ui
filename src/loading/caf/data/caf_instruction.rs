use bevy::reflect::{TupleStructInfo, TypeInfo, TypeRegistry};
use serde::ser::Impossible;
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
            && !matches!(registration.type_info(), &TypeInfo::List(_))
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

pub struct CafInstructionSerializer
{
    /// The instruction name is injected because serde doesn't know about generics.
    //todo: add ability to customize the instruction name e.g. in the case of 'using' statements
    pub name: &'static str,
}

impl serde::Serializer for CafInstructionSerializer
{
    type Ok = CafInstruction;
    type Error = CafError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = SerializeTupleStruct;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, _: bool) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_i8(self, _: i8) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_i16(self, _: i16) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_i32(self, _: i32) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_i64(self, _: i64) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_i128(self, _: i128) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u8(self, _: u8) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u16(self, _: u16) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u32(self, _: u32) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u64(self, _: u64) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_u128(self, _: u128) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_f32(self, _: f32) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_f64(self, _: f64) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_char(self, _: char) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_str(self, _: &str) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_bytes(self, _: &[u8]) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_unit(self) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> CafResult<CafInstruction>
    {
        Ok(CafInstruction::Unit { id: self.name.try_into()? })
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> CafResult<CafInstruction>
    {
        Ok(CafInstruction::Enum {
            id: self.name.try_into()?,
            variant: CafEnumVariant::unit(variant),
        })
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> CafResult<CafInstruction>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CafValue::extract(value)?;

        if let CafValue::Array(array) = value_ser {
            Ok(CafInstruction::Array { id: self.name.try_into()?, array })
        } else {
            Ok(CafInstruction::Tuple {
                id: self.name.try_into()?,
                tuple: CafTuple::single(value_ser),
            })
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> CafResult<CafInstruction>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CafValue::extract(value)?;

        if let CafValue::Array(arr) = value_ser {
            Ok(CafInstruction::Enum {
                id: self.name.try_into()?,
                variant: CafEnumVariant::array(variant, arr),
            })
        } else {
            Ok(CafInstruction::Enum {
                id: self.name.try_into()?,
                variant: CafEnumVariant::newtype(variant, value_ser),
            })
        }
    }

    #[inline]
    fn serialize_none(self) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_some<T>(self, _: &T) -> CafResult<CafInstruction>
    where
        T: ?Sized + Serialize,
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_seq(self, _: Option<usize>) -> CafResult<Self::SerializeSeq>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_tuple(self, _: usize) -> CafResult<Self::SerializeTuple>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> CafResult<Self::SerializeTupleStruct>
    {
        Ok(SerializeTupleStruct { name: self.name, vec: Vec::with_capacity(len) })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CafResult<Self::SerializeTupleVariant>
    {
        Ok(SerializeTupleVariant { name: self.name, variant, vec: Vec::with_capacity(len) })
    }

    fn serialize_map(self, _: Option<usize>) -> CafResult<Self::SerializeMap>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> CafResult<Self::SerializeStruct>
    {
        Ok(SerializeStruct { name: self.name, vec: Vec::with_capacity(len) })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CafResult<Self::SerializeStructVariant>
    {
        Ok(SerializeStructVariant { name: self.name, variant, vec: Vec::with_capacity(len) })
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTupleStruct
{
    name: &'static str,
    vec: Vec<CafValue>,
}

impl serde::ser::SerializeTupleStruct for SerializeTupleStruct
{
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        if self.vec.len() > 0 {
            Ok(CafInstruction::Tuple { id: self.name.try_into()?, tuple: CafTuple::from(self.vec) })
        } else {
            Ok(CafInstruction::Unit { id: self.name.try_into()? })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTupleVariant
{
    name: &'static str,
    variant: &'static str,
    vec: Vec<CafValue>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant
{
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        Ok(CafInstruction::Enum {
            id: self.name.try_into()?,
            variant: CafEnumVariant::tuple(self.variant, CafTuple::from(self.vec)),
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStruct
{
    name: &'static str,
    vec: Vec<CafMapEntry>,
}

impl serde::ser::SerializeStruct for SerializeStruct
{
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        if self.vec.len() > 0 {
            Ok(CafInstruction::Map { id: self.name.try_into()?, map: CafMap::from(self.vec) })
        } else {
            Ok(CafInstruction::Unit { id: self.name.try_into()? })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStructVariant
{
    name: &'static str,
    variant: &'static str,
    vec: Vec<CafMapEntry>,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant
{
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        if self.vec.len() > 0 {
            Ok(CafInstruction::Enum {
                id: self.name.try_into()?,
                variant: CafEnumVariant::map(self.variant, CafMap::from(self.vec)),
            })
        } else {
            Ok(CafInstruction::Enum {
                id: self.name.try_into()?,
                variant: CafEnumVariant::unit(self.variant),
            })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

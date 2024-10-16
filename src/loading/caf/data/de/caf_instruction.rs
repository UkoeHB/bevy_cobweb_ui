use serde::de::{Expected, Unexpected, Visitor};
use serde::forward_to_deserialize_any;

use super::{
    visit_array_ref, visit_map_ref, visit_tuple_ref, visit_wrapped_array_ref, EnumRefDeserializer,
    MapRefDeserializer,
};
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Allows converting a [`CafValue`] to a concrete type.
impl<'de> serde::Deserializer<'de> for &'de CafInstruction
{
    type Error = CafError;

    fn deserialize_any<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafInstruction::Unit { .. } => visitor.visit_unit(),
            CafInstruction::Tuple { tuple, .. } => visit_tuple_ref(tuple, visitor),
            CafInstruction::Array { array, .. } => visit_array_ref(array, visitor),
            CafInstruction::Map { map, .. } => visit_map_ref(map, visitor),
            CafInstruction::Enum { variant, .. } => visitor.visit_enum(EnumRefDeserializer { variant }),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let re = match self {
            CafInstruction::Enum { variant, .. } => visitor.visit_enum(EnumRefDeserializer { variant }),
            _ => Err(self.invalid_type(&visitor)),
        }?;
        Ok(re)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafInstruction::Tuple { tuple, .. } => {
                if tuple.entries.len() == 1 {
                    visitor.visit_newtype_struct(&tuple.entries[0])
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            CafInstruction::Array { array, .. } => visitor.visit_newtype_struct(array),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafInstruction::Unit { .. } => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafInstruction::Tuple { tuple, .. } => visit_tuple_ref(tuple, visitor),
            CafInstruction::Array { array, .. } => {
                if len == 1 {
                    visit_wrapped_array_ref(array, visitor)
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafInstruction::Unit { .. } => {
                // Use this instead of `visitor.visit_unit()` because some visitor implementations don't handle it
                // properly.
                let mut deserializer = MapRefDeserializer::new(&[]);
                visitor.visit_map(&mut deserializer)
            }
            CafInstruction::Map { map, .. } => visit_map_ref(map, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl CafInstruction
{
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected
    {
        match self {
            CafInstruction::Unit { .. } => Unexpected::Unit,
            CafInstruction::Tuple { tuple, .. } => {
                if tuple.entries.len() == 1 {
                    Unexpected::NewtypeStruct
                } else {
                    Unexpected::Seq
                }
            }
            CafInstruction::Array { .. } => Unexpected::NewtypeStruct,
            CafInstruction::Map { .. } => Unexpected::Map,
            CafInstruction::Enum { variant, .. } => variant.unexpected(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

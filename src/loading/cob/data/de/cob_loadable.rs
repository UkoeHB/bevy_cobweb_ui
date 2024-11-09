use serde::de::{Expected, Unexpected, Visitor};
use serde::forward_to_deserialize_any;

use super::{
    visit_array_ref, visit_map_ref, visit_tuple_ref, visit_wrapped_array_ref, visit_wrapped_erased_ref,
    visit_wrapped_map_ref, visit_wrapped_tuple_ref, EnumRefDeserializer, ErasedNewtypeStruct, MapRefDeserializer,
};
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Allows converting a [`CobValue`] to a concrete type.
impl<'de> serde::Deserializer<'de> for &'de CobLoadable
{
    type Error = CobError;

    fn deserialize_any<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.variant {
            CobLoadableVariant::Unit => visitor.visit_unit(),
            CobLoadableVariant::Tuple(tuple) => visit_tuple_ref(&tuple.entries, visitor),
            CobLoadableVariant::Array(array) => visit_array_ref(&array.entries, visitor),
            CobLoadableVariant::Map(map) => visit_map_ref(&map.entries, visitor),
            CobLoadableVariant::Enum(variant) => visitor.visit_enum(EnumRefDeserializer { variant }),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let re = match &self.variant {
            CobLoadableVariant::Enum(variant) => visitor.visit_enum(EnumRefDeserializer { variant }),
            _ => Err(self.invalid_type(&visitor)),
        }?;
        Ok(re)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.variant {
            CobLoadableVariant::Tuple(tuple) => {
                if tuple.entries.len() == 1 {
                    visitor.visit_newtype_struct(&tuple.entries[0])
                } else {
                    visit_wrapped_tuple_ref(tuple, visitor)
                }
            }
            CobLoadableVariant::Array(array) => visitor.visit_newtype_struct(array),
            CobLoadableVariant::Map(map) => visitor.visit_newtype_struct(map),
            CobLoadableVariant::Unit => visitor.visit_newtype_struct(ErasedNewtypeStruct),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.variant {
            CobLoadableVariant::Unit => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.variant {
            CobLoadableVariant::Tuple(tuple) => {
                if tuple.entries.len() == len {
                    visit_tuple_ref(&tuple.entries, visitor)
                } else {
                    visit_wrapped_tuple_ref(tuple, visitor)
                }
            }
            // Special cases: flattened
            CobLoadableVariant::Array(array) => {
                if len == 1 {
                    visit_wrapped_array_ref(array, visitor)
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            CobLoadableVariant::Map(map) => {
                if len == 1 {
                    visit_wrapped_map_ref(map, visitor)
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            CobLoadableVariant::Unit => {
                if len == 0 {
                    visit_tuple_ref(&[], visitor)
                } else if len == 1 {
                    visit_wrapped_erased_ref(visitor)
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
    ) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.variant {
            CobLoadableVariant::Unit => {
                // Use this instead of `visitor.visit_unit()` because some visitor implementations don't handle it
                // properly.
                let mut deserializer = MapRefDeserializer::new(&[]);
                visitor.visit_map(&mut deserializer)
            }
            CobLoadableVariant::Map(map) => visit_map_ref(&map.entries, visitor),
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

impl CobLoadable
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
        match &self.variant {
            CobLoadableVariant::Unit => Unexpected::Unit,
            CobLoadableVariant::Tuple(tuple) => {
                if tuple.entries.len() == 1 {
                    Unexpected::NewtypeStruct
                } else {
                    Unexpected::Seq
                }
            }
            CobLoadableVariant::Array(..) => Unexpected::NewtypeStruct,
            CobLoadableVariant::Map(..) => Unexpected::Map,
            CobLoadableVariant::Enum(variant) => variant.unexpected(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

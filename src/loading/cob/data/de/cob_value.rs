use serde::de::{Expected, Unexpected, Visitor};

use super::{
    deserialize_builtin, visit_array_ref, visit_map_ref, visit_tuple_ref, visit_wrapped_value_ref,
    EnumRefDeserializer,
};
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

macro_rules! deserialize_value_ref_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> CobResult<V::Value>
        where
            V: Visitor<'de>,
        {
            match self {
                CobValue::Number(n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }
    };
}

//-------------------------------------------------------------------------------------------------------------------

/// Allows converting a [`CobValue`] to a concrete type.
impl<'de> serde::Deserializer<'de> for &'de CobValue
{
    type Error = CobError;

    fn deserialize_any<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::Enum(e) => visitor.visit_enum(EnumRefDeserializer { variant: e }),
            CobValue::Builtin(b) => deserialize_builtin(b, visitor),
            CobValue::Array(a) => visit_array_ref(&a.entries, visitor),
            CobValue::Tuple(t) => visit_tuple_ref(&t.entries, visitor),
            CobValue::Map(m) => visit_map_ref(&m.entries, visitor),
            CobValue::Number(n) => n.deserialize_any(visitor),
            CobValue::Bool(b) => visitor.visit_bool(b.value),
            CobValue::None(_) => visitor.visit_none(),
            CobValue::String(s) => visitor.visit_borrowed_str(s.as_str()),
            CobValue::Constant(_) => Err(self.invalid_type(&visitor)),
        }
    }

    deserialize_value_ref_number!(deserialize_i8);
    deserialize_value_ref_number!(deserialize_i16);
    deserialize_value_ref_number!(deserialize_i32);
    deserialize_value_ref_number!(deserialize_i64);
    deserialize_value_ref_number!(deserialize_i128);
    deserialize_value_ref_number!(deserialize_u8);
    deserialize_value_ref_number!(deserialize_u16);
    deserialize_value_ref_number!(deserialize_u32);
    deserialize_value_ref_number!(deserialize_u64);
    deserialize_value_ref_number!(deserialize_u128);
    deserialize_value_ref_number!(deserialize_f32);
    deserialize_value_ref_number!(deserialize_f64);

    fn deserialize_option<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::None(_) => visitor.visit_none(),
            _ => visitor.visit_some(self),
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
        match self {
            CobValue::Enum(variant) => visitor.visit_enum(EnumRefDeserializer { variant }),
            CobValue::Builtin(builtin) => deserialize_builtin(builtin, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::Bool(v) => visitor.visit_bool(v.value),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::String(s) => visitor.visit_borrowed_str(s.as_str()),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::Array(v) => visit_array_ref(&v.entries, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::Tuple(tuple) => {
                if tuple.entries.len() == 0 {
                    visitor.visit_unit()
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::Array(v) => visit_array_ref(&v.entries, visitor),
            CobValue::Tuple(v) => visit_tuple_ref(&v.entries, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::Tuple(tuple) => visit_tuple_ref(&tuple.entries, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        // If visiting a newtype struct, need to implicitly destructure it.
        if len == 1 {
            return visit_wrapped_value_ref(self, visitor);
        }

        match self {
            CobValue::Tuple(v) => visit_tuple_ref(&v.entries, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CobValue::Map(v) => visit_map_ref(&v.entries, visitor),
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
        match self {
            // Allow empty tuples to be treated as unit structs.
            CobValue::Tuple(tuple) => {
                if tuple.entries.len() == 0 {
                    visit_map_ref(&[], visitor)
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            CobValue::Map(v) => visit_map_ref(&v.entries, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl CobValue
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
            CobValue::Enum(e) => e.unexpected(),
            CobValue::Builtin(_) => Unexpected::Other("builtin"),
            CobValue::Array(_) => Unexpected::Seq,
            CobValue::Tuple(tuple) => {
                if tuple.entries.len() == 0 {
                    Unexpected::Unit
                } else {
                    Unexpected::Seq
                }
            }
            CobValue::Map(_) => Unexpected::Map,
            CobValue::Number(n) => n.unexpected(),
            CobValue::Bool(b) => Unexpected::Bool(b.value),
            CobValue::None(_) => Unexpected::Option,
            CobValue::String(s) => Unexpected::Str(s.as_str()),
            CobValue::Constant(_) => Unexpected::Other("constant"),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

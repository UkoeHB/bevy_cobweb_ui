use serde::de::{Expected, Unexpected, Visitor};

use super::{
    deserialize_builtin, visit_array_ref, visit_map_ref, visit_tuple_ref, visit_wrapped_value_ref,
    EnumRefDeserializer,
};
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

macro_rules! deserialize_value_ref_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> CafResult<V::Value>
        where
            V: Visitor<'de>,
        {
            match self {
                CafValue::Number(n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }
    };
}

//-------------------------------------------------------------------------------------------------------------------

/// Allows converting a [`CafValue`] to a concrete type.
impl<'de> serde::Deserializer<'de> for &'de CafValue
{
    type Error = CafError;

    fn deserialize_any<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::EnumVariant(e) => visitor.visit_enum(EnumRefDeserializer { variant: e }),
            CafValue::Builtin(b) => deserialize_builtin(b, visitor),
            CafValue::Array(a) => visit_array_ref(a, visitor),
            CafValue::Tuple(t) => visit_tuple_ref(t, visitor),
            CafValue::Map(m) => visit_map_ref(m, visitor),
            CafValue::Number(n) => n.deserialize_any(visitor),
            CafValue::Bool(b) => visitor.visit_bool(b.value),
            CafValue::None(_) => visitor.visit_none(),
            CafValue::String(s) => visitor.visit_borrowed_str(s.as_str()),
            CafValue::FlattenGroup(_) => Err(self.invalid_type(&visitor)),
            CafValue::Constant(_) => Err(self.invalid_type(&visitor)),
            CafValue::DataMacro(_) => Err(self.invalid_type(&visitor)),
            CafValue::MacroParam(_) => Err(self.invalid_type(&visitor)),
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

    fn deserialize_option<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::None(_) => visitor.visit_none(),
            _ => visitor.visit_some(self),
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
        match self {
            CafValue::EnumVariant(variant) => visitor.visit_enum(EnumRefDeserializer { variant }),
            CafValue::Builtin(builtin) => deserialize_builtin(builtin, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::Bool(v) => visitor.visit_bool(v.value),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::String(s) => visitor.visit_borrowed_str(s.as_str()),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::Array(v) => visit_array_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("unit val");
        match self {
            CafValue::Tuple(tuple) => {
                if tuple.entries.len() == 0 {
                    visitor.visit_unit()
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("unit struct val");
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::Array(v) => visit_array_ref(v, visitor),
            CafValue::Tuple(v) => visit_tuple_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::Tuple(tuple) => {
                if tuple.entries.len() == 0 {
                    Err(self.invalid_type(&visitor))
                } else {
                    visit_tuple_ref(tuple, visitor)
                }
            }
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("tuple struct val");

        // If visiting a newtype struct, need to implicitly destructure it.
        if len == 1 {
            return visit_wrapped_value_ref(self, visitor);
        }

        match self {
            CafValue::Tuple(v) => visit_tuple_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            CafValue::Map(v) => visit_map_ref(v, visitor),
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
        println!("struct val");
        match self {
            // Allow empty tuples to be treated as unit structs.
            CafValue::Tuple(tuple) => {
                if tuple.entries.len() == 0 {
                    visit_tuple_ref(tuple, visitor)
                } else {
                    Err(self.invalid_type(&visitor))
                }
            }
            CafValue::Map(v) => visit_map_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl CafValue
{
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        println!("{:?}", self);
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected
    {
        println!("{:?}", self);
        match self {
            CafValue::EnumVariant(e) => e.unexpected(),
            CafValue::Builtin(_) => Unexpected::Other("builtin"),
            CafValue::Array(_) => Unexpected::Seq,
            CafValue::Tuple(tuple) => {
                if tuple.entries.len() == 0 {
                    Unexpected::Unit
                } else {
                    Unexpected::Seq
                }
            }
            CafValue::Map(_) => Unexpected::Map,
            CafValue::FlattenGroup(_) => Unexpected::Other("flatten group"),
            CafValue::Number(n) => n.unexpected(),
            CafValue::Bool(b) => Unexpected::Bool(b.value),
            CafValue::None(_) => Unexpected::Option,
            CafValue::String(s) => Unexpected::Str(s.as_str()),
            CafValue::Constant(_) => Unexpected::Other("constant"),
            CafValue::DataMacro(_) => Unexpected::Other("data macro"),
            CafValue::MacroParam(_) => Unexpected::Other("macro param"),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

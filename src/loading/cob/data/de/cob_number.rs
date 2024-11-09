use serde::de::{Unexpected, Visitor};
use serde::{forward_to_deserialize_any, Deserializer};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl CobNumber
{
    #[cold]
    pub(super) fn unexpected(&self) -> Unexpected
    {
        match self.number {
            CobNumberValue::Uint(val) => Unexpected::Unsigned(val as u64),
            CobNumberValue::Int(val) => Unexpected::Signed(val as i64),
            CobNumberValue::Float64(val) => Unexpected::Float(val),
            CobNumberValue::Float32(val) => Unexpected::Float(val as f64),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

macro_rules! deserialize_any {
    (@expand [$($num_string:tt)*]) => {
        #[inline]
        fn deserialize_any<V>(self, visitor: V) -> CobResult<V::Value>
        where
            V: Visitor<'de>,
        {
            // TODO: might need to implement the specific number hints separately in case we need to coerce
            // an int to float
            match self.number {
                CobNumberValue::Uint(val) => {
                    // Simplify to u64 if possible, in case u128 is unsupported by the visitor.
                    if val <= u64::MAX as u128 {
                        visitor.visit_u64(val as u64)
                    } else {
                        visitor.visit_u128(val)
                    }
                }
                CobNumberValue::Int(val) => {
                    // Simplify to i64 if possible, in case i128 is unsupported by the visitor.
                    if val >= i64::MIN as i128 && val <= i64::MAX as i128 {
                        visitor.visit_i64(val as i64)
                    } else {
                        visitor.visit_i128(val)
                    }
                }
                CobNumberValue::Float64(val) => visitor.visit_f64(val),
                CobNumberValue::Float32(val) => visitor.visit_f32(val),
            }
        }
    };

    (owned) => {
        deserialize_any!(@expand [n]);
    };

    (ref) => {
        deserialize_any!(@expand [n.clone()]);
    };
}

//-------------------------------------------------------------------------------------------------------------------

macro_rules! deserialize_number {
    ($deserialize:ident) => {
        fn $deserialize<V>(self, visitor: V) -> CobResult<V::Value>
        where
            V: Visitor<'de>,
        {
            self.deserialize_any(visitor)
        }
    };
}

//-------------------------------------------------------------------------------------------------------------------

impl<'de> Deserializer<'de> for CobNumber
{
    type Error = CobError;

    deserialize_any!(owned);

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_i128);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_u128);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    forward_to_deserialize_any! {
        bool char str string bytes byte_buf option unit unit_struct
        newtype_struct seq tuple tuple_struct map struct enum identifier
        ignored_any
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<'de, 'a> Deserializer<'de> for &'a CobNumber
{
    type Error = CobError;

    deserialize_any!(ref);

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_i128);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_u128);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    forward_to_deserialize_any! {
        bool char str string bytes byte_buf option unit unit_struct
        newtype_struct seq tuple tuple_struct map struct enum identifier
        ignored_any
    }
}

//-------------------------------------------------------------------------------------------------------------------

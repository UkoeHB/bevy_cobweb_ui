use serde::de::{Unexpected, Visitor};
use serde::{forward_to_deserialize_any, Deserializer};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl CafNumber
{
    #[cold]
    pub(super) fn unexpected(&self) -> Unexpected
    {
        if self.number.deserialized.is_f64() {
            Unexpected::Float(self.number.deserialized.as_f64().unwrap())
        } else if self.number.deserialized.is_u64() {
            Unexpected::Unsigned(self.number.deserialized.as_u64().unwrap())
        } else if self.number.deserialized.is_i64() {
            Unexpected::Signed(self.number.deserialized.as_i64().unwrap())
        } else {
            unreachable!();
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

macro_rules! deserialize_any {
    (@expand [$($num_string:tt)*]) => {
        #[inline]
        fn deserialize_any<V>(self, visitor: V) -> CafResult<V::Value>
        where
            V: Visitor<'de>,
        {
            if self.number.deserialized.is_f64() {
                visitor.visit_f64(self.number.deserialized.as_f64().unwrap())
            } else if self.number.deserialized.is_u64() {
                visitor.visit_u64(self.number.deserialized.as_u64().unwrap())
            } else if self.number.deserialized.is_i64() {
                visitor.visit_i64(self.number.deserialized.as_i64().unwrap())
            } else {
                unreachable!();
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
        fn $deserialize<V>(self, visitor: V) -> CafResult<V::Value>
        where
            V: Visitor<'de>,
        {
            self.deserialize_any(visitor)
        }
    };
}

//-------------------------------------------------------------------------------------------------------------------

impl<'de> Deserializer<'de> for CafNumber
{
    type Error = CafError;

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

impl<'de, 'a> Deserializer<'de> for &'a CafNumber
{
    type Error = CafError;

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

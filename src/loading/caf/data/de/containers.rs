use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Unexpected, Visitor};
use serde::forward_to_deserialize_any;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl<'de> serde::Deserializer<'de> for &'de CafArray
{
    type Error = CafError;

    fn deserialize_any<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_array_ref(self, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
        enum newtype_struct unit_struct tuple_struct struct
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<'de> serde::Deserializer<'de> for &'de CafTuple
{
    type Error = CafError;

    fn deserialize_any<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_tuple_ref(self, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
        enum newtype_struct unit_struct tuple_struct struct
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<'de> serde::Deserializer<'de> for &'de CafMap
{
    type Error = CafError;

    fn deserialize_any<V>(self, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_map_ref(self, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
        enum newtype_struct unit_struct tuple_struct struct
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_array_ref<'de, V>(array: &'de CafArray, visitor: V) -> CafResult<V::Value>
where
    V: Visitor<'de>,
{
    let len = array.entries.len();
    let mut deserializer = SeqRefDeserializer::new(&array.entries);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_wrapped_array_ref<'de, V>(array: &'de CafArray, visitor: V) -> CafResult<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = WrappedArrayRefDeserializer { arr: Some(array) };
    let seq = visitor.visit_seq(&mut deserializer)?;
    if deserializer.arr.is_none() {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(1, &"no wrapped array"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_tuple_ref<'de, V>(tuple: &'de CafTuple, visitor: V) -> CafResult<V::Value>
where
    V: Visitor<'de>,
{
    let len = tuple.entries.len();
    let mut deserializer = SeqRefDeserializer::new(&tuple.entries);
    println!("visit tuple {tuple:?}");
    let seq = visitor.visit_seq(&mut deserializer)?;
    println!("visit tuple success");
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(len, &"fewer elements in tuple"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_map_ref<'de, V>(map: &'de CafMap, visitor: V) -> CafResult<V::Value>
where
    V: Visitor<'de>,
{
    let len = map.entries.len();
    let mut deserializer = MapRefDeserializer::new(&map.entries);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(len, &"fewer elements in map"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_wrapped_value_ref<'de, V>(val: &'de CafValue, visitor: V) -> CafResult<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = WrappedValueRefDeserializer { val: Some(val) };
    let seq = visitor.visit_seq(&mut deserializer)?;
    if deserializer.val.is_none() {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(1, &"no wrapped val"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct SeqRefDeserializer<'de>
{
    iter: std::slice::Iter<'de, CafValue>,
}

impl<'de> SeqRefDeserializer<'de>
{
    fn new(slice: &'de [CafValue]) -> Self
    {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de>
{
    type Error = CafError;

    fn next_element_seed<T>(&mut self, seed: T) -> CafResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapRefDeserializer<'de>
{
    iter: <&'de [CafMapEntry] as IntoIterator>::IntoIter,
    value: Option<&'de CafValue>,
}

impl<'de> MapRefDeserializer<'de>
{
    pub(super) fn new(entries: &'de [CafMapEntry]) -> Self
    {
        MapRefDeserializer { iter: entries.into_iter(), value: None }
    }
}

impl<'de> MapAccess<'de> for MapRefDeserializer<'de>
{
    type Error = CafError;

    fn next_key_seed<T>(&mut self, seed: T) -> CafResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(CafMapEntry::KeyValue(CafMapKeyValue { key, value, .. })) => {
                self.value = Some(value);
                match key {
                    CafMapKey::FieldName { name, .. } => {
                        let name = name.as_str().into_deserializer();
                        seed.deserialize(name).map(Some)
                    }
                    CafMapKey::Value(value) => seed.deserialize(value).map(Some),
                }
            }
            None => Ok(None),
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Other(&"unresolved map entry"),
                &"map entry with key/value",
            )),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> CafResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("map value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct WrappedArrayRefDeserializer<'de>
{
    arr: Option<&'de CafArray>,
}

impl<'de> SeqAccess<'de> for WrappedArrayRefDeserializer<'de>
{
    type Error = CafError;

    fn next_element_seed<T>(&mut self, seed: T) -> CafResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.arr.take() {
            Some(arr) => seed.deserialize(arr).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        if self.arr.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct WrappedValueRefDeserializer<'de>
{
    val: Option<&'de CafValue>,
}

impl<'de> SeqAccess<'de> for WrappedValueRefDeserializer<'de>
{
    type Error = CafError;

    fn next_element_seed<T>(&mut self, seed: T) -> CafResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.val.take() {
            Some(val) => seed.deserialize(val).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        if self.val.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl<'de> serde::Deserializer<'de> for &'de CobArray
{
    type Error = CobError;

    fn deserialize_any<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_array_ref(&self.entries, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
        enum newtype_struct unit_struct tuple_struct struct
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<'de> serde::Deserializer<'de> for &'de CobTuple
{
    type Error = CobError;

    fn deserialize_any<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_tuple_ref(&self.entries, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
        enum newtype_struct unit_struct tuple_struct struct
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<'de> serde::Deserializer<'de> for &'de CobMap
{
    type Error = CobError;

    fn deserialize_any<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_map_ref(&self.entries, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
        enum newtype_struct unit_struct tuple_struct struct
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Corresponds to a type that was erased from a newtype struct or variant because it, or something it ultimately
/// wraps, is zero length.
///
/// Note that `A(Some(()))` can be erased to `A`, but not `A(None)`.
pub(super) struct ErasedNewtypeStruct;

impl<'de> serde::Deserializer<'de> for ErasedNewtypeStruct
{
    type Error = CobError;

    fn deserialize_any<V>(self, _visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(CobError::UnresolvedNewtypeStruct)
    }

    fn deserialize_option<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_array_ref(&[], visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if len == 0 {
            visit_tuple_ref(&[], visitor)
        } else if len == 1 {
            visit_wrapped_erased_ref(visitor)
        } else {
            Err(CobError::UnresolvedNewtypeStruct)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if len == 0 {
            visit_tuple_ref(&[], visitor)
        } else if len == 1 {
            visit_wrapped_erased_ref(visitor)
        } else {
            Err(CobError::UnresolvedNewtypeStruct)
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_map_ref(&[], visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visit_wrapped_erased_ref(visitor)
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
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
        let mut deserializer = MapRefDeserializer::new(&[]);
        visitor.visit_map(&mut deserializer)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf
        identifier ignored_any
        enum
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_array_ref<'de, V>(array: &'de [CobValue], visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqRefDeserializer::new(array);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(len, &"fewer elements in array"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_tuple_ref<'de, V>(tuple: &'de [CobValue], visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    let len = tuple.len();
    let mut deserializer = SeqRefDeserializer::new(tuple);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(len, &"fewer elements in tuple"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_map_ref<'de, V>(map: &'de [CobMapEntry], visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    let len = map.len();
    let mut deserializer = MapRefDeserializer::new(map);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(len, &"fewer elements in map"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_wrapped_array_ref<'de, V>(array: &'de CobArray, visitor: V) -> CobResult<V::Value>
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

pub(super) fn visit_wrapped_tuple_ref<'de, V>(tuple: &'de CobTuple, visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = WrappedTupleRefDeserializer { tuple: Some(tuple) };
    let seq = visitor.visit_seq(&mut deserializer)?;
    if deserializer.tuple.is_none() {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(1, &"no wrapped tuple"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_wrapped_map_ref<'de, V>(map: &'de CobMap, visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = WrappedMapRefDeserializer { map: Some(map) };
    let seq = visitor.visit_seq(&mut deserializer)?;
    if deserializer.map.is_none() {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(1, &"no wrapped map"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_wrapped_erased_ref<'de, V>(visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = WrappedErasedRefDeserializer { visited: false };
    let seq = visitor.visit_seq(&mut deserializer)?;
    if deserializer.visited {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(1, &"no wrapped erased value"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_wrapped_value_ref<'de, V>(val: &'de CobValue, visitor: V) -> CobResult<V::Value>
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
    iter: std::slice::Iter<'de, CobValue>,
}

impl<'de> SeqRefDeserializer<'de>
{
    fn new(slice: &'de [CobValue]) -> Self
    {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de>
{
    type Error = CobError;

    fn next_element_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
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
    iter: <&'de [CobMapEntry] as IntoIterator>::IntoIter,
    value: Option<&'de CobValue>,
}

impl<'de> MapRefDeserializer<'de>
{
    pub(super) fn new(entries: &'de [CobMapEntry]) -> Self
    {
        MapRefDeserializer { iter: entries.into_iter(), value: None }
    }
}

impl<'de> MapAccess<'de> for MapRefDeserializer<'de>
{
    type Error = CobError;

    fn next_key_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(CobMapEntry::KeyValue(CobMapKeyValue { key, value, .. })) => {
                self.value = Some(value);
                match key {
                    CobMapKey::FieldName { name, .. } => {
                        let name = name.as_str().into_deserializer();
                        seed.deserialize(name).map(Some)
                    }
                    CobMapKey::Value(value) => seed.deserialize(value).map(Some),
                }
            }
            None => Ok(None),
            #[cfg(feature = "full")]
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Other(&"unresolved map entry"),
                &"map entry with key/value",
            )),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> CobResult<T::Value>
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
    arr: Option<&'de CobArray>,
}

impl<'de> SeqAccess<'de> for WrappedArrayRefDeserializer<'de>
{
    type Error = CobError;

    fn next_element_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
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

struct WrappedTupleRefDeserializer<'de>
{
    tuple: Option<&'de CobTuple>,
}

impl<'de> SeqAccess<'de> for WrappedTupleRefDeserializer<'de>
{
    type Error = CobError;

    fn next_element_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.tuple.take() {
            Some(tuple) => seed.deserialize(tuple).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        if self.tuple.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct WrappedMapRefDeserializer<'de>
{
    map: Option<&'de CobMap>,
}

impl<'de> SeqAccess<'de> for WrappedMapRefDeserializer<'de>
{
    type Error = CobError;

    fn next_element_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.map.take() {
            Some(map) => seed.deserialize(map).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        if self.map.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct WrappedErasedRefDeserializer
{
    visited: bool,
}

impl<'de> SeqAccess<'de> for WrappedErasedRefDeserializer
{
    type Error = CobError;

    fn next_element_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.visited {
            false => {
                self.visited = true;
                seed.deserialize(ErasedNewtypeStruct).map(Some)
            }
            true => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        if !self.visited {
            Some(1)
        } else {
            Some(0)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct WrappedValueRefDeserializer<'de>
{
    val: Option<&'de CobValue>,
}

impl<'de> SeqAccess<'de> for WrappedValueRefDeserializer<'de>
{
    type Error = CobError;

    fn next_element_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
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

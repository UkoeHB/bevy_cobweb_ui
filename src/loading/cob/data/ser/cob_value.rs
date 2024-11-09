use serde::Serialize;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Allows constructing a [`CobValue`] from any serializable rust type `T`.
pub struct CobValueSerializer;

impl serde::Serializer for CobValueSerializer
{
    type Ok = CobValue;
    type Error = CobError;

    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeTuple;
    type SerializeTupleStruct = SerializeTuple;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> CobResult<CobValue>
    {
        Ok(CobValue::Bool(CobBool::from(value)))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    fn serialize_i64(self, value: i64) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    fn serialize_i128(self, value: i128) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    fn serialize_u128(self, value: u128) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> CobResult<CobValue>
    {
        Ok(CobValue::Number(CobNumber::from(value)))
    }

    #[inline]
    fn serialize_char(self, value: char) -> CobResult<CobValue>
    {
        Ok(CobValue::String(CobString::from(value)))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> CobResult<CobValue>
    {
        Ok(CobValue::String(CobString::from(value)))
    }

    fn serialize_bytes(self, value: &[u8]) -> CobResult<CobValue>
    {
        let vec: Vec<CobValue> = value
            .iter()
            .map(|&b| CobValue::Number(CobNumber::from(b)))
            .collect();
        Ok(CobValue::Array(CobArray::from(vec)))
    }

    #[inline]
    fn serialize_unit(self) -> CobResult<CobValue>
    {
        Ok(CobValue::Tuple(CobTuple::from(vec![])))
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> CobResult<CobValue>
    {
        Ok(CobValue::Tuple(CobTuple::from(vec![])))
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> CobResult<CobValue>
    {
        if let Some(result) = CobBuiltin::try_from_unit_variant(name, variant)? {
            return Ok(CobValue::Builtin(result));
        }
        Ok(CobValue::Enum(CobEnum::unit(variant)))
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> CobResult<CobValue>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> CobResult<CobValue>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value so we know what to do with it.
        // TODO: for builtin types this feels super unnecessary, but rust sucks and doesn't have
        // 'if constexpr' OR specialization OR any way to get a unique type ID for non-static types.
        let value_ser = value.serialize(self)?;

        // Check for built-in type.
        if let Some(result) = CobBuiltin::try_from_newtype_variant(name, variant, &value_ser)? {
            return Ok(CobValue::Builtin(result));
        }

        if let CobValue::Array(array) = value_ser {
            if array.entries.len() == 0 {
                Ok(CobValue::Enum(CobEnum::unit(variant)))
            } else {
                Ok(CobValue::Enum(CobEnum::array(variant, array)))
            }
        } else if let CobValue::Tuple(tuple) = value_ser {
            if tuple.entries.len() == 0 {
                Ok(CobValue::Enum(CobEnum::unit(variant)))
            } else {
                Ok(CobValue::Enum(CobEnum::tuple(variant, tuple)))
            }
        } else if let CobValue::Map(map) = value_ser {
            if map.entries.len() == 0 {
                Ok(CobValue::Enum(CobEnum::unit(variant)))
            } else {
                Ok(CobValue::Enum(CobEnum::map(variant, map)))
            }
        } else {
            Ok(CobValue::Enum(CobEnum::newtype(variant, value_ser)))
        }
    }

    #[inline]
    fn serialize_none(self) -> CobResult<CobValue>
    {
        Ok(CobValue::None(CobNone::default()))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> CobResult<CobValue>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> CobResult<Self::SerializeSeq>
    {
        Ok(SerializeSeq { vec: Vec::with_capacity(len.unwrap_or(0)) })
    }

    fn serialize_tuple(self, len: usize) -> CobResult<Self::SerializeTuple>
    {
        Ok(SerializeTuple { vec: Vec::with_capacity(len) })
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> CobResult<Self::SerializeTupleStruct>
    {
        Self::serialize_tuple(self, len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CobResult<Self::SerializeTupleVariant>
    {
        Ok(SerializeTupleVariant { variant, vec: Vec::with_capacity(len) })
    }

    fn serialize_map(self, len: Option<usize>) -> CobResult<Self::SerializeMap>
    {
        Ok(SerializeMap { vec: Vec::with_capacity(len.unwrap_or(0)), next_key: None })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> CobResult<Self::SerializeStruct>
    {
        Ok(SerializeStruct { vec: Vec::with_capacity(len) })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CobResult<Self::SerializeStructVariant>
    {
        Ok(SerializeStructVariant { variant, vec: Vec::with_capacity(len) })
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeSeq
{
    vec: Vec<CobValue>,
}

impl serde::ser::SerializeSeq for SerializeSeq
{
    type Ok = CobValue;
    type Error = CobError;

    fn serialize_element<T>(&mut self, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CobValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CobResult<CobValue>
    {
        Ok(CobValue::Array(CobArray::from(self.vec)))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTuple
{
    vec: Vec<CobValue>,
}

impl serde::ser::SerializeTuple for SerializeTuple
{
    type Ok = CobValue;
    type Error = CobError;

    fn serialize_element<T>(&mut self, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CobValue::extract(value)?);
        Ok(())
    }

    fn end(mut self) -> CobResult<CobValue>
    {
        if self.vec.len() == 1 {
            Ok(self.vec.drain(..).next().unwrap())
        } else {
            Ok(CobValue::Tuple(CobTuple::from(self.vec)))
        }
    }
}

impl serde::ser::SerializeTupleStruct for SerializeTuple
{
    type Ok = CobValue;
    type Error = CobError;

    fn serialize_field<T>(&mut self, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> CobResult<CobValue>
    {
        serde::ser::SerializeTuple::end(self)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTupleVariant
{
    variant: &'static str,
    vec: Vec<CobValue>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant
{
    type Ok = CobValue;
    type Error = CobError;

    fn serialize_field<T>(&mut self, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CobValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CobResult<CobValue>
    {
        if self.vec.len() == 0 {
            Ok(CobValue::Enum(CobEnum::unit(self.variant)))
        } else {
            Ok(CobValue::Enum(CobEnum::tuple(self.variant, CobTuple::from(self.vec))))
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeMap
{
    vec: Vec<CobMapEntry>,
    next_key: Option<CobValue>,
}

impl serde::ser::SerializeMap for SerializeMap
{
    type Ok = CobValue;
    type Error = CobError;

    fn serialize_key<T>(&mut self, key: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(CobValue::extract(key)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.vec
            .push(CobMapEntry::map_entry(key, CobValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CobResult<CobValue>
    {
        Ok(CobValue::Map(CobMap::from(self.vec)))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStruct
{
    vec: Vec<CobMapEntry>,
}

impl serde::ser::SerializeStruct for SerializeStruct
{
    type Ok = CobValue;
    type Error = CobError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CobMapEntry::struct_field(key, CobValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CobResult<CobValue>
    {
        // Note: we don't *want* to convert to a tuple if the struct has no fields, because we want empty structs
        // to be displayed as '{}' for clarity. HOWEVER, `bevy_reflect` currently has no way to tell if a
        // type is a struct with only `#[reflect(ignore)]` fields (and doesn't give us a hint that a struct
        // is a unit struct either, all unit structs pass through here). So for simplicity we just assume
        // all structs with no members are unit structs...
        if self.vec.len() == 0 {
            Ok(CobValue::Tuple(CobTuple::from(vec![])))
        } else {
            Ok(CobValue::Map(CobMap::from(self.vec)))
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStructVariant
{
    variant: &'static str,
    vec: Vec<CobMapEntry>,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant
{
    type Ok = CobValue;
    type Error = CobError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CobMapEntry::struct_field(key, CobValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CobResult<CobValue>
    {
        if self.vec.len() > 0 {
            Ok(CobValue::Enum(CobEnum::map(self.variant, CobMap::from(self.vec))))
        } else {
            Ok(CobValue::Enum(CobEnum::unit(self.variant)))
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

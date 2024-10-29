use serde::ser::Impossible;
use serde::Serialize;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Allows constructing a [`CafLoadable`] from any serializable rust type `T` that has been registered with
/// bevy's type registry.
pub struct CafLoadableSerializer
{
    /// The loadable name is injected because serde doesn't know about generics.
    //todo: add ability to customize the loadable name e.g. in the case of 'using' statements
    pub name: &'static str,
}

impl serde::Serializer for CafLoadableSerializer
{
    type Ok = CafLoadable;
    type Error = CafError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = SerializeTupleStruct;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, _: bool) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_i8(self, _: i8) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_i16(self, _: i16) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_i32(self, _: i32) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    fn serialize_i64(self, _: i64) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    fn serialize_i128(self, _: i128) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_u8(self, _: u8) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_u16(self, _: u16) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_u32(self, _: u32) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_u64(self, _: u64) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    fn serialize_u128(self, _: u128) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_f32(self, _: f32) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_f64(self, _: f64) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_char(self, _: char) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_str(self, _: &str) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    fn serialize_bytes(self, _: &[u8]) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_unit(self) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> CafResult<CafLoadable>
    {
        Ok(CafLoadable {
            fill: CafFill::default(),
            id: self.name.try_into()?,
            variant: CafLoadableVariant::Unit,
        })
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> CafResult<CafLoadable>
    {
        Ok(CafLoadable {
            fill: CafFill::default(),
            id: self.name.try_into()?,
            variant: CafLoadableVariant::Enum(CafEnum::unit(variant)),
        })
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> CafResult<CafLoadable>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CafValue::extract(value)?;

        if let CafValue::Array(array) = value_ser {
            if array.entries.len() == 0 {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Unit,
                })
            } else {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Array(array),
                })
            }
        } else if let CafValue::Tuple(tuple) = value_ser {
            if tuple.entries.len() == 0 {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Unit,
                })
            } else {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Tuple(tuple),
                })
            }
        } else if let CafValue::Map(map) = value_ser {
            if map.entries.len() == 0 {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Unit,
                })
            } else {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Map(map),
                })
            }
        } else {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Tuple(CafTuple::single(value_ser)),
            })
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> CafResult<CafLoadable>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CafValue::extract(value)?;

        if let CafValue::Array(array) = value_ser {
            if array.entries.len() == 0 {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Enum(CafEnum::unit(variant)),
                })
            } else {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Enum(CafEnum::array(variant, array)),
                })
            }
        } else if let CafValue::Tuple(tuple) = value_ser {
            if tuple.entries.len() == 0 {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Enum(CafEnum::unit(variant)),
                })
            } else {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Enum(CafEnum::tuple(variant, tuple)),
                })
            }
        } else if let CafValue::Map(map) = value_ser {
            if map.entries.len() == 0 {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Enum(CafEnum::unit(variant)),
                })
            } else {
                Ok(CafLoadable {
                    fill: CafFill::default(),
                    id: self.name.try_into()?,
                    variant: CafLoadableVariant::Enum(CafEnum::map(variant, map)),
                })
            }
        } else {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Enum(CafEnum::newtype(variant, value_ser)),
            })
        }
    }

    #[inline]
    fn serialize_none(self) -> CafResult<CafLoadable>
    {
        Err(CafError::NotALoadable)
    }

    #[inline]
    fn serialize_some<T>(self, _: &T) -> CafResult<CafLoadable>
    where
        T: ?Sized + Serialize,
    {
        Err(CafError::NotALoadable)
    }

    fn serialize_seq(self, _: Option<usize>) -> CafResult<Self::SerializeSeq>
    {
        Err(CafError::NotALoadable)
    }

    fn serialize_tuple(self, _: usize) -> CafResult<Self::SerializeTuple>
    {
        Err(CafError::NotALoadable)
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
        Err(CafError::NotALoadable)
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
    type Ok = CafLoadable;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafLoadable>
    {
        if self.vec.len() == 0 {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Unit,
            })
        } else {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Tuple(CafTuple::from(self.vec)),
            })
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
    type Ok = CafLoadable;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafLoadable>
    {
        if self.vec.len() == 0 {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Enum(CafEnum::unit(self.variant)),
            })
        } else {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Enum(CafEnum::tuple(self.variant, CafTuple::from(self.vec))),
            })
        }
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
    type Ok = CafLoadable;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafLoadable>
    {
        if self.vec.len() == 0 {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Unit,
            })
        } else {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Map(CafMap::from(self.vec)),
            })
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
    type Ok = CafLoadable;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafLoadable>
    {
        if self.vec.len() == 0 {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Enum(CafEnum::unit(self.variant)),
            })
        } else {
            Ok(CafLoadable {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafLoadableVariant::Enum(CafEnum::map(self.variant, CafMap::from(self.vec))),
            })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

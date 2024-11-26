use serde::ser::Impossible;
use serde::Serialize;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Allows constructing a [`CobLoadable`] from any serializable rust type `T` that has been registered with
/// bevy's type registry.
pub struct CobLoadableSerializer
{
    /// The loadable name is injected because serde doesn't know about generics.
    pub name: &'static str,
}

impl serde::Serializer for CobLoadableSerializer
{
    type Ok = CobLoadable;
    type Error = CobError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = SerializeTupleStruct;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, _: bool) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_i8(self, _: i8) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_i16(self, _: i16) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_i32(self, _: i32) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_i64(self, _: i64) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_i128(self, _: i128) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_u8(self, _: u8) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_u16(self, _: u16) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_u32(self, _: u32) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_u64(self, _: u64) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_u128(self, _: u128) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_f32(self, _: f32) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_f64(self, _: f64) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_char(self, _: char) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_str(self, _: &str) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_bytes(self, _: &[u8]) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_unit(self) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> CobResult<CobLoadable>
    {
        Ok(CobLoadable {
            fill: CobFill::default(),
            id: self.name.try_into()?,
            variant: CobLoadableVariant::Unit,
        })
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> CobResult<CobLoadable>
    {
        Ok(CobLoadable {
            fill: CobFill::default(),
            id: self.name.try_into()?,
            variant: CobLoadableVariant::Enum(CobEnum::unit(variant)),
        })
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> CobResult<CobLoadable>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CobValue::extract(value)?;

        if let CobValue::Array(array) = value_ser {
            if array.entries.len() == 0 {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Unit,
                })
            } else {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Array(array),
                })
            }
        } else if let CobValue::Tuple(tuple) = value_ser {
            if tuple.entries.len() == 0 {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Unit,
                })
            } else {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Tuple(tuple),
                })
            }
        } else if let CobValue::Map(map) = value_ser {
            if map.entries.len() == 0 {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Unit,
                })
            } else {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Map(map),
                })
            }
        } else {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Tuple(CobTuple::single(value_ser)),
            })
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> CobResult<CobLoadable>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CobValue::extract(value)?;

        if let CobValue::Array(array) = value_ser {
            if array.entries.len() == 0 {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Enum(CobEnum::unit(variant)),
                })
            } else {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Enum(CobEnum::array(variant, array)),
                })
            }
        } else if let CobValue::Tuple(tuple) = value_ser {
            if tuple.entries.len() == 0 {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Enum(CobEnum::unit(variant)),
                })
            } else {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Enum(CobEnum::tuple(variant, tuple)),
                })
            }
        } else if let CobValue::Map(map) = value_ser {
            if map.entries.len() == 0 {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Enum(CobEnum::unit(variant)),
                })
            } else {
                Ok(CobLoadable {
                    fill: CobFill::default(),
                    id: self.name.try_into()?,
                    variant: CobLoadableVariant::Enum(CobEnum::map(variant, map)),
                })
            }
        } else {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Enum(CobEnum::newtype(variant, value_ser)),
            })
        }
    }

    #[inline]
    fn serialize_none(self) -> CobResult<CobLoadable>
    {
        Err(CobError::NotALoadable)
    }

    #[inline]
    fn serialize_some<T>(self, _: &T) -> CobResult<CobLoadable>
    where
        T: ?Sized + Serialize,
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_seq(self, _: Option<usize>) -> CobResult<Self::SerializeSeq>
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_tuple(self, _: usize) -> CobResult<Self::SerializeTuple>
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> CobResult<Self::SerializeTupleStruct>
    {
        Ok(SerializeTupleStruct { name: self.name, vec: Vec::with_capacity(len) })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CobResult<Self::SerializeTupleVariant>
    {
        Ok(SerializeTupleVariant { name: self.name, variant, vec: Vec::with_capacity(len) })
    }

    fn serialize_map(self, _: Option<usize>) -> CobResult<Self::SerializeMap>
    {
        Err(CobError::NotALoadable)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> CobResult<Self::SerializeStruct>
    {
        Ok(SerializeStruct { name: self.name, vec: Vec::with_capacity(len) })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CobResult<Self::SerializeStructVariant>
    {
        Ok(SerializeStructVariant { name: self.name, variant, vec: Vec::with_capacity(len) })
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTupleStruct
{
    name: &'static str,
    vec: Vec<CobValue>,
}

impl serde::ser::SerializeTupleStruct for SerializeTupleStruct
{
    type Ok = CobLoadable;
    type Error = CobError;

    fn serialize_field<T>(&mut self, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CobValue::extract(value)?);
        Ok(())
    }

    fn end(mut self) -> CobResult<CobLoadable>
    {
        let unit = || -> CobResult<CobLoadable> {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Unit,
            })
        };
        if self.vec.len() == 0 {
            (unit)()
        } else if self.vec.len() == 1 {
            let value = self.vec.drain(..).next().unwrap();
            match value {
                CobValue::Tuple(tuple) => {
                    if tuple.entries.len() == 0 {
                        (unit)()
                    } else {
                        Ok(CobLoadable {
                            fill: CobFill::default(),
                            id: self.name.try_into()?,
                            variant: CobLoadableVariant::Tuple(tuple),
                        })
                    }
                }
                CobValue::Array(array) => {
                    if array.entries.len() == 0 {
                        (unit)()
                    } else {
                        Ok(CobLoadable {
                            fill: CobFill::default(),
                            id: self.name.try_into()?,
                            variant: CobLoadableVariant::Array(array),
                        })
                    }
                }
                CobValue::Map(map) => {
                    if map.entries.len() == 0 {
                        (unit)()
                    } else {
                        Ok(CobLoadable {
                            fill: CobFill::default(),
                            id: self.name.try_into()?,
                            variant: CobLoadableVariant::Map(map),
                        })
                    }
                }
                value => {
                    self.vec.push(value);
                    Ok(CobLoadable {
                        fill: CobFill::default(),
                        id: self.name.try_into()?,
                        variant: CobLoadableVariant::Tuple(CobTuple::from(self.vec)),
                    })
                }
            }
        } else {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Tuple(CobTuple::from(self.vec)),
            })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTupleVariant
{
    name: &'static str,
    variant: &'static str,
    vec: Vec<CobValue>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant
{
    type Ok = CobLoadable;
    type Error = CobError;

    fn serialize_field<T>(&mut self, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CobValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CobResult<CobLoadable>
    {
        if self.vec.len() == 0 {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Enum(CobEnum::unit(self.variant)),
            })
        } else {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Enum(CobEnum::tuple(self.variant, CobTuple::from(self.vec))),
            })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStruct
{
    name: &'static str,
    vec: Vec<CobMapEntry>,
}

impl serde::ser::SerializeStruct for SerializeStruct
{
    type Ok = CobLoadable;
    type Error = CobError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CobMapEntry::struct_field(key, CobValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CobResult<CobLoadable>
    {
        if self.vec.len() == 0 {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Unit,
            })
        } else {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Map(CobMap::from(self.vec)),
            })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStructVariant
{
    name: &'static str,
    variant: &'static str,
    vec: Vec<CobMapEntry>,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant
{
    type Ok = CobLoadable;
    type Error = CobError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CobResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CobMapEntry::struct_field(key, CobValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CobResult<CobLoadable>
    {
        if self.vec.len() == 0 {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Enum(CobEnum::unit(self.variant)),
            })
        } else {
            Ok(CobLoadable {
                fill: CobFill::default(),
                id: self.name.try_into()?,
                variant: CobLoadableVariant::Enum(CobEnum::map(self.variant, CobMap::from(self.vec))),
            })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

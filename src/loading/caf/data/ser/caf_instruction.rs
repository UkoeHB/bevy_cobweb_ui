use serde::ser::Impossible;
use serde::Serialize;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Allows constructing a [`CafInstruction`] from any serializable rust type `T` that has been registered with
/// bevy's type registry.
pub struct CafInstructionSerializer
{
    /// The instruction name is injected because serde doesn't know about generics.
    //todo: add ability to customize the instruction name e.g. in the case of 'using' statements
    pub name: &'static str,
}

impl serde::Serializer for CafInstructionSerializer
{
    type Ok = CafInstruction;
    type Error = CafError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = SerializeTupleStruct;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, _: bool) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_i8(self, _: i8) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_i16(self, _: i16) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_i32(self, _: i32) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_i64(self, _: i64) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_i128(self, _: i128) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u8(self, _: u8) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u16(self, _: u16) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u32(self, _: u32) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_u64(self, _: u64) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_u128(self, _: u128) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_f32(self, _: f32) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_f64(self, _: f64) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_char(self, _: char) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_str(self, _: &str) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_bytes(self, _: &[u8]) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_unit(self) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> CafResult<CafInstruction>
    {
        Ok(CafInstruction {
            fill: CafFill::default(),
            id: self.name.try_into()?,
            variant: CafInstructionVariant::Unit,
        })
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> CafResult<CafInstruction>
    {
        Ok(CafInstruction {
            fill: CafFill::default(),
            id: self.name.try_into()?,
            variant: CafInstructionVariant::Enum(CafEnumVariant::unit(variant)),
        })
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> CafResult<CafInstruction>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CafValue::extract(value)?;

        if let CafValue::Array(array) = value_ser {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Array(array),
            })
        } else {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Tuple(CafTuple::single(value_ser)),
            })
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> CafResult<CafInstruction>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value first so we know what to do with it.
        let value_ser = CafValue::extract(value)?;

        if let CafValue::Array(arr) = value_ser {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Enum(CafEnumVariant::array(variant, arr)),
            })
        } else {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Enum(CafEnumVariant::newtype(variant, value_ser)),
            })
        }
    }

    #[inline]
    fn serialize_none(self) -> CafResult<CafInstruction>
    {
        Err(CafError::NotAnInstruction)
    }

    #[inline]
    fn serialize_some<T>(self, _: &T) -> CafResult<CafInstruction>
    where
        T: ?Sized + Serialize,
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_seq(self, _: Option<usize>) -> CafResult<Self::SerializeSeq>
    {
        Err(CafError::NotAnInstruction)
    }

    fn serialize_tuple(self, _: usize) -> CafResult<Self::SerializeTuple>
    {
        Err(CafError::NotAnInstruction)
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
        Err(CafError::NotAnInstruction)
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
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        if self.vec.len() > 0 {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Tuple(CafTuple::from(self.vec)),
            })
        } else {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Unit,
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
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        Ok(CafInstruction {
            fill: CafFill::default(),
            id: self.name.try_into()?,
            variant: CafInstructionVariant::Enum(CafEnumVariant::tuple(self.variant, CafTuple::from(self.vec))),
        })
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
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        if self.vec.len() > 0 {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Map(CafMap::from(self.vec)),
            })
        } else {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Unit,
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
    type Ok = CafInstruction;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafInstruction>
    {
        if self.vec.len() > 0 {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Enum(CafEnumVariant::map(self.variant, CafMap::from(self.vec))),
            })
        } else {
            Ok(CafInstruction {
                fill: CafFill::default(),
                id: self.name.try_into()?,
                variant: CafInstructionVariant::Enum(CafEnumVariant::unit(self.variant)),
            })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

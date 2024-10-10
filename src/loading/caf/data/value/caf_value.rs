use serde::Serialize;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafValue
{
    EnumVariant(CafEnumVariant),
    /// Special built-in types like `none` and `#FFFFFF` for colors.
    Builtin(CafBuiltin),
    Array(CafArray),
    Tuple(CafTuple),
    Map(CafMap),
    FlattenGroup(CafFlattenGroup),
    Number(CafNumber),
    Bool(CafBool),
    None(CafNone),
    String(CafString),
    Constant(CafConstant),
    DataMacro(CafDataMacroCall),
    /// Only valid inside a macro definition.
    MacroParam(CafMacroParam),
}

impl CafValue
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str)
        -> Result<(), std::io::Error>
    {
        match self {
            Self::EnumVariant(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Builtin(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Array(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Tuple(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Map(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::FlattenGroup(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Number(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Bool(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::None(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::String(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Constant(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::DataMacro(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::MacroParam(val) => {
                val.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        match self {
            Self::EnumVariant(val) => val.to_json(),
            Self::Builtin(val) => val.to_json(),
            Self::Array(val) => val.to_json(),
            Self::Tuple(val) => val.to_json_for_type(),
            Self::Map(val) => val.to_json(),
            Self::FlattenGroup(val) => Err(std::io::Error::other(
                format!("cannot convert flatten group {val:?} to JSON"),
            )),
            Self::Number(val) => val.to_json(),
            Self::Bool(val) => val.to_json(),
            Self::None(val) => val.to_json(),
            Self::String(val) => val.to_json(),
            Self::Constant(val) => Err(std::io::Error::other(
                format!("cannot convert constant {val:?} to JSON"),
            )),
            Self::DataMacro(val) => Err(std::io::Error::other(
                format!("cannot convert data macro {val:?} to JSON"),
            )),
            Self::MacroParam(val) => Err(std::io::Error::other(
                format!("cannot convert macro param {val:?} to JSON"),
            )),
        }
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::EnumVariant(val), Self::EnumVariant(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Builtin(val), Self::Builtin(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Array(val), Self::Array(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Tuple(val), Self::Tuple(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Map(val), Self::Map(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::FlattenGroup(val), Self::FlattenGroup(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Number(val), Self::Number(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Bool(val), Self::Bool(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::None(val), Self::None(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::String(val), Self::String(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Constant(val), Self::Constant(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::DataMacro(val), Self::DataMacro(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::MacroParam(val), Self::MacroParam(other_val)) => {
                val.recover_fill(other_val);
            }
            _ => (),
        }
    }

    pub fn extract<T: ?Sized + Serialize>(value: &T) -> CafResult<Self>
    {
        value.serialize(CafValueSerializer)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct CafValueSerializer;

impl serde::Serializer for CafValueSerializer
{
    type Ok = CafValue;
    type Error = CafError;

    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeTuple;
    type SerializeTupleStruct = SerializeTuple;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> CafResult<CafValue>
    {
        Ok(CafValue::Bool(CafBool::from(value)))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    fn serialize_i64(self, value: i64) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    fn serialize_i128(self, value: i128) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    fn serialize_u128(self, value: u128) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> CafResult<CafValue>
    {
        Ok(CafValue::Number(CafNumber::from(value)))
    }

    #[inline]
    fn serialize_char(self, value: char) -> CafResult<CafValue>
    {
        Ok(CafValue::String(CafString::try_from(value)?))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> CafResult<CafValue>
    {
        Ok(CafValue::String(CafString::try_from(value)?))
    }

    fn serialize_bytes(self, value: &[u8]) -> CafResult<CafValue>
    {
        let vec: Vec<CafValue> = value
            .iter()
            .map(|&b| CafValue::Number(CafNumber::from(b)))
            .collect();
        Ok(CafValue::Array(CafArray::from(vec)))
    }

    #[inline]
    fn serialize_unit(self) -> CafResult<CafValue>
    {
        Ok(CafValue::Tuple(CafTuple::from(vec![])))
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> CafResult<CafValue>
    {
        Ok(CafValue::Map(CafMap::from(vec![])))
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> CafResult<CafValue>
    {
        if let Some(result) = CafBuiltin::try_from_unit_variant(name, variant)? {
            return Ok(CafValue::Builtin(result));
        }
        Ok(CafValue::EnumVariant(CafEnumVariant::unit(variant)))
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> CafResult<CafValue>
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
    ) -> CafResult<CafValue>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the value so we know what to do with it.
        // TODO: for builtin types this feels super unnecessary, but rust sucks and doesn't have
        // 'if constexpr' OR specialization OR any way to get a unique type ID for non-static types.
        let value_ser = value.serialize(self)?;

        // Check for built-in type.
        if let Some(result) = CafBuiltin::try_from_newtype_variant(name, variant, &value_ser)? {
            return Ok(CafValue::Builtin(result));
        }

        if let CafValue::Array(arr) = value_ser {
            Ok(CafValue::EnumVariant(CafEnumVariant::array(variant, arr)))
        } else {
            Ok(CafValue::EnumVariant(CafEnumVariant::newtype(variant, value_ser)))
        }
    }

    #[inline]
    fn serialize_none(self) -> CafResult<CafValue>
    {
        Ok(CafValue::None(CafNone::default()))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> CafResult<CafValue>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> CafResult<Self::SerializeSeq>
    {
        Ok(SerializeSeq { vec: Vec::with_capacity(len.unwrap_or(0)) })
    }

    fn serialize_tuple(self, len: usize) -> CafResult<Self::SerializeTuple>
    {
        Ok(SerializeTuple { vec: Vec::with_capacity(len) })
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> CafResult<Self::SerializeTupleStruct>
    {
        Self::serialize_tuple(self, len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CafResult<Self::SerializeTupleVariant>
    {
        Ok(SerializeTupleVariant { variant, vec: Vec::with_capacity(len) })
    }

    fn serialize_map(self, len: Option<usize>) -> CafResult<Self::SerializeMap>
    {
        Ok(SerializeMap { vec: Vec::with_capacity(len.unwrap_or(0)), next_key: None })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> CafResult<Self::SerializeStruct>
    {
        Ok(SerializeStruct { vec: Vec::with_capacity(len) })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> CafResult<Self::SerializeStructVariant>
    {
        Ok(SerializeStructVariant { variant, vec: Vec::with_capacity(len) })
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeSeq
{
    vec: Vec<CafValue>,
}

impl serde::ser::SerializeSeq for SerializeSeq
{
    type Ok = CafValue;
    type Error = CafError;

    fn serialize_element<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafValue>
    {
        Ok(CafValue::Array(CafArray::from(self.vec)))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTuple
{
    vec: Vec<CafValue>,
}

impl serde::ser::SerializeTuple for SerializeTuple
{
    type Ok = CafValue;
    type Error = CafError;

    fn serialize_element<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafValue>
    {
        Ok(CafValue::Tuple(CafTuple::from(self.vec)))
    }
}

impl serde::ser::SerializeTupleStruct for SerializeTuple
{
    type Ok = CafValue;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> CafResult<CafValue>
    {
        serde::ser::SerializeTuple::end(self)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeTupleVariant
{
    variant: &'static str,
    vec: Vec<CafValue>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant
{
    type Ok = CafValue;
    type Error = CafError;

    fn serialize_field<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(CafValue::extract(value)?);
        Ok(())
    }

    fn end(self) -> CafResult<CafValue>
    {
        if self.vec.len() > 0 {
            Ok(CafValue::EnumVariant(CafEnumVariant::tuple(
                self.variant,
                CafTuple::from(self.vec),
            )))
        } else {
            Ok(CafValue::EnumVariant(CafEnumVariant::unit(self.variant)))
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeMap
{
    vec: Vec<CafMapEntry>,
    next_key: Option<CafValue>,
}

impl serde::ser::SerializeMap for SerializeMap
{
    type Ok = CafValue;
    type Error = CafError;

    fn serialize_key<T>(&mut self, key: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(CafValue::extract(key)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.vec
            .push(CafMapEntry::map_entry(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafValue>
    {
        Ok(CafValue::Map(CafMap::from(self.vec)))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStruct
{
    vec: Vec<CafMapEntry>,
}

impl serde::ser::SerializeStruct for SerializeStruct
{
    type Ok = CafValue;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafValue>
    {
        Ok(CafValue::Map(CafMap::from(self.vec)))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SerializeStructVariant
{
    variant: &'static str,
    vec: Vec<CafMapEntry>,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant
{
    type Ok = CafValue;
    type Error = CafError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> CafResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec
            .push(CafMapEntry::struct_field(key, CafValue::extract(value)?));
        Ok(())
    }

    fn end(self) -> CafResult<CafValue>
    {
        if self.vec.len() > 0 {
            Ok(CafValue::EnumVariant(CafEnumVariant::map(
                self.variant,
                CafMap::from(self.vec),
            )))
        } else {
            Ok(CafValue::EnumVariant(CafEnumVariant::unit(self.variant)))
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::reflect::{TypeInfo, TypeRegistry};

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

    pub fn from_json(
        val: &serde_json::Value,
        type_info: &TypeInfo,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        match type_info {
            TypeInfo::Struct(_) => Ok(Self::Map(CafMap::from_json_as_type(val, type_info, registry)?)),
            TypeInfo::TupleStruct(_) => Ok(Self::Tuple(CafTuple::from_json_as_type(val, type_info, registry)?)),
            TypeInfo::Tuple(_) => Ok(Self::Tuple(CafTuple::from_json_as_type(val, type_info, registry)?)),
            TypeInfo::List(_) => Ok(Self::Array(CafArray::from_json(val, type_info, registry)?)),
            TypeInfo::Array(_) => Ok(Self::Array(CafArray::from_json(val, type_info, registry)?)),
            TypeInfo::Map(_) => Ok(Self::Map(CafMap::from_json_as_type(val, type_info, registry)?)),
            TypeInfo::Enum(info) => {
                // Special case: built-in type.
                if let Some(result) = CafBuiltin::try_from_json(val, type_info)? {
                    return Ok(Self::Builtin(result));
                }

                // Special case: Option.
                if let Some(result) = CafEnumVariant::try_from_json_option(val, info, registry)? {
                    // Result is a `CafValue`.
                    return Ok(result);
                }

                // Normal enum.
                Ok(Self::EnumVariant(CafEnumVariant::from_json(val, info, registry)?))
            }
            TypeInfo::Value(_) => match val {
                serde_json::Value::Bool(value) => Ok(Self::Bool(CafBool::from_json_bool(*value, type_info)?)),
                serde_json::Value::Number(value) => Ok(Self::Number(CafNumber::from_json_number(value.clone()))),
                serde_json::Value::String(value) => Ok(Self::String(CafString::from_json_string(value)?)),
                _ => Err(format!(
                        "failed converting {:?} from json {:?} into a value; json is not a bool/number/string so \
                        we don't know how to handle it",
                        type_info.type_path(), val
                    )),
            },
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
}

//-------------------------------------------------------------------------------------------------------------------

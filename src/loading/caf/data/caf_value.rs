
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
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
        match *self {
            Self::EnumVariant(val) => {
                val.write_to(writer)?;
            }
            Self::Builtin(val) => {
                val.write_to(writer)?;
            }
            Self::Array(val) => {
                val.write_to(writer)?;
            }
            Self::Tuple(val) => {
                val.write_to(writer)?;
            }
            Self::Map(val) => {
                val.write_to(writer)?;
            }
            Self::FlattenGroup(val) => {
                val.write_to(writer)?;
            }
            Self::Number(val) => {
                val.write_to(writer)?;
            }
            Self::Bool(val) => {
                val.write_to(writer)?;
            }
            Self::None(val) => {
                val.write_to(writer)?;
            }
            Self::String(val) => {
                val.write_to(writer)?;
            }
            Self::Constant(val) => {
                val.write_to(writer)?;
            }
            Self::DataMacro(val) => {
                val.write_to(writer)?;
            }
            Self::MacroParam(val) => {
                val.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        match *self {
            Self::Enum(val) => {
                val.to_json(writer)
            }
            Self::Builtin(val) => {
                val.to_json(writer)
            }
            Self::Array(val) => {
                val.to_json(writer)
            }
            Self::Tuple(val) => {
                val.to_json(writer)
            }
            Self::Map(val) => {
                val.to_json(writer)
            }
            Self::FlattenGroup(val) => {
                Err(std::io::Error::other(format!("cannot convert flatten group {val:?} to JSON")))
            }
            Self::Number(val) => {
                val.to_json(writer)
            }
            Self::Bool(val) => {
                val.to_json(writer)
            }
            Self::None(val) => {
                val.to_json(writer)
            }
            Self::String(val) => {
                val.to_json(writer)
            }
            Self::Constant(val) => {
                Err(std::io::Error::other(format!("cannot convert constant {val:?} to JSON")))
            }
            Self::DataMacro(val) => {
                Err(std::io::Error::other(format!("cannot convert data macro {val:?} to JSON")))
            }
            Self::MacroParam(val) => {
                Err(std::io::Error::other(format!("cannot convert macro param {val:?} to JSON")))
            }
        }
    }

    pub fn from_json(val: &serde_json::Value, type_info: &TypeInfo, registry: &TypeRegistry) -> Result<Self, String>
    {
        match type_info {
            TypeInfo::Struct(_) => {
                Ok(Self::Map(CafMap::from_json_as_type(val, type_info, registry)?))
            }
            TypeInfo::TupleStruct(_) => {
                Ok(Self::Tuple(CafTuple::from_json_as_type(val, type_info, registry)?))
            }
            TypeInfo::Tuple(_) => {
                Ok(Self::Tuple(CafTuple::from_json_as_type(val, type_info, registry)?))
            }
            TypeInfo::List(_) => {
                Ok(Self::Array(CafArray::from_json(val, type_info, registry)?))
            }
            TypeInfo::Array(_) => {
                Ok(Self::Array(CafArray::from_json(val, type_info, registry)?))
            }
            TypeInfo::Map(_) => {
                Ok(Self::Map(CafMap::from_json_as_type(val, type_info, registry)?))
            }
            TypeInfo::Enum(info) => {
                // Special case: built-in type.
                if let Some(result) = CafBuiltin::try_from_json(val, info)? {
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
            TypeInfo::Value(_) => {
                match val {
                    serde_json::Value::Bool(value) => Ok(Self::Bool(CafBool{ fill: CafFill::default(), value})),
                    serde_json::Value::Number(value) => Ok(Self::Number(CafNumber::from_json_number(value)?)),
                    serde_json::Value::String(value) => Ok(Self::String(CafString::from_json_string(value)?)),
                    _ => Err(format!(
                        "failed converting {:?} from json {:?} into a value; json is not a bool/number/string so \
                        we don't know how to handle it",
                        type_info.type_path(), val
                    ))
                }
            }
        }
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Enum(val), Self::Enum(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::Builtin(val), Self::Builtin(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::Array(val), Self::Array(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::Tuple(val), Self::Tuple(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::Map(val), Self::Map(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::FlattenGroup(val), Self::FlattenGroup(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::Number(val), Self::Number(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::Bool(val), Self::Bool(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::None(val), Self::None(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::String(val), Self::String(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::Constant(val), Self::Constant(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::DataMacro(val), Self::DataMacro(other_val)) => {
                val.recover_fill(other_fill);
            }
            (Self::MacroParam(val), Self::MacroParam(other_val)) => {
                val.recover_fill(other_fill);
            }
            _ => ()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

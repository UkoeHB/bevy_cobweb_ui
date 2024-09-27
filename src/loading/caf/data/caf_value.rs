
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
            TypeInfo::Struct(info) => {

            }
            TypeInfo::TupleStruct(info) => {

            }
            TypeInfo::Tuple(_) => {

            }
            TypeInfo::List(_) => {

            }
            TypeInfo::Array(_) => {

            }
            TypeInfo::Map(_) => {
                Err(format!(
                    "failed converting {:?} from json {:?} as an instruction; type is a map not a struct/enum",
                    val, type_info.type_path()
                ))
            }
            TypeInfo::Enum(info) => {

            }
            TypeInfo::Value(_) => {
                
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

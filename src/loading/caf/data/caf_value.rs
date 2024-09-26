
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
}

//-------------------------------------------------------------------------------------------------------------------

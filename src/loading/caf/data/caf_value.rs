
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafValue
{
    /// Includes instructions (which are 1:1 rust structs) and enums (which are struct-like in caf files).
    Struct(CafStruct),
    /// Special built-in enum variants like `none` and `#FFFFFF` for colors.
    StructBuiltin(CafStructBuiltin),
    Array(CafArray),
    Tuple(CafTuple),
    Map(CafMap),
    FlattenGroup(CafFlattenGroup),
    Number(CafNumber),
    Bool(CafBool),
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
            Self::Struct(val) => {
                val.write_to(writer)?;
            }
            Self::StructBuiltin(val) => {
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
}

//-------------------------------------------------------------------------------------------------------------------


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafGenericValue
{
    Identifier(CafTypeIdentifier),
    MacroParam(CafMacroParam),
    Macro(CafDataMacroCall),
    Constant(CafConstant),
}

impl CafGenericValue
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Identifier(val) => {
                val.write_to(writer)?;
            }
            Self::MacroParam(val) => {
                val.write_to(writer)?;
            }
            Self::Macro(val) => {
                val.write_to(writer)?;
            }
            Self::Constant(val) => {
                val.write_to(writer)?;
            }
        }
        Ok(())
    }
}

/*
Parsing:
- Purely optional macro params are not allowed. Optional with default value are allowed.
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafGenerics
{
    /// Fill before opening <
    pub open_fill: CafFill,
    /// Each of these values is expected to take care of its own fill.
    pub values: Vec<CafGenericValue>,
    /// Fill before closing >
    pub close_fill: CafFill,
}

impl CafGenerics
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.open_fill.write_to(writer)?;
        writer.write('<'.as_bytes())?;
        for generic in self.values.iter() {
            generic.write_to(writer)?;
        }
        self.close_fill.write_to(writer)?;
        writer.write('>'.as_bytes())?;
        Ok(())
    }
}

/*
Parsing: lowercase identifiers, can be a sequence separated by '.' and not ending or starting with '.'
*/

//-------------------------------------------------------------------------------------------------------------------

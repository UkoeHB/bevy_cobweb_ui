
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafTypeIdentifier
{
    pub start_fill: CafFill,
    pub name: Arc<str>,
    pub generics: Option<CafGenerics>
}

impl CafTypeIdentifier
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write(self.as_bytes())?;
        Ok(())
    }
}

impl Default for CafTypeIdentifier
{
    fn default() -> Self
    {
        Self{
            start_fill: CafFill::new(' '),
            name: Arc::from(""),
            generics: None,
        }
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

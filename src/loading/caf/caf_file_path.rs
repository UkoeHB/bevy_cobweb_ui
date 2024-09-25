
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafFilePath(pub Arc<str>);

impl CafFilePath
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write('\"'.as_bytes())?;
        writer.write(self.as_bytes())?;  // TODO: convert special characters to literal representation?
        writer.write('\"'.as_bytes())?;
        Ok(())
    }
}

impl Default for CafFilePath
{
    fn default() -> Self
    {
        Self(Arc::from(""))
    }
}

/*
Parsing:
- should be a valid AssetPath file path without weird special characters or newlines (no multi-line paths)
- last file extension should be .caf
*/

//-------------------------------------------------------------------------------------------------------------------

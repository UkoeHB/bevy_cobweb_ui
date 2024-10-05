use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafSection
{
    Manifest(CafManifest),
    Import(CafImport),
    Using(CafUsing),
    Defs(CafDefs),
    Commands(CafCommands),
    Scenes(CafScenes),
}

impl CafSection
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Manifest(section) => section.write_to(first_section, writer),
            Self::Import(section) => section.write_to(first_section, writer),
            Self::Using(section) => section.write_to(first_section, writer),
            Self::Defs(section) => section.write_to(first_section, writer),
            Self::Commands(section) => section.write_to(first_section, writer),
            Self::Scenes(section) => section.write_to(first_section, writer),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Caf
{
    pub sections: Vec<CafSection>,
    /// Whitespace and comments at the end of the file.
    pub end_fill: CafFill,
    pub metadata: CafMetadata,
}

impl Caf
{
    /// Will automatically insert a newline at the end if one is missing.
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        for (idx) in &self.sections.enumerate() {
            section.write_to(idx == 0, writer)?;
        }
        let ends_newline = self.end_fill.ends_with_newline();
        self.end_fill.write_to(writer)?;
        if !ends_newline {
            writer.write('\n'.as_bytes())?;
        }
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

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
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Manifest(section) => section.write_to(writer),
            Self::Import(section) => section.write_to(writer),
            Self::Using(section) => section.write_to(writer),
            Self::Defs(section) => section.write_to(writer),
            Self::Commands(section) => section.write_to(writer),
            Self::Scenes(section) => section.write_to(writer),
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
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        for section in &self.sections {
            section.write_to(writer)?;
        }
        self.end_fill.write_to(writer)?;
        Ok(())
    }

    /// Insert a newline to the end if there is none.
    ///
    /// This is useful cleanup when writing back to a file. Github, for example, likes a newline at the end.
    pub fn end_with_newline(&mut self)
    {
        if !self.end_fill.ends_with_newline() {
            self.end_fill.segments.push(CafFillSegment::newline());
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

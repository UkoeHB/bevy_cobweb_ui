use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafImportFile
{
    ManifestKey(CafManifestKey),
    File(CafFilePath)
}

impl CafImportFile
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::ManifestKey(key) => {
                key.write_to(writer)?;
            }
            Self::File(file) => {
                file.write_to(writer)?;
            }
        }
        Ok(())
    }
}

impl Default for CafImportFile
{
    fn default() -> Self
    {
        Self::File(CafFilePath::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafImportAlias
{
    None,
    Alias(SmolStr)
}

impl CafImportAlias
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::None => {
                writer.write('_'.as_bytes())?;
            }
            Self::Alias(alias) => {
                writer.write(alias.as_bytes())?;
            }
        }
        Ok(())
    }
}

impl Default for CafImportAlias
{
    fn default() -> Self
    {
        Self::None
    }
}

/*
Parsing:
- None: match '_'
- Alias: identifier with lowercase, underscores, and numbers after the first letter
*/

//-------------------------------------------------------------------------------------------------------------------

/// {file} as {alias}
#[derive(Debug, Clone, PartialEq)]
pub struct CafImportEntry
{
    pub entry_fill: CafFill,
    pub file: CafImportFile,
    pub as_fill: CafFill,
    pub alias_fill: CafFill,
    pub alias: CafImportAlias,
}

impl CafImportEntry
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.entry_fill.write_to_or_else(writer, '\n')?;
        self.file.write_to(writer)?;
        self.as_fill.write_to_or_else(writer, ' ')?;
        writer.write("as".as_bytes())?;
        self.alias_fill.write_to_or_else(writer, ' ')?;
        self.alias.write_to(writer)?;
        Ok(())
    }

    // Makes a new entry with default spacing.
    pub fn new(file: impl AsRef<str>, alias: impl AsRef<str>) -> Self
    {
        Self {
            file: CafImportFile::File(Arc::from(file.as_ref())),
            alias: CafImportAlias(Arc::from(alias.as_ref())),
            ..default()
        }
    }
}

impl Default for CafImportEntry
{
    fn default() -> Self
    {
        Self {
            start_fill: CafFill::new('\n'),
            file: Default::default(),
            as_fill: CafFill::new(' '),
            alias_fill: CafFill::new(' '),
            alias: Default::default(),
        }
    }
}

/*
Parsing:
- Must start with newline.
- Must be 'file as alias'.
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafImport
{
    pub start_fill: CafFill,
    pub entries: Vec<CafImportEntry>,
}

impl CafImport
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write("#import".as_bytes)?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }
}

impl Default for CafImport
{
    fn default() -> Self
    {
        Self { start_fill: CafFill::default(), entries: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

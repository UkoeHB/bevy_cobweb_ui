use std::sync::Arc;

use bevy::prelude::default;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafManifestEntry
{
    pub start_fill: CafFill,
    pub file: SceneFile,
    pub pre_as_fill: CafFill,
    pub post_as_fill: CafFill,
    pub key: Arc<str>,
}

impl CafManifestEntry
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write(self.file.as_str().as_bytes())?;
        self.pre_as_fill.write_to(writer)?;
        self.post_as_fill.write_to(writer)?;
        writer.write(self.key.as_bytes())?;
        Ok(())
    }

    // Makes a new entry with default spacing.
    pub fn new(file: impl AsRef<str>, key: impl AsRef<str>) -> Self
    {
        Self {
            file: SceneFile::new(file),
            key: Arc::from(key.as_ref()),
            ..default()
        }
    }

    pub fn eq_ignore_whitespace(&self, other: &CafManifestEntry) -> bool
    {
        self.file == other.file && self.key == other.key
    }
}

impl Default for CafManifestEntry
{
    fn default() -> Self
    {
        Self {
            start_fill: CafFill::newline(),
            file: Default::default(),
            pre_as_fill: CafFill::space(),
            post_as_fill: CafFill::space(),
            key: Arc::from(""),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafManifest
{
    pub start_fill: CafFill,
    pub entries: Vec<CafManifestEntry>,
}

impl CafManifest
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    pub fn eq_ignore_whitespace(&self, other: &CafManifest) -> bool
    {
        if self.entries.len() != other.entries.len() {
            return false;
        }
        !self
            .entries
            .iter()
            .zip(other.entries.iter())
            .any(|(a, b)| !a.eq_ignore_whitespace(b))
    }
}

impl Default for CafManifest
{
    fn default() -> Self
    {
        Self { start_fill: CafFill::default(), entries: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

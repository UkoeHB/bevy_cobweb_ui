//-------------------------------------------------------------------------------------------------------------------
use std::sync::Arc;

use bevy::prelude::Deref;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafFilePath(pub Arc<str>);

impl CafFilePath
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("\"".as_bytes())?;
        writer.write_bytes(self.as_bytes())?;
        writer.write_bytes("\"".as_bytes())?;
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
- should be a valid AssetPath file path without weird special characters
- last file extension should be .caf
*/

//-------------------------------------------------------------------------------------------------------------------

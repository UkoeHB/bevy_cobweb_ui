use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when a new file is added to the editor.
#[derive(Debug, Clone)]
pub struct EditorNewFile
{
    pub file: CobFile,
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when a file's data saved in the editor is changed by extenal factors.
#[derive(Debug, Clone)]
pub struct EditorFileExternalChange
{
    pub file: CobFile,
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when the save status of a file changes in the editor.
#[derive(Debug, Clone)]
pub struct EditorSaveStatus
{
    pub file: CobFile,
    /// If `true` then the file has modifications that are unsaved.
    pub unsaved: bool,
}

//-------------------------------------------------------------------------------------------------------------------

use super::*;
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

/// Reactive event broadcasted when a file acquires 'unsaved' status in the editor.
#[derive(Debug, Clone)]
pub struct EditorFileUnsaved
{
    pub file: CobFile,
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when an unsaved file acquires 'saved' status in the editor.
#[derive(Debug, Clone)]
pub struct EditorFileSaved
{
    pub file: CobFile,
    /// The hash of the file after saving.
    // TODO: make this public?
    pub(super) hash: CobFileHash,
}

//-------------------------------------------------------------------------------------------------------------------

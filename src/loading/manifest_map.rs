use std::collections::HashMap;
use std::sync::Arc;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Stores a map between manifest aliases and canonical file names.
#[derive(Default, Debug)]
pub(crate) struct ManifestMap
{
    map: HashMap<Arc<str>, SceneFile>,
}

impl ManifestMap
{
    pub(crate) fn insert(&mut self, key: Arc<str>, file: SceneFile) -> Option<SceneFile>
    {
        self.map.insert(key, file)
    }

    pub(crate) fn remove(&mut self, key: &Arc<str>) -> Option<SceneFile>
    {
        self.map.remove(key)
    }

    /// Gets a file reference from a scene file reference.
    ///
    /// Returns `None` if the requested file is [`SceneFile::ManifestKey`] and lookup failed.
    pub(crate) fn get(&self, file: &SceneFile) -> Option<SceneFile>
    {
        let SceneFile::ManifestKey(key) = file else { return Some(file.clone()) };
        self.map.get(key).cloned()
    }

    /// Swaps a manifest key for a file reference.
    ///
    /// Logs an error if looking up a manifest key fails.
    pub(crate) fn swap_for_file(&self, maybe_key: &mut SceneFile)
    {
        let SceneFile::ManifestKey(key) = maybe_key else { return };
        let Some(file_ref) = self.map.get(key) else {
            tracing::error!("tried accessing manifest key {:?} but no file was found", key);
            return;
        };
        *maybe_key = file_ref.clone();
    }
}

//-------------------------------------------------------------------------------------------------------------------

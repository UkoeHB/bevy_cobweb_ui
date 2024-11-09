use std::collections::HashMap;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Stores a map between manifest aliases and canonical file names.
#[derive(Default, Debug)]
pub(crate) struct ManifestMap
{
    map: HashMap<ManifestKey, CobFile>,
}

impl ManifestMap
{
    pub(crate) fn insert(&mut self, key: ManifestKey, file: CobFile) -> Option<CobFile>
    {
        self.map.insert(key, file)
    }

    pub(crate) fn remove(&mut self, key: &ManifestKey) -> Option<CobFile>
    {
        self.map.remove(key)
    }

    /// Gets a file reference from a scene file reference.
    ///
    /// Returns `None` if the requested file is [`SceneFile::ManifestKey`] and lookup failed.
    pub(crate) fn get(&self, key: &ManifestKey) -> Option<CobFile>
    {
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
        *maybe_key = SceneFile::File(file_ref.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

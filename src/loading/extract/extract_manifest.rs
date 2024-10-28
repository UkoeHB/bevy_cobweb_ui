use std::collections::HashMap;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_manifest_section(
    file: &CafFile,
    section: &CafManifest,
    manifests: &mut HashMap<CafFile, ManifestKey>,
)
{
    for entry in section.entries.iter() {
        let entry_file = match &entry.file {
            CafManifestFile::SelfRef => file.clone(),
            CafManifestFile::File(entry_file) => entry_file.clone(),
        };

        manifests.insert(entry_file, entry.key.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

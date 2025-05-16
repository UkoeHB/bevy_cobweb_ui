use crate::cob::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_manifest_section(
    file: &CobFile,
    section: &CobManifest,
    manifests: &mut Vec<(CobFile, ManifestKey)>,
)
{
    for entry in section.entries.iter() {
        let entry_file = match &entry.file {
            CobManifestFile::SelfRef => file.clone(),
            CobManifestFile::File(entry_file) => entry_file.clone(),
        };

        if manifests
            .iter()
            .any(|(other_file, _)| entry_file == *other_file)
        {
            tracing::warn!("ignoring duplicate file {:?} in manifest of {:?}",
                entry_file, file);
            continue;
        }

        manifests.push((entry_file, entry.key.clone()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

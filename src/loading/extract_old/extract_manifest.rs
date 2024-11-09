use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Map, Value};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn extract_manifest_section(
    file: &SceneFile,
    map: &Map<String, Value>,
    manifests: &mut HashMap<String, Arc<str>>,
)
{
    let Some(manifest_section) = map.get(MANIFEST_KEYWORD) else {
        return;
    };

    let Value::Object(manifest_section) = manifest_section else {
        tracing::error!("failed parsing 'manifest' section in {:?}, it is not an Object", file);
        return;
    };

    for (manifest_file, manifest_key) in manifest_section.iter() {
        let Value::String(manifest_key) = manifest_key else {
            tracing::error!("failed parsing manifest key {:?} for {:?} in 'manifest' section of {:?}, it is not a \
                String", manifest_key.as_str(), manifest_file.as_str(), file.as_str());
            continue;
        };
        let manifest_key = Arc::from(manifest_key.as_str());

        match manifest_file.as_str() {
            // Empty file name means use the file where the manifest section was found.
            "" => {
                tracing::trace!("adding manifest {file:?} {manifest_key:?}");
                let prev = manifests.insert(String::from(file.as_str()), manifest_key);
                if let Some(prev) = prev {
                    tracing::error!("found duplicate file name {:?} in manifest of file {:?}, ignoring manifest key {:?}",
                        manifest_file.as_str(), file.as_str(), prev);
                }
            }
            _ => {
                if !SceneFile::str_is_file_path(manifest_file) {
                    tracing::error!("ignoring manifest entry in {:?} with invalid file path {:?} (key: {:?}); \
                        cobweb asset files should terminate with `.cob.json`",
                        file.as_str(), manifest_file.as_str(), manifest_key);
                    continue;
                }
                tracing::trace!("adding manifest {manifest_file:?} {manifest_key:?}");
                manifests.insert(manifest_file.clone(), manifest_key);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

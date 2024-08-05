use serde_json::{Map, Value};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn extract_import_section(
    file: &SceneFile,
    map: &Map<String, Value>,
    imports: &mut Vec<(String, SmolStr)>,
)
{
    let Some(import_section) = map.get(IMPORT_KEYWORD) else {
        return;
    };

    let Value::Object(import_section) = import_section else {
        tracing::error!("failed parsing 'import' section in {:?}, it is not an Object", file);
        return;
    };

    for (import, alias) in import_section.iter() {
        let Value::String(alias) = alias else {
            tracing::error!("failed parsing import alias {:?} for {:?} in 'import' section of {:?}, it is not a \
                String", alias, import, file);
            continue;
        };

        if !SceneFile::str_is_file_path(import) {
            tracing::error!("ignoring import entry in {:?} that does not have a valid file path {:?}",
                file, import.as_str());
            continue;
        }
        imports.push((import.clone(), alias.as_str().into()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

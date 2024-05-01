use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn extract_import_section(
    file: &LoadableFile,
    map: &Map<String, Value>,
    imports: &mut HashMap<LoadableFile, String>,
)
{
    let Some(import_section) = map.get(&String::from(IMPORT_KEYWORD)) else {
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

        imports.insert(LoadableFile::new(import.as_str()), alias.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

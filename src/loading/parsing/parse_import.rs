use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn extract_import_section(
    file: &LoadableFile,
    map: &Map<String, Value>,
    imports: &mut HashSet<LoadableFile>,
)
{
    let Some(import_section) = map.get(&String::from(IMPORT_KEYWORD)) else {
        return;
    };

    let Value::Array(import_section) = import_section else {
        tracing::error!("failed parsing 'import' section in {:?}, it is not an Array", file);
        return;
    };

    for import in import_section.iter() {
        let Value::String(import) = import else {
            tracing::error!("failed parsing import {:?} in 'import' section of {:?}, it is not a String", import, file);
            continue;
        };

        imports.insert(LoadableFile::new(import.as_str()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

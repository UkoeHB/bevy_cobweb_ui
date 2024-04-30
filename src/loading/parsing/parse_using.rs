use std::collections::HashMap;

use bevy::reflect::TypeRegistry;
use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn extract_using_section(
    type_registry: &TypeRegistry,
    file: &LoadableFile,
    map: &Map<String, Value>,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    let Some(using_section) = map.get(&String::from(USING_KEYWORD)) else {
        return;
    };

    let Value::Array(longnames) = using_section else {
        tracing::error!("failed parsing 'using' section in {:?}, it is not an Array", file);
        return;
    };

    for longname in longnames.iter() {
        let Value::String(longname) = longname else {
            tracing::error!("failed parsing longname {:?} in 'using' section of {:?}, it is not a String",
                longname, file);
            continue;
        };

        let Some(registration) = type_registry.get_with_type_path(longname.as_str()) else {
            tracing::error!("longname {:?} in 'using' section of {:?} not found in type registry",
                longname, file);
            continue;
        };
        let short_name = registration.type_info().type_path_table().short_path();
        let long_name = registration.type_info().type_path_table().path(); //get static version

        name_shortcuts.insert(short_name, long_name);
    }
}

//-------------------------------------------------------------------------------------------------------------------

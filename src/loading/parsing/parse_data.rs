use std::collections::HashMap;

use bevy::reflect::TypeRegistry;
use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_loadable_entry(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: &LoadableFile,
    current_path: &LoadablePath,
    short_name: &str,
    value: Value,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    // Get the loadable's longname.
    let Some((_short_name, long_name, type_id, deserializer)) =
        get_loadable_meta(type_registry, file, current_path, short_name, name_shortcuts)
    else {
        return;
    };

    // Get the loadable's value.
    let loadable_value = get_loadable_value(deserializer, value);

    // Save this loadable.
    loadablesheet.insert_loadable(
        &LoadableRef { file: file.clone(), path: current_path.clone() },
        loadable_value,
        type_id,
        long_name,
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_branch_entry(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: &LoadableFile,
    current_path: &LoadablePath,
    key: &str,
    value: Value,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    let Value::Object(data) = value else {
        tracing::error!("failed parsing extension {:?} at {:?} in {:?}, extension is not an Object",
            key, current_path, file);
        return;
    };

    let extended_path = current_path.extend(key);
    parse_branch(type_registry, loadablesheet, file, &extended_path, data, name_shortcuts);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn parse_branch(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: &LoadableFile,
    current_path: &LoadablePath,
    mut data: Map<String, Value>,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    for (key, value) in data.iter_mut() {
        // Skip keyword map entries.
        if key_is_keyword(key) {
            continue;
        }

        let value = value.take();

        if is_loadable_entry(key) {
            handle_loadable_entry(
                type_registry,
                loadablesheet,
                file,
                current_path,
                key.as_str(),
                value,
                name_shortcuts,
            );
        } else {
            handle_branch_entry(
                type_registry,
                loadablesheet,
                file,
                current_path,
                key.as_str(),
                value,
                name_shortcuts,
            );
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

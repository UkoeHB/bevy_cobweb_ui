use std::collections::HashMap;

use bevy::reflect::TypeRegistry;
use serde_json::{Map, Value};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_commands_entry(
    type_registry: &TypeRegistry,
    caf_cache: &mut CobwebAssetCache,
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
    let command_value = get_loadable_value(deserializer, value);

    // Save this command.
    caf_cache.insert_command(
        &LoadableRef { file: file.clone(), path: current_path.clone() },
        command_value,
        type_id,
        long_name,
    );
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn parse_commands_section(
    type_registry: &TypeRegistry,
    caf_cache: &mut CobwebAssetCache,
    file: &LoadableFile,
    data: &mut Map<String, Value>,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    let Some(commands_section) = data.get_mut(COMMANDS_KEYWORD) else {
        return;
    };

    let Value::Object(commands_section) = commands_section else {
        tracing::error!("failed parsing 'commands' section in {:?}, it is not an Object", file);
        return;
    };

    let pseudo_path = LoadablePath::new(COMMANDS_KEYWORD);

    for (key, value) in commands_section.iter_mut() {
        let value = value.take();

        if is_loadable_entry(key) {
            handle_commands_entry(
                type_registry,
                caf_cache,
                file,
                &pseudo_path,
                key.as_str(),
                value,
                name_shortcuts,
            );
        } else {
            tracing::error!("skipping #commands entry in {:?} with invalid key {:?}", file, key);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

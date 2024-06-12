use std::collections::HashMap;
use std::sync::Arc;

use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::TypeRegistry;
use serde::de::DeserializeSeed;
use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn get_inherited_loadable_value(
    file: &LoadableFile,
    current_path: &LoadablePath,
    short_name: &str,
    loadable_entry: &[ReflectedLoadable],
) -> Option<ReflectedLoadable>
{
    // Try to inherit the last loadable entry in the stack.
    let Some(inherited) = loadable_entry.last() else {
        tracing::error!("failed inheriting {:?} at {:?} in {:?}, no inheritable value found",
            short_name, current_path, file);
        return None;
    };

    Some(inherited.clone())
}

//-------------------------------------------------------------------------------------------------------------------

fn get_loadable_value(
    file: &LoadableFile,
    current_path: &LoadablePath,
    short_name: &str,
    value: Value,
    loadable_entry: &[ReflectedLoadable],
    deserializer: TypedReflectDeserializer,
) -> Option<ReflectedLoadable>
{
    match &value {
        Value::String(stringvalue) if (stringvalue.as_str() == INHERITED_KEYWORD) => {
            get_inherited_loadable_value(file, current_path, short_name, loadable_entry)
        }
        _ => match deserializer.deserialize(value) {
            Ok(value) => Some(ReflectedLoadable::Value(Arc::new(value))),
            Err(err) => Some(ReflectedLoadable::DeserializationFailed(Arc::new(err))),
        },
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_loadable_entry(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: &LoadableFile,
    current_path: &LoadablePath,
    short_name: &str,
    value: Value,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
    loadable_stack: &mut HashMap<&'static str, Vec<ReflectedLoadable>>,
    stack_tracker: &mut Vec<(&'static str, usize)>,
)
{
    // Get the loadable's longname.
    let Some((short_name, long_name, type_id, deserializer)) =
        get_loadable_meta(type_registry, file, current_path, short_name, name_shortcuts)
    else {
        return;
    };

    // Get the loadable's value.
    let loadable_entry = loadable_stack.entry(short_name).or_default();
    let starting_len = loadable_entry.len();

    let Some(loadable_value) =
        get_loadable_value(file, current_path, short_name, value, loadable_entry, deserializer)
    else {
        return;
    };

    // Save this loadable.
    loadable_entry.push(loadable_value.clone());
    stack_tracker.push((short_name, starting_len));

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
    loadable_stack: &mut HashMap<&'static str, Vec<ReflectedLoadable>>,
    stack_trackers: &mut Vec<Vec<(&'static str, usize)>>,
)
{
    let Value::Object(data) = value else {
        tracing::error!("failed parsing extension {:?} at {:?} in {:?}, extension is not an Object",
            key, current_path, file);
        return;
    };

    let extended_path = current_path.extend(key);
    parse_branch(
        type_registry,
        loadablesheet,
        file,
        &extended_path,
        data,
        name_shortcuts,
        loadable_stack,
        stack_trackers,
    );
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn parse_branch(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: &LoadableFile,
    current_path: &LoadablePath,
    mut data: Map<String, Value>,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
    loadable_stack: &mut HashMap<&'static str, Vec<ReflectedLoadable>>,
    stack_trackers: &mut Vec<Vec<(&'static str, usize)>>,
)
{
    let mut stack_tracker = stack_trackers.pop().unwrap_or_default();

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
                loadable_stack,
                &mut stack_tracker,
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
                loadable_stack,
                stack_trackers,
            );
        }
    }

    // Clear loadables tracked for inheritance.
    for (shortname, initial_size) in stack_tracker.drain(..) {
        loadable_stack
            .get_mut(&shortname)
            .unwrap()
            .truncate(initial_size);
    }
    stack_trackers.push(stack_tracker);
}

//-------------------------------------------------------------------------------------------------------------------

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::TypeRegistry;
use serde::de::DeserializeSeed;
use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

const IMPORT_KEYWORD: &str = "#import";
const USING_KEYWORD: &str = "#using";
const CONSTANTS_KEYWORD: &str = "#constants";
const COMMENT_KEYWORD: &str = "#c:";

//-------------------------------------------------------------------------------------------------------------------

fn is_keyword(key: &str) -> bool
{
    key == IMPORT_KEYWORD || key == USING_KEYWORD || key == CONSTANTS_KEYWORD || key.starts_with(COMMENT_KEYWORD)
}

//-------------------------------------------------------------------------------------------------------------------

fn is_loadable_entry(key: &str) -> bool
{
    // Check if camelcase
    let Some(first_char) = key.chars().next() else {
        return false;
    };
    first_char.is_uppercase()
}

//-------------------------------------------------------------------------------------------------------------------

fn get_loadable_meta<'a>(
    type_registry: &'a TypeRegistry,
    file: &LoadableFile,
    current_path: &LoadablePath,
    short_name: &str,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
) -> Option<(&'static str, &'static str, TypeId, TypedReflectDeserializer<'a>)>
{
    // Check if we already have this mapping.
    let mut found_mapping = false;
    let registration = match name_shortcuts.get(short_name) {
        Some(long_name) => {
            found_mapping = true;
            type_registry.get_with_type_path(long_name)
        }
        None => type_registry.get_with_short_type_path(short_name),
    };

    // Look up the longname
    let Some(registration) = registration else {
        tracing::error!("failed getting long type name for {:?} at {:?} in {:?}; if the type is ambiguous because \
            there are multiple types with this short name, add its long name to the loadablesheet file's 'using' section",
            short_name, current_path, file);
        return None;
    };

    let short_name = registration.type_info().type_path_table().short_path(); //get static version
    let long_name = registration.type_info().type_path_table().path();

    // Save this mapping for later.
    if !found_mapping {
        name_shortcuts.insert(short_name, long_name);
    }

    // Deserializer
    let deserializer = TypedReflectDeserializer::new(registration, type_registry);

    Some((short_name, long_name, registration.type_info().type_id(), deserializer))
}

//-------------------------------------------------------------------------------------------------------------------

fn get_inherited_loadable_value(
    file: &LoadableFile,
    current_path: &LoadablePath,
    short_name: &str,
    loadable_entry: &Vec<ReflectedLoadable>,
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
    loadable_entry: &Vec<ReflectedLoadable>,
    deserializer: TypedReflectDeserializer,
) -> Option<ReflectedLoadable>
{
    match &value {
        Value::String(stringvalue) if (stringvalue.as_str() == "inherited") => {
            get_inherited_loadable_value(file, current_path, short_name, loadable_entry)
        }
        _ => match deserializer.deserialize(value) {
            Ok(value) => Some(ReflectedLoadable::Value(Arc::new(value))),
            Err(err) => Some(ReflectedLoadable::DeserializationFailed(Arc::new(err))),
        },
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn try_handle_using_entry(
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
    let loadable_entry = loadable_stack.entry(short_name).or_insert_with(|| Vec::default());
    let starting_len = loadable_entry.len();

    let Some(loadable_value) =
        get_loadable_value(file, current_path, short_name, value, &loadable_entry, deserializer)
    else {
        return;
    };

    // Save this loadable.
    loadable_entry.push(loadable_value.clone());
    stack_tracker.push((short_name, starting_len));

    loadablesheet.insert(
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
        &file,
        &extended_path,
        data,
        name_shortcuts,
        loadable_stack,
        stack_trackers,
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn parse_branch(
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
        if is_keyword(key) {
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
        loadable_stack.get_mut(&shortname).unwrap().truncate(initial_size);
    }
    stack_trackers.push(stack_tracker);
}

//-------------------------------------------------------------------------------------------------------------------

/// Consumes a loadablesheet file's data and loads it into [`LoadableSheet`].
pub(crate) fn parse_loadablesheet_file(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: LoadableFile,
    data: Value,
) -> bool
{
    tracing::info!("parsing loadablesheet {:?}", file.file);
    loadablesheet.initialize_file(file.clone());

    let Value::Object(data) = data else {
        tracing::error!("failed parsing loadablesheet {:?}, data base layer is not an Object", file);
        return false;
    };

    // [ shortname : longname ]
    let mut name_shortcuts: HashMap<&'static str, &'static str> = HashMap::default();
    // [ shortname : [ loadable value ] ]
    let mut loadable_stack: HashMap<&'static str, Vec<ReflectedLoadable>> = HashMap::default();
    // [ {shortname, top index into loadablestack when first stack added this frame} ]
    let mut stack_trackers: Vec<Vec<(&'static str, usize)>> = Vec::default();

    // TODO: handle imports

    // - get imports section
    // - check if imports available, else cache for parsing later
    // - copy saved using and constants from imported

    // Extract using section.
    try_handle_using_entry(type_registry, &file, &data, &mut name_shortcuts);

    // Extract constants section.
    // - build constants map and check for $$ constant references

    // TODO: save using and constants in case this file is imported by another file

    // Search and replace constants.

    // Recursively consume the file contents.
    parse_branch(
        type_registry,
        loadablesheet,
        &file,
        &LoadablePath::new(""),
        data,
        &mut name_shortcuts,
        &mut loadable_stack,
        &mut stack_trackers,
    );

    // On return true, load any files that depend on it.
    // TODO: use a cfg_if on the file_watcher feature to decide whether to discard all file contents once all
    // registerd sheets are done loading
    true
}

//-------------------------------------------------------------------------------------------------------------------

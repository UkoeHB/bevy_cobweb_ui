use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn path_to_string(path: &[String]) -> String
{
    path.iter()
        .fold(String::default(), |prev, val| append_constant_extension(prev, val))
}

//-------------------------------------------------------------------------------------------------------------------

fn get_constants_set<'a>(
    file: &LoadableFile,
    prefix: &'static str,
    value_str: &'a String,
    constants: &'a HashMap<String, Map<String, Value>>,
) -> Option<(&'a str, &'a Map<String, Value>)>
{
    let Some(("", path_ref)) = value_str.split_once(prefix) else { return None };

    if path_ref.is_empty() {
        tracing::warn!("ignoring zero-length constant reference {:?} in {:?}", value_str, file);
        return None;
    }

    // Extract path terminator.
    let mut rev_iterator = path_ref.rsplitn(2, "::");
    let terminator = rev_iterator.next().unwrap();
    let path = rev_iterator.next().unwrap_or("");

    let Some(constant_value) = constants.get(&String::from(path)) else {
        tracing::warn!("ignoring unknown constant reference {:?} in constants \
            section of {:?}", value_str, file);
        return None;
    };

    Some((terminator, constant_value))
}

//-------------------------------------------------------------------------------------------------------------------

fn try_replace_string_with_constant(
    file: &LoadableFile,
    prefix: &'static str,
    value: &mut Value,
    constants: &HashMap<String, Map<String, Value>>,
)
{
    let Value::String(value_str) = &value else { return };
    let Some((terminator, constants_set)) = get_constants_set(file, prefix, value_str, constants) else {
        return;
    };

    // For map values, paste the data pointed-to by the terminator.
    let Some(constant_data) = constants_set.get(&String::from(terminator)) else {
        tracing::warn!("ignoring constant reference {:?} with no recorded data in {:?}", value_str, file);
        return;
    };

    *value = constant_data.clone();
}

//-------------------------------------------------------------------------------------------------------------------

fn try_replace_map_key_with_constant(
    file: &LoadableFile,
    prefix: &'static str,
    key: String,
    map: &mut Map<String, Value>,
    constants: &HashMap<String, Map<String, Value>>,
)
{
    let Some((terminator, constants_set)) = get_constants_set(file, prefix, &key, constants) else {
        return;
    };

    //TODO: Ordering is NOT preserved when replacing a map key with constants.
    map.remove(&key);

    match terminator {
        // If 'paste all' terminator, then insert all contents of the section into the map.
        "*" => {
            let mut constants_set = constants_set.clone();
            map.append(&mut constants_set);
        }
        // Otherwise, just paste the terminator and its value.
        _ => {
            // For map values, paste the data pointed-to by the terminator.
            let Some(constant_data) = constants_set.get(&String::from(terminator)) else {
                tracing::warn!("ignoring constant reference {:?} with no recorded data in {:?}", key, file);
                return;
            };

            map.insert(terminator.into(), constant_data.clone());
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) const CONSTANT_SEPARATOR: &str = "::";

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn append_constant_extension(mut path: String, ext: &str) -> String
{
    path.reserve(CONSTANT_SEPARATOR.len() + ext.len());
    if !path.is_empty() && !ext.is_empty() {
        path.push_str(CONSTANT_SEPARATOR);
    }
    path.push_str(ext);
    path
}

//-------------------------------------------------------------------------------------------------------------------

/// Replaces constants throughout a map, ignoring sections that start with keywords.
pub(crate) fn search_and_replace_map_constants(
    file: &LoadableFile,
    prefix: &'static str,
    map: &mut Map<String, Value>,
    constants: &HashMap<String, Map<String, Value>>,
)
{
    for key in map
        .keys()
        .filter(|k| !key_is_non_content_keyword(k))
        .cloned()
        .collect::<Vec<String>>()
        .drain(..)
    {
        try_replace_map_key_with_constant(file, prefix, key, map, constants);
    }

    for (key, value) in map.iter_mut() {
        // Ignore sections that start with a non-content keyword.
        if key_is_non_content_keyword(key) {
            continue;
        }
        search_and_replace_constants(file, prefix, value, constants);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Replaces constants throughout a value.
pub(crate) fn search_and_replace_constants(
    file: &LoadableFile,
    prefix: &'static str,
    value: &mut Value,
    constants: &HashMap<String, Map<String, Value>>,
)
{
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => (),
        Value::String(_) => {
            try_replace_string_with_constant(file, prefix, value, constants);
        }
        Value::Array(vec) => {
            for value in vec.iter_mut() {
                search_and_replace_constants(file, prefix, value, constants);
            }
        }
        Value::Object(map) => {
            search_and_replace_map_constants(file, prefix, map, constants);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn constants_builder_recurse_into_value(
    file: &LoadableFile,
    key: &String,
    value: &mut Value,
    path: &mut Vec<String>,
    constants: &mut HashMap<String, Map<String, Value>>,
)
{
    // Update the value if it references a constant.
    // - We do this in a separate step in case expanding the constant introduces more constants/path segments.
    try_replace_string_with_constant(file, "$$", value, constants);

    // Parse constants from the value.
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => (),
        Value::Array(vec) => {
            for value in vec.iter_mut() {
                search_and_replace_constants(file, "$$", value, constants);
            }
        }
        //todo: it's ugly
        Value::Object(map) => {
            // Add path stack.
            path.push(key.clone());

            let mut is_normal_segment = false;
            let mut is_constants_segment = false;

            for (key, value) in map.iter_mut() {
                if let Some(("", key)) = key.split_once('$') {
                    if is_normal_segment {
                        tracing::error!("ignoring constant section at {:?} in {:?}, constant path mixed up with value map",
                            path, file);
                        continue;
                    }
                    is_constants_segment = true;

                    // This entry in the data map adds to the constants map.
                    constants_builder_recurse_into_value(file, &key.into(), value, path, constants);
                } else {
                    if is_constants_segment {
                        tracing::error!("ignoring value section at {:?} in {:?}, constant path mixed up with value map",
                            path, file);
                        continue;
                    }
                    is_normal_segment = true;

                    // This key is a normal map entry, so its value is a normal value.
                    search_and_replace_constants(file, "$$", value, constants);
                }
            }

            // End this path stack.
            path.pop();

            if is_constants_segment {
                return;
            }
        }
    }

    // If value was not a map of constant path segments, then the value is a *value* and can be saved.
    let insert = |inner: &mut Map<String, Value>| {
        let prev = inner.insert(key.into(), value.clone());
        if prev.is_some() {
            tracing::warn!("overwriting duplicate terminal path segment {:?} in constants map at {:?}", key, file);
        }
    };
    let base_path = path_to_string(path);

    if let Some(inner) = constants.get_mut(&base_path) {
        insert(inner);
    } else {
        let mut inner = Map::default();
        insert(&mut inner);
        constants.insert(base_path.clone(), inner);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Pulls constants from a constants section.
pub(crate) fn extract_constants_section(
    file: &LoadableFile,
    data: &mut Map<String, Value>,
    constants: &mut HashMap<String, Map<String, Value>>,
)
{
    let Some(constants_section) = data.get_mut(&String::from(CONSTANTS_KEYWORD)) else {
        return;
    };

    let Value::Object(ref mut data) = constants_section else {
        tracing::error!("failed parsing constants in {:?}, section is not an Object", file);
        return;
    };

    let mut path: Vec<String> = Vec::default();

    // Replace map keys with constants.
    for key in data
        .keys()
        .filter(|k| !key_is_non_content_keyword(k))
        .cloned()
        .collect::<Vec<String>>()
        .drain(..)
    {
        try_replace_map_key_with_constant(file, "$$", key, data, constants);
    }

    // Iterate into the map to replace values.
    for (key, value) in data.iter_mut() {
        // Check if value.
        let Some(("", key)) = key.split_once('$') else {
            // Don't warn for comments.
            if !key.starts_with(COMMENT_KEYWORD) {
                tracing::warn!("ignoring non-path in base level of constants section in {:?}", file);
            }
            continue;
        };

        constants_builder_recurse_into_value(file, &key.into(), value, &mut path, constants);
    }
}

//-------------------------------------------------------------------------------------------------------------------

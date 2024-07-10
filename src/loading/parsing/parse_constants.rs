use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Map, Value};
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn get_constants_set<'a>(
    file: &LoadableFile,
    prefix: &'static str,
    value_str: &'a str,
    constants: &'a HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
) -> Option<(&'a str, &'a HashMap<SmolStr, Arc<Value>>)>
{
    let Some(("", path_ref)) = value_str.split_once(prefix) else { return None };

    if path_ref.is_empty() {
        tracing::warn!("ignoring zero-length constant reference {} in {:?}", value_str, file);
        return None;
    }

    // Extract path terminator.
    let mut rev_iterator = path_ref.rsplitn(2, CONSTANT_SEPARATOR);
    let terminator = rev_iterator.next().unwrap();
    let path = rev_iterator.next().unwrap_or("");

    let Some(constant_value) = constants.get(path) else {
        tracing::warn!("ignoring unknown constant reference {:?} in constants \
            section of {:?}; {path:?} {terminator:?}", value_str, file);
        return None;
    };

    Some((terminator, constant_value))
}

//-------------------------------------------------------------------------------------------------------------------

fn try_replace_string_with_constant(
    file: &LoadableFile,
    prefix: &'static str,
    value: &mut Value,
    constants: &HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
)
{
    let Value::String(value_str) = &value else { return };
    let Some((terminator, constants_set)) = get_constants_set(file, prefix, value_str.as_str(), constants) else {
        return;
    };

    // For map values, paste the data pointed-to by the terminator.
    let Some(constant_data) = constants_set.get(terminator) else {
        tracing::warn!("ignoring constant reference {:?} with no recorded data in {:?}", value_str, file);
        return;
    };

    *value = (**constant_data).clone();
}

//-------------------------------------------------------------------------------------------------------------------

fn try_replace_map_key_with_constant(
    file: &LoadableFile,
    prefix: &'static str,
    key: &str,
    map: &mut Map<String, Value>,
    constants: &HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
)
{
    let Some(("", path_ref)) = key.split_once(prefix) else { return };

    if path_ref.is_empty() {
        tracing::warn!("ignoring zero-length constant reference {} in {:?}", key, file);
        return;
    }

    // Extract path terminator.
    let mut rev_iterator = path_ref.rsplitn(2, CONSTANT_SEPARATOR);
    let terminator = rev_iterator.next().unwrap();

    match terminator {
        CONSTANT_PASTE_ALL_TERMINATOR => {
            let real_terminator = rev_iterator.next().unwrap_or("");
            let path = rev_iterator.next().unwrap_or("");

            let Some(constants_set) = constants.get(path) else {
                tracing::warn!("ignoring unknown constant reference {:?} in {:?}", key, file);
                return;
            };

            let Value::Object(constants_value) = constants_set
                .get(real_terminator)
                .map(|v| &**v)
                .unwrap_or(&Value::Null)
            else {
                tracing::warn!("ignoring invalid paste-all constant reference {:?} in {:?}; \
                    the constant's value should be a map", key, file);
                return;
            };

            //TODO: Ordering is NOT preserved when replacing a map key with constants.
            map.remove(key);

            // If 'paste all' terminator, then insert all contents of the constant's value into the map.
            let mut constants_value = constants_value.clone();
            map.append(&mut constants_value);
        }
        _ => {
            tracing::warn!("ignoring map key constant {:?} in {:?}; currently only map key constants with the \
                paste-all terminator are supported (e.g. \"{:?}{}{}\")",
                key, file, key, CONSTANT_SEPARATOR, CONSTANT_PASTE_ALL_TERMINATOR);
            map.remove(key);
            //TODO: consider adding more features for map key constant references
            /*
                let path = rev_iterator.next().unwrap_or("");

                let Some(constants_set) = constants.get(path) else {
                    tracing::warn!("ignoring unknown constant reference {:?} in constants \
                        section of {:?}", key, file);
                    return;
                };

                //TODO: Ordering is NOT preserved when replacing a map key with constants.
                map.remove(key);

                // For map values, paste the data pointed-to by the terminator.
                let Some(constant_data) = constants_set.get(terminator) else {
                    tracing::warn!("ignoring constant reference {} with no recorded data in {:?}", key, file);
                    return;
                };

                map.insert(terminator.into(), constant_data.clone());
            */
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn path_to_constant_string<T: AsRef<str>>(path: &[T]) -> SmolStr
{
    // trim empties then concatenate a::b::c
    let mut count = 0;
    SmolStr::from_iter(
        path.iter()
            .filter(|p| !p.as_ref().is_empty())
            .flat_map(|p| {
                count += 1;
                match count {
                    1 => ["", p.as_ref()],
                    _ => [CONSTANT_SEPARATOR, p.as_ref()],
                }
            }),
    )
}

//-------------------------------------------------------------------------------------------------------------------

/// Replaces constants throughout a map, ignoring sections that start with keywords.
pub(crate) fn search_and_replace_map_constants(
    file: &LoadableFile,
    prefix: &'static str,
    map: &mut Map<String, Value>,
    constants: &HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
)
{
    for key in map
        .keys()
        .filter(|k| k.starts_with(prefix))
        .map(|k| SmolStr::from(k))
        .collect::<Vec<SmolStr>>()
        .drain(..)
    {
        try_replace_map_key_with_constant(file, prefix, key.as_str(), map, constants);
    }

    for (key, value) in map.iter_mut() {
        // Ignore sections that start with a non-content keyword.
        if is_keyword_for_non_constant_editable_section(key) {
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
    constants: &HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
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

fn constants_builder_recurse_into_value(
    file: &LoadableFile,
    key: &str,
    value: &mut Value,
    path: &mut SmallVec<[SmolStr; 10]>,
    constants: &mut HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
)
{
    // Update the value if it references a constant.
    // - We do this in a separate step in case expanding the constant introduces more constants/path segments.
    try_replace_string_with_constant(file, CONSTANT_IN_CONSTANT_MARKER, value, constants);

    // Parse constants from the value.
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => (),
        Value::Array(vec) => {
            for value in vec.iter_mut() {
                search_and_replace_constants(file, CONSTANT_IN_CONSTANT_MARKER, value, constants);
            }
        }
        //todo: it's ugly
        Value::Object(map) => {
            // Add path stack.
            path.push(key.into());

            let mut is_normal_segment = false;
            let mut is_constants_segment = false;

            for (key, value) in map.iter_mut() {
                if let Some(("", key)) = key.split_once(CONSTANT_MARKER) {
                    if is_normal_segment {
                        tracing::error!("ignoring constant section at {:?} in {:?}, constant path mixed up with value map",
                            path, file);
                        continue;
                    }
                    is_constants_segment = true;

                    // This entry in the data map adds to the constants map.
                    constants_builder_recurse_into_value(file, key, value, path, constants);
                } else {
                    if is_constants_segment {
                        tracing::error!("ignoring value section at {:?} in {:?}, constant path mixed up with value map",
                            path, file);
                        continue;
                    }
                    is_normal_segment = true;

                    // This key is a normal map entry, so its value is a normal value.
                    search_and_replace_constants(file, CONSTANT_IN_CONSTANT_MARKER, value, constants);
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
    let insert = |inner: &mut HashMap<SmolStr, Arc<Value>>| {
        let prev = inner.insert(SmolStr::from(key), Arc::new(value.clone()));
        if prev.is_some() {
            tracing::warn!("overwriting duplicate terminal path segment {} in constants map at {:?}", key, file);
        }
    };
    let base_path = path_to_constant_string(path);

    if let Some(inner) = constants.get_mut(&base_path) {
        insert(inner);
    } else {
        let mut inner = HashMap::default();
        insert(&mut inner);
        constants.insert(base_path, inner);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Pulls constants from a constants section.
pub(crate) fn extract_constants_section(
    file: &LoadableFile,
    data: &mut Map<String, Value>,
    constants: &mut HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
)
{
    let Some(constants_section) = data.get_mut(CONSTANTS_KEYWORD) else {
        return;
    };

    let Value::Object(ref mut data) = constants_section else {
        tracing::error!("failed parsing constants in {:?}, section is not an Object", file);
        return;
    };

    let mut path: SmallVec<[SmolStr; 10]> = SmallVec::default();

    // Replace map keys with constants.
    for key in data
        .keys()
        .filter(|k| k.starts_with(CONSTANT_IN_CONSTANT_MARKER))
        .map(|k| SmolStr::from(k))
        .collect::<Vec<SmolStr>>()
        .drain(..)
    {
        try_replace_map_key_with_constant(file, CONSTANT_IN_CONSTANT_MARKER, &key, data, constants);
    }

    // Iterate into the map to replace values.
    for (key, value) in data.iter_mut() {
        // Check if value.
        let Some(("", key)) = key.split_once(CONSTANT_MARKER) else {
            // Don't warn for comments.
            if !key.starts_with(COMMENT_KEYWORD) {
                tracing::warn!("ignoring non-path in base level of constants section in {:?}", file);
            }
            continue;
        };

        constants_builder_recurse_into_value(file, key, value, &mut path, constants);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::TypeRegistry;
use serde::de::DeserializeSeed;
use serde_json::Value;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn is_loadable_entry(key: &str) -> bool
{
    // Check if camelcase
    let Some(first_char) = key.chars().next() else {
        return false;
    };
    first_char.is_uppercase()
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn get_loadable_meta<'a>(
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
            there are multiple types with this short name, add its long name to the cobweb asset file's 'using' section",
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

pub(crate) fn get_loadable_value(deserializer: TypedReflectDeserializer, value: Value) -> ReflectedLoadable
{
    match deserializer.deserialize(value) {
        Ok(value) => ReflectedLoadable::Value(Arc::new(value)),
        Err(err) => ReflectedLoadable::DeserializationFailed(Arc::new(err)),
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn try_parse_spec_invocation(key: &str) -> Result<Option<(&str, &str)>, ()>
{
    // Expected format: "key(SPEC_INVOCATION_KEYWORDspec_key)"
    let Some((new_key, maybe_spec_req)) = key.split_once('(') else { return Ok(None) };
    let Some(("", maybe_spec_key)) = maybe_spec_req.split_once(SPEC_INVOCATION_KEYWORD) else { return Err(()) };
    let Some((spec_key, "")) = maybe_spec_key.split_once(')') else { return Err(()) };
    Ok(Some((new_key, spec_key)))
}

//-------------------------------------------------------------------------------------------------------------------

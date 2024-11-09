use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::TypeRegistry;
use serde::de::DeserializeSeed;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn get_loadable_meta<'a>(
    type_registry: &'a TypeRegistry,
    file: &CobFile,
    current_path: &ScenePath,
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

pub(super) fn get_loadable_value(deserializer: TypedReflectDeserializer, value: &CobLoadable)
    -> ReflectedLoadable
{
    match deserializer.deserialize(value) {
        Ok(value) => ReflectedLoadable::Value(Arc::new(value)),
        Err(err) => ReflectedLoadable::DeserializationFailed(Arc::new(err)),
    }
}

//-------------------------------------------------------------------------------------------------------------------

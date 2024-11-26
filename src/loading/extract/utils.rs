use std::any::TypeId;
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
    loadables: &LoadableRegistry,
) -> Option<(&'static str, &'static str, TypeId, TypedReflectDeserializer<'a>)>
{
    // Look up the registration.
    let registration = match loadables.get_type_id(short_name) {
        Some(type_id) => type_registry.get(type_id),
        None => {
            tracing::warn!("failed getting type id for loadable {} at {:?} in {:?}; no loadable with this name was \
                registered in the app",
                short_name, current_path, file);
            return None;
        }
    };

    // Look up the longname
    let Some(registration) = registration else {
        tracing::error!("failed getting type registration for {} at {:?} in {:?}; type was not registered in the app \
            (this is a bug)",
            short_name, current_path, file);
        return None;
    };

    let short_name = registration.type_info().type_path_table().short_path(); //get static version
    let long_name = registration.type_info().type_path_table().path();

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

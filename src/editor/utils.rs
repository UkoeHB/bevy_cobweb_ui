use std::any::TypeId;
use std::collections::HashMap;

use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::{TypeRegistration, TypeRegistry};

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn get_targeted<'a>(data: &'a mut Cob, editor_ref: &CobEditorRef) -> Option<&'a mut CobLoadable>
{
    match editor_ref.is_command() {
        true => data.get_command_loadable_mut(&*editor_ref.loadable_name),
        false => data.get_scene_loadable_mut(&editor_ref.scene_ref.path, &*editor_ref.loadable_name),
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn get_registration<'a>(
    type_registry: &'a TypeRegistry,
    short_name: &str,
    name_shortcuts: &HashMap<&'static str, &'static str>,
) -> Option<(&'a TypeRegistration, TypeId, &'static str, &'static str)>
{
    // Check if we already have this mapping.
    let registration = match name_shortcuts.get(short_name) {
        Some(long_name) => type_registry.get_with_type_path(long_name),
        None => type_registry.get_with_short_type_path(short_name),
    }?;

    let type_id = registration.type_info().type_id();
    let longname = registration.type_info().type_path_table().path();
    let shortname = registration.type_info().type_path_table().short_path(); //get static version

    Some((registration, type_id, longname, shortname))
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn get_deserializer<'a>(
    type_registry: &'a TypeRegistry,
    short_name: &str,
    name_shortcuts: &HashMap<&'static str, &'static str>,
) -> Option<(TypedReflectDeserializer<'a>, TypeId, &'static str, &'static str)>
{
    let (registration, type_id, longname, shortname) =
        get_registration(type_registry, short_name, name_shortcuts)?;
    let deserializer = TypedReflectDeserializer::new(registration, type_registry);
    Some((deserializer, type_id, longname, shortname))
}

//-------------------------------------------------------------------------------------------------------------------

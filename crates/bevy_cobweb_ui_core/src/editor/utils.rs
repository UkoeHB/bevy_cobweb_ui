use std::any::TypeId;

use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::{TypeRegistration, TypeRegistry};

use super::*;
use crate::cob::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub fn get_targeted<'a>(data: &'a mut Cob, editor_ref: &CobEditorRef) -> Option<&'a mut CobLoadable>
{
    match editor_ref.is_command() {
        true => data.get_command_loadable_mut(&*editor_ref.loadable_name),
        false => data.get_scene_loadable_mut(&editor_ref.scene_ref.path, &*editor_ref.loadable_name),
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn get_registration<'a>(
    type_registry: &'a TypeRegistry,
    short_name: &str,
    loadables: &LoadableRegistry,
) -> Option<(&'a TypeRegistration, TypeId, &'static str, &'static str)>
{
    let type_id = loadables.get_type_id(short_name)?;
    let registration = type_registry.get(type_id)?;

    let type_id = registration.type_info().type_id();
    let longname = registration.type_info().type_path_table().path();
    let shortname = registration.type_info().type_path_table().short_path(); //get static version

    Some((registration, type_id, longname, shortname))
}

//-------------------------------------------------------------------------------------------------------------------

pub fn get_deserializer<'a>(
    type_registry: &'a TypeRegistry,
    short_name: &str,
    loadables: &LoadableRegistry,
) -> Option<(TypedReflectDeserializer<'a>, TypeId, &'static str, &'static str)>
{
    let (registration, type_id, longname, shortname) = get_registration(type_registry, short_name, loadables)?;
    let deserializer = TypedReflectDeserializer::new(registration, type_registry);
    Some((deserializer, type_id, longname, shortname))
}

//-------------------------------------------------------------------------------------------------------------------

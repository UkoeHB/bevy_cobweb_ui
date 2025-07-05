use std::any::{type_name, TypeId};
use std::collections::HashMap;

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use super::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct CobWidgetRegistry
{
    /// [ longname : widget fn ]
    widgets: HashMap<&'static str, EditorWidgetSpawnFn>,
}

impl CobWidgetRegistry
{
    fn register_editor_widget<T: CobEditorWidget>(&mut self, registry: &TypeRegistry)
    {
        let Some(info) = registry.get(TypeId::of::<T::Value>()) else {
            tracing::warn!("failed registering editor widget {}; type {} is not registered in the app",
                type_name::<T>(), type_name::<T::Value>());
            return;
        };

        let long_path = info.type_info().type_path();

        if self.widgets.insert(long_path, T::try_spawn).is_some() {
            tracing::warn!("overwritting editor widget registration for type {}; only one CobEditorWidget \
                implementation should exist per type", long_path);
        }
    }

    pub fn get(&self, longname: &'static str) -> Option<EditorWidgetSpawnFn>
    {
        self.widgets.get(longname).copied()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// App extension trait for registering editor widgets.
pub trait CobWidgetAppExt
{
    /// Adds a [`CobEditorWidget`] to the app.
    fn register_editor_widget<T: CobEditorWidget>(&mut self) -> &mut Self;
}

impl CobWidgetAppExt for App
{
    fn register_editor_widget<T: CobEditorWidget>(&mut self) -> &mut Self
    {
        self.world_mut().resource_scope::<AppTypeRegistry, ()>(
            |world: &mut World, registry: Mut<AppTypeRegistry>| {
                let mut widget_registry = world.get_resource_or_init::<CobWidgetRegistry>();
                widget_registry.register_editor_widget::<T>(&registry.read());
            },
        );
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct CobWidgetRegistryPlugin;

impl Plugin for CobWidgetRegistryPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<CobWidgetRegistry>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

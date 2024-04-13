use crate::*;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use std::any::TypeId;
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct StyleLoaderDerived<T>
{
    conversion: fn(T, &mut EntityCommands),
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the style `React<T>` on entities.
fn reactive_style_loader<T: ReactComponent + ReflectableStyle>(
    mut rc       : ReactCommands,
    mut styles   : ReactResMut<StyleSheet>,
    mut entities : Query<Option<&mut React<T>>>
){
    styles.get_mut_noreact().update_reactive_styles::<T>(&mut rc, &mut entities);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the style bundle `T` on entities.
fn bundle_style_loader<T: Bundle + ReflectableStyle>(
    mut rc       : ReactCommands,
    mut styles   : ReactResMut<StyleSheet>,
){
    styles.get_mut_noreact().update_bundle_styles::<T>(&mut rc);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Uses `T` to derive changes on subscribed entities.
fn derived_style_loader<T: ReflectableStyle>(
    mut rc       : ReactCommands,
    mut styles   : ReactResMut<StyleSheet>,
    derived      : Res<StyleLoaderDerived<T>>,
){
    styles.get_mut_noreact().update_derived_styles::<T>(&mut rc, derived.conversion);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct StyleLoaderReactors
{
    reactors: HashMap<TypeId, SystemCommand>,
}

impl StyleLoaderReactors
{
    pub(crate) fn get(&self, type_id: TypeId) -> Option<SystemCommand>
    {
        self.reactors.get(&type_id).cloned()
    }
}

impl Default for StyleLoaderReactors
{
    fn default() -> Self
    {
        Self{ reactors: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component added to nodes that load styles from the stylesheet.
#[derive(Component)]
pub(crate) struct LoadedStyles;

//-------------------------------------------------------------------------------------------------------------------

/// Entity event emitted when styles have been updated on an entity.
#[derive(Debug, Default, Copy, Clone, Hash)]
pub struct StylesLoaded;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering entities for style loading.
pub trait StyleLoadingEntityCommandsExt
{
    /// Registers the current entity to load styles from `style_ref`.
    fn load(&mut self, style_ref: &StyleRef) -> &mut Self;
}

impl StyleLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn load(&mut self, style_ref: &StyleRef) -> &mut Self
    {
        self.insert(LoadedStyles);

        let id = self.id();
        self.commands().syscall((id, style_ref.clone()),
                |
                    In((id, style_ref)): In<(Entity, StyleRef)>,
                    mut rc: ReactCommands,
                    loaders: Res<StyleLoaderReactors>,
                    mut stylesheet: ReactResMut<StyleSheet>,
                |
                {
                    stylesheet.get_mut_noreact().track_entity(id, style_ref, &mut rc, &loaders);
                }
            );

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting [`StyleSheet`] use.
pub trait StyleRegistrationAppExt
{
    /// Registers a style type that will be inserted as [`T`] bundles on entities that subscribe to
    /// stylesheet paths containing the type.
    fn register_style<T: Bundle + ReflectableStyle>(&mut self) -> &mut Self;

    /// Registers a style type that will be inserted as [`React<T>`] components on entities that subscribe to
    /// stylesheet paths containing the type.
    fn register_reactive_style<T: ReactComponent + ReflectableStyle>(&mut self) -> &mut Self;

    /// Registers a style type that will be inserted as [`T`] bundles on entities that subscribe to
    /// stylesheet paths containing the type.
    fn register_derived_style<T: ReflectableStyle>(&mut self, conversion: fn(T, &mut EntityCommands)) -> &mut Self;
}

impl StyleRegistrationAppExt for App
{
    fn register_style<T: Bundle + ReflectableStyle>(&mut self) -> &mut Self
    {
        if !self.world.contains_resource::<StyleLoaderReactors>()
        {
            self.init_resource::<StyleLoaderReactors>();
        }

        CallbackSystem::new(
            |mut rc: ReactCommands, mut reactors: ResMut<StyleLoaderReactors>|
            {
                let entry = reactors.reactors.entry(TypeId::of::<T>());
                if matches!(entry, std::collections::hash_map::Entry::Occupied(_))
                {
                    tracing::warn!("tried registering bundle style {} multiple times", std::any::type_name::<T>());
                }

                entry.or_insert_with(
                        || rc.on_persistent(resource_mutation::<StyleSheet>(), bundle_style_loader::<T>)
                    );
            }
        ).run(&mut self.world, ());

        self
    }

    fn register_reactive_style<T: ReactComponent + ReflectableStyle>(&mut self) -> &mut Self
    {
        if !self.world.contains_resource::<StyleLoaderReactors>()
        {
            self.init_resource::<StyleLoaderReactors>();
        }

        CallbackSystem::new(
            |mut rc: ReactCommands, mut reactors: ResMut<StyleLoaderReactors>|
            {
                let entry = reactors.reactors.entry(TypeId::of::<T>());
                if matches!(entry, std::collections::hash_map::Entry::Occupied(_))
                {
                    tracing::warn!("tried registering reactive style {} multiple times", std::any::type_name::<T>());
                }

                entry.or_insert_with(
                        || rc.on_persistent(resource_mutation::<StyleSheet>(), reactive_style_loader::<T>)
                    );
            }
        ).run(&mut self.world, ());

        self
    }

    fn register_derived_style<T: ReflectableStyle>(&mut self, conversion: fn(T, &mut EntityCommands)) -> &mut Self
    {
        if !self.world.contains_resource::<StyleLoaderReactors>()
        {
            self.init_resource::<StyleLoaderReactors>();
        }

        CallbackSystem::new(
            move |mut rc: ReactCommands, mut reactors: ResMut<StyleLoaderReactors>|
            {
                let entry = reactors.reactors.entry(TypeId::of::<T>());
                if matches!(entry, std::collections::hash_map::Entry::Occupied(_))
                {
                    tracing::warn!("tried registering derived style {} multiple times", std::any::type_name::<T>());
                }

                entry.or_insert_with(
                        || rc.on_persistent(resource_mutation::<StyleSheet>(), derived_style_loader::<T>)
                    );
                rc.commands().insert_resource(StyleLoaderDerived{ conversion });
            }
        ).run(&mut self.world, ());

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct StyleLoaderPlugin;

impl Plugin for StyleLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<StyleLoaderReactors>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

use crate::*;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn register_style_impl<M, T: 'static>(
    app: &mut App,
    callback: impl IntoSystem<(), (), M> + Send + Sync + 'static + Copy,
    _p: PhantomData<T>,
    register_type: &'static str,
){
    if !app.world.contains_resource::<StyleLoaderCallbacks>()
    {
        app.init_resource::<StyleLoaderCallbacks>();
    }

    CallbackSystem::new(
        move |mut c: Commands, mut loaders: ResMut<StyleLoaderCallbacks>|
        {
            let entry = loaders.callbacks.entry(TypeId::of::<T>());
            if matches!(entry, std::collections::hash_map::Entry::Occupied(_))
            {
                tracing::warn!("tried registering {register_type} style {} multiple times", std::any::type_name::<T>());
            }

            entry.or_insert_with(
                    || c.react().on_persistent(resource_mutation::<StyleSheet>(), callback)
                );
        }
    ).run(&mut app.world, ());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the style `React<T>` on entities.
fn reactive_style_loader<T: ReactComponent + ReflectableStyle>(
    mut c        : Commands,
    mut styles   : ReactResMut<StyleSheet>,
    mut entities : Query<Option<&mut React<T>>>
){
    styles.get_noreact().update_styles::<T>(
        |entity, style_ref, style|
        {
            let Ok(component) = entities.get_mut(entity) else { return };
            let Some(new_val) = style.get_value(style_ref) else { return };

            match component
            {
                Some(mut component) =>
                {
                    *component.get_mut(&mut c) = new_val;
                }
                None =>
                {
                    c.react().insert(entity, new_val);
                }
            }

            c.react().entity_event::<StylesLoaded>(entity, StylesLoaded);
        }
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the style bundle `T` on entities.
fn bundle_style_loader<T: Bundle + ReflectableStyle>(
    mut c        : Commands,
    mut styles   : ReactResMut<StyleSheet>,
){
    styles.get_noreact().update_styles::<T>(
        |entity, style_ref, style|
        {
            let Some(bundle) = style.get_value::<T>(style_ref) else { return };
            let Some(mut ec) = c.get_entity(entity) else { return };
            ec.try_insert(bundle);

            c.react().entity_event::<StylesLoaded>(entity, StylesLoaded);
        }
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Uses `T` to derive changes on subscribed entities.
fn derived_style_loader<T: StyleToBevy + ReflectableStyle>(
    mut c        : Commands,
    mut styles   : ReactResMut<StyleSheet>,
){
    styles.get_noreact().update_styles::<T>(
        |entity, style_ref, style|
        {
            let Some(value) = style.get_value::<T>(style_ref) else { return };
            let Some(mut ec) = c.get_entity(entity) else { return };
            value.to_bevy(&mut ec);

            c.react().entity_event::<StylesLoaded>(entity, StylesLoaded);
        }
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub(crate) struct StyleLoaderCallbacks
{
    callbacks: HashMap<TypeId, SystemCommand>,
}

impl StyleLoaderCallbacks
{
    pub(crate) fn get(&self, type_id: TypeId) -> Option<SystemCommand>
    {
        self.callbacks.get(&type_id).cloned()
    }
}

impl Default for StyleLoaderCallbacks
{
    fn default() -> Self
    {
        Self{ callbacks: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for converting [`Self`] into entity modifications.
///
/// Used by [`register_derived_style`].
pub trait StyleToBevy
{
    fn to_bevy(self, ec: &mut EntityCommands);
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
    fn load_style(&mut self, style_ref: StyleRef) -> &mut Self;
}

impl StyleLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn load_style(&mut self, style_ref: StyleRef) -> &mut Self
    {
        self.insert(LoadedStyles);

        let id = self.id();
        self.commands().syscall((id, style_ref),
                |
                    In((id, style_ref)): In<(Entity, StyleRef)>,
                    mut c: Commands,
                    loaders: Res<StyleLoaderCallbacks>,
                    mut stylesheet: ReactResMut<StyleSheet>,
                |
                {
                    stylesheet.get_noreact().track_entity(id, style_ref, &mut c, &loaders);
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
    fn register_derived_style<T: StyleToBevy + ReflectableStyle>(&mut self) -> &mut Self;
}

impl StyleRegistrationAppExt for App
{
    fn register_style<T: Bundle + ReflectableStyle>(&mut self) -> &mut Self
    {
        register_style_impl(self, bundle_style_loader::<T>, PhantomData::<T>::default(), "bundle");
        self
    }

    fn register_reactive_style<T: ReactComponent + ReflectableStyle>(&mut self) -> &mut Self
    {
        register_style_impl(self, reactive_style_loader::<T>, PhantomData::<T>::default(), "reactive");
        self
    }

    fn register_derived_style<T: StyleToBevy + ReflectableStyle>(&mut self) -> &mut Self
    {
        register_style_impl(self, derived_style_loader::<T>, PhantomData::<T>::default(), "derived");
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct StyleLoaderPlugin;

impl Plugin for StyleLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<StyleLoaderCallbacks>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

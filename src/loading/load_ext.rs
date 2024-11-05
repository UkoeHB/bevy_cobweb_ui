use std::any::TypeId;
use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn register_command_loadable<T: Command + Loadable>(app: &mut App)
{
    let mut loaders = app
        .world_mut()
        .get_resource_or_insert_with::<LoaderCallbacks>(|| Default::default());

    let entry = loaders.command_callbacks.entry(TypeId::of::<T>());
    if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
        tracing::warn!("tried registering command loadable {} multiple times", std::any::type_name::<T>());
    }

    entry.or_insert(command_loader::<T>);
}

//-------------------------------------------------------------------------------------------------------------------

fn register_node_loadable<T: 'static>(
    app: &mut App,
    callback: fn(&mut World, Entity, ReflectedLoadable, SceneRef),
    _reverter: fn(Entity, &mut World),
    register_type: &'static str,
)
{
    let mut loaders = app
        .world_mut()
        .get_resource_or_insert_with::<LoaderCallbacks>(|| Default::default());

    // Applier callback.
    let entry = loaders.node_callbacks.entry(TypeId::of::<T>());
    if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
        tracing::warn!("tried registering {register_type} loadable {} multiple times", std::any::type_name::<T>());
    }

    entry.or_insert(callback);

    // Reverter callback.
    #[cfg(feature = "hot_reload")]
    loaders
        .revert_callbacks
        .entry(TypeId::of::<T>())
        .or_insert(_reverter);
}

//-------------------------------------------------------------------------------------------------------------------

/// Applies a loadable command of type `T`.
fn command_loader<T: Command + Loadable>(w: &mut World, loadable: ReflectedLoadable, loadable_ref: SceneRef)
{
    let Some(command) = loadable.get_value::<T>(&loadable_ref) else { return };
    command.apply(w);
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable bundle `T` on an entity.
fn bundle_loader<T: Bundle + Loadable>(
    w: &mut World,
    entity: Entity,
    loadable: ReflectedLoadable,
    loadable_ref: SceneRef,
)
{
    let Some(mut ec) = w.get_entity_mut(entity) else { return };
    let Some(bundle) = loadable.get_value::<T>(&loadable_ref) else { return };
    ec.insert(bundle);
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable `React<T>` on an entity.
fn reactive_loader<T: ReactComponent + Loadable>(
    w: &mut World,
    entity: Entity,
    loadable: ReflectedLoadable,
    loadable_ref: SceneRef,
)
{
    let Some(mut emut) = w.get_entity_mut(entity) else { return };
    let Some(new_val) = loadable.get_value(&loadable_ref) else { return };
    match emut.get_mut::<React<T>>() {
        Some(mut component) => {
            *component.get_noreact() = new_val;
            React::<T>::trigger_mutation(entity, w);
        }
        None => {
            w.react(|rc| rc.insert(entity, new_val));
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Uses `T` instruction on an entity.
fn instruction_loader<T: Instruction + Loadable>(
    w: &mut World,
    entity: Entity,
    loadable: ReflectedLoadable,
    loadable_ref: SceneRef,
)
{
    if !w.entities().contains(entity) {
        return;
    }
    let Some(value) = loadable.get_value::<T>(&loadable_ref) else { return };
    value.apply(entity, w);
}

//-------------------------------------------------------------------------------------------------------------------

fn load_from_ref(
    In((id, loadable_ref, initializer)): In<(Entity, SceneRef, NodeInitializer)>,
    mut c: Commands,
    loaders: Res<LoaderCallbacks>,
    mut scene_buffer: ResMut<SceneBuffer>,
    load_state: Res<State<LoadState>>,
)
{
    if *load_state.get() != LoadState::Done {
        tracing::error!("failed loading scene node {loadable_ref:?} into {id:?}, app state is not LoadState::Done");
        return;
    }

    scene_buffer.track_entity(id, loadable_ref, initializer, &loaders, &mut c);
}

//-------------------------------------------------------------------------------------------------------------------

// TODO (bevy v0.15): need to use `remove_with_requires`
fn revert_bundle<T: Bundle>(entity: Entity, world: &mut World)
{
    let Some(mut emut) = world.get_entity_mut(entity) else { return };
    emut.remove::<T>();
}

//-------------------------------------------------------------------------------------------------------------------

fn revert_reactive<T: ReactComponent>(entity: Entity, world: &mut World)
{
    let Some(mut emut) = world.get_entity_mut(entity) else { return };
    emut.remove::<React<T>>();
}

//-------------------------------------------------------------------------------------------------------------------

/// Same as `load_from_ref` except loads are queued instead of immediately executed.
#[cfg(feature = "hot_reload")]
pub(crate) fn load_queued_from_ref(
    In((id, loadable_ref, initializer)): In<(Entity, SceneRef, NodeInitializer)>,
    mut scene_buffer: ResMut<SceneBuffer>,
    load_state: Res<State<LoadState>>,
)
{
    if *load_state.get() != LoadState::Done {
        tracing::error!("failed loading scene node {loadable_ref:?} into {id:?}, app state is not LoadState::Done");
        return;
    }

    scene_buffer.track_entity_queued(id, loadable_ref, initializer);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
pub(crate) struct LoaderCallbacks
{
    command_callbacks: HashMap<TypeId, fn(&mut World, ReflectedLoadable, SceneRef)>,
    node_callbacks: HashMap<TypeId, fn(&mut World, Entity, ReflectedLoadable, SceneRef)>,
    #[cfg(feature = "hot_reload")]
    revert_callbacks: HashMap<TypeId, fn(Entity, &mut World)>,
}

impl LoaderCallbacks
{
    pub(crate) fn get_for_command(&self, type_id: TypeId) -> Option<fn(&mut World, ReflectedLoadable, SceneRef)>
    {
        self.command_callbacks.get(&type_id).cloned()
    }

    pub(crate) fn get_for_node(
        &self,
        type_id: TypeId,
    ) -> Option<fn(&mut World, Entity, ReflectedLoadable, SceneRef)>
    {
        self.node_callbacks.get(&type_id).cloned()
    }

    #[cfg(feature = "hot_reload")]
    pub(crate) fn get_for_revert(&self, type_id: TypeId) -> Option<fn(Entity, &mut World)>
    {
        self.revert_callbacks.get(&type_id).cloned()
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub(crate) struct NodeInitializer
{
    pub(crate) initializer: fn(&mut EntityCommands),
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting [`CobwebAssetCache`] use.
pub trait CobwebAssetRegistrationAppExt
{
    /// Registers a command that will be applied to the Bevy world when it is loaded.
    fn register_command<T: Command + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`Self::register_command`].
    fn register_command_type<T: TypePath + GetTypeRegistration + Command + Loadable>(&mut self) -> &mut Self;

    /// Registers a bundle that can be inserted on entities via CAF loadables.
    fn register_bundle<T: Bundle + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`Self::register_bundle`].
    fn register_bundle_type<T: TypePath + GetTypeRegistration + Bundle + Loadable>(&mut self) -> &mut Self;

    /// Registers a [`React<T>`] component that can be inserted on entities via CAF loadables.
    fn register_reactive<T: ReactComponent + Loadable>(&mut self) -> &mut Self;

    /// Registers an instruction that can be applied to entities via CAF loadables.
    fn register_instruction<T: Instruction + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`Self::register_instruction`].
    fn register_instruction_type<T: TypePath + GetTypeRegistration + Instruction + Loadable>(
        &mut self,
    ) -> &mut Self;
}

impl CobwebAssetRegistrationAppExt for App
{
    fn register_command<T: Command + Loadable>(&mut self) -> &mut Self
    {
        register_command_loadable::<T>(self);
        self
    }

    fn register_command_type<T: TypePath + GetTypeRegistration + Command + Loadable>(&mut self) -> &mut Self
    {
        self.register_type::<T>().register_command::<T>()
    }

    fn register_bundle<T: Bundle + Loadable>(&mut self) -> &mut Self
    {
        register_node_loadable::<T>(self, bundle_loader::<T>, revert_bundle::<T>, "bundle");
        self
    }

    fn register_bundle_type<T: TypePath + GetTypeRegistration + Bundle + Loadable>(&mut self) -> &mut Self
    {
        self.register_type::<T>().register_bundle::<T>()
    }

    fn register_reactive<T: ReactComponent + Loadable>(&mut self) -> &mut Self
    {
        register_node_loadable::<T>(self, reactive_loader::<T>, revert_reactive::<T>, "reactive");
        self
    }

    fn register_instruction<T: Instruction + Loadable>(&mut self) -> &mut Self
    {
        register_node_loadable::<T>(self, instruction_loader::<T>, T::revert, "instruction");
        self
    }

    fn register_instruction_type<T: TypePath + GetTypeRegistration + Instruction + Loadable>(
        &mut self,
    ) -> &mut Self
    {
        self.register_type::<T>().register_instruction::<T>()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering entities for loadable loading.
pub trait CafLoadingEntityCommandsExt
{
    /// Registers the current entity to load loadables from `loadable_ref`.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load(&mut self, loadable_ref: SceneRef) -> &mut Self;

    /// Registers the current entity to load loadables from `loadable_ref`.
    ///
    /// The `initializer` callback will be called before refreshing the `loadable_ref` loadable set on the entity.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load_with_initializer(&mut self, loadable_ref: SceneRef, initializer: fn(&mut EntityCommands))
        -> &mut Self;
}

impl CafLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn load(&mut self, loadable_ref: SceneRef) -> &mut Self
    {
        self.load_with_initializer(loadable_ref, |_| {})
    }

    fn load_with_initializer(&mut self, loadable_ref: SceneRef, initializer: fn(&mut EntityCommands))
        -> &mut Self
    {
        self.insert(HasLoadables);

        let id = self.id();
        self.commands()
            .syscall((id, loadable_ref, NodeInitializer { initializer }), load_from_ref);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadExtPlugin;

impl Plugin for LoadExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<LoaderCallbacks>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

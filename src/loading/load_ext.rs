use std::any::TypeId;
use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn validate_loadable_name(shortname: &'static str) -> bool
{
    if shortname.is_empty() || !shortname.chars().next().unwrap().is_ascii_uppercase() {
        tracing::warn!("failed registering loadable '{shortname}', type must be a named struct that starts uppercase");
        return false;
    }
    true
}

//-------------------------------------------------------------------------------------------------------------------

fn register_loadable_type<T: Loadable>(app: &mut App) -> Option<(&mut LoadableRegistry, TypeId)>
{
    // Look up canonical short name.
    let type_id = TypeId::of::<T>();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    let Some(registration) = registry.get(type_id) else {
        tracing::warn!("failed registering command loadable {} whose type is not registered in the app",
            std::any::type_name::<T>());
        return None;
    };
    let shortname = registration.type_info().type_path_table().short_path();
    std::mem::drop(registry);

    if !validate_loadable_name(shortname) {
        return None;
    }

    // Save loadable type.
    let mut loadables = app
        .world_mut()
        .get_resource_or_insert_with::<LoadableRegistry>(|| Default::default());

    if let Some(prev) = loadables.loadables.insert(shortname, type_id) {
        if prev != type_id {
            tracing::warn!("overwriting command loadable registration; new type id: {:?}, old type id: {:?}",
                type_id, prev);
        }
    }

    Some((loadables.into_inner(), type_id))
}

//-------------------------------------------------------------------------------------------------------------------

fn register_command_loadable<T: Command + Loadable>(app: &mut App)
{
    // Register type.
    let Some((loadables, type_id)) = register_loadable_type::<T>(app) else { return };

    // Add callback entry.
    let entry = loadables.command_callbacks.entry(type_id);
    if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
        tracing::warn!("tried registering command loadable {} multiple times", std::any::type_name::<T>());
    }

    entry.or_insert(command_loader::<T>);
}

//-------------------------------------------------------------------------------------------------------------------

fn register_node_loadable<T: Loadable + 'static>(
    app: &mut App,
    callback: fn(&mut World, Entity, ReflectedLoadable, SceneRef),
    _reverter: fn(Entity, &mut World),
    register_type: &'static str,
)
{
    // Register type.
    let Some((loadables, type_id)) = register_loadable_type::<T>(app) else { return };

    // Applier callback.
    let entry = loadables.node_callbacks.entry(type_id);
    if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
        tracing::warn!("tried registering {register_type} loadable {} multiple times", std::any::type_name::<T>());
    }

    entry.or_insert(callback);

    // Reverter callback.
    #[cfg(feature = "hot_reload")]
    loadables
        .revert_callbacks
        .entry(type_id)
        .or_insert(_reverter);
}

//-------------------------------------------------------------------------------------------------------------------

/// Applies a loadable command of type `T`.
fn command_loader<T: Command + Loadable>(w: &mut World, loadable: ReflectedLoadable, scene_ref: SceneRef)
{
    let registry = w.resource::<AppTypeRegistry>();
    let Some(command) = loadable.get_value::<T>(&scene_ref, &registry.read()) else { return };
    command.apply(w);
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable bundle `T` on an entity.
fn bundle_loader<T: Bundle + Loadable>(
    w: &mut World,
    entity: Entity,
    loadable: ReflectedLoadable,
    scene_ref: SceneRef,
)
{
    w.resource_scope(|world, registry: Mut<AppTypeRegistry>| {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        let Some(bundle) = loadable.get_value::<T>(&scene_ref, &registry.read()) else { return };
        emut.insert(bundle);
    });
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable `React<T>` on an entity.
fn reactive_loader<T: ReactComponent + Loadable>(
    w: &mut World,
    entity: Entity,
    loadable: ReflectedLoadable,
    scene_ref: SceneRef,
)
{
    w.resource_scope(|world, registry: Mut<AppTypeRegistry>| {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        let Some(new_val) = loadable.get_value(&scene_ref, &registry.read()) else { return };
        match emut.get_mut::<React<T>>() {
            Some(mut component) => {
                *component.get_noreact() = new_val;
                React::<T>::trigger_mutation(entity, world);
            }
            None => {
                world.react(|rc| rc.insert(entity, new_val));
            }
        }
    });
}

//-------------------------------------------------------------------------------------------------------------------

/// Uses `T` instruction on an entity.
fn instruction_loader<T: Instruction + Loadable>(
    w: &mut World,
    entity: Entity,
    loadable: ReflectedLoadable,
    scene_ref: SceneRef,
)
{
    if !w.entities().contains(entity) {
        return;
    }
    let registry = w.resource::<AppTypeRegistry>();
    let Some(value) = loadable.get_value::<T>(&scene_ref, &registry.read()) else { return };
    value.apply(entity, w);
}

//-------------------------------------------------------------------------------------------------------------------

fn load_from_ref(
    In((id, scene_ref, initializer)): In<(Entity, SceneRef, NodeInitializer)>,
    mut c: Commands,
    loadables: Res<LoadableRegistry>,
    mut scene_buffer: ResMut<SceneBuffer>,
    load_state: Res<State<LoadState>>,
    #[cfg(feature = "hot_reload")] commands_buffer: Res<CommandsBuffer>,
)
{
    if *load_state.get() != LoadState::Done {
        tracing::error!("failed loading scene node {scene_ref:?} into {id:?}, app state is not LoadState::Done");
        return;
    }

    scene_buffer.track_entity(
        id,
        scene_ref,
        initializer,
        &loadables,
        &mut c,
        #[cfg(feature = "hot_reload")]
        &commands_buffer,
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn revert_bundle<T: Bundle>(entity: Entity, world: &mut World)
{
    let Ok(mut emut) = world.get_entity_mut(entity) else { return };
    emut.remove_with_requires::<T>();
}

//-------------------------------------------------------------------------------------------------------------------

fn revert_reactive<T: ReactComponent>(entity: Entity, world: &mut World)
{
    let Ok(mut emut) = world.get_entity_mut(entity) else { return };
    emut.remove::<React<T>>();
}

//-------------------------------------------------------------------------------------------------------------------

/// Same as `load_from_ref` except loads are queued instead of immediately executed.
#[cfg(feature = "hot_reload")]
pub(crate) fn load_queued_from_ref(
    In((id, scene_ref, initializer)): In<(Entity, SceneRef, NodeInitializer)>,
    mut scene_buffer: ResMut<SceneBuffer>,
    load_state: Res<State<LoadState>>,
)
{
    if *load_state.get() != LoadState::Done {
        tracing::error!("failed loading scene node {scene_ref:?} into {id:?}, app state is not LoadState::Done");
        return;
    }

    scene_buffer.track_entity_queued(id, scene_ref, initializer);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
pub(crate) struct LoadableRegistry
{
    /// [ short name : type id ]
    loadables: HashMap<&'static str, TypeId>,

    command_callbacks: HashMap<TypeId, fn(&mut World, ReflectedLoadable, SceneRef)>,
    node_callbacks: HashMap<TypeId, fn(&mut World, Entity, ReflectedLoadable, SceneRef)>,
    #[cfg(feature = "hot_reload")]
    revert_callbacks: HashMap<TypeId, fn(Entity, &mut World)>,
}

impl LoadableRegistry
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

    pub(crate) fn get_type_id(&self, id: impl AsRef<str>) -> Option<TypeId>
    {
        self.loadables.get(id.as_ref()).copied()
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub(crate) struct NodeInitializer
{
    pub(crate) initializer: fn(&mut EntityCommands),
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods for registering loadables.
///
/// All registered loadable types must have unique short names (e.g. `BorderColor`).
pub trait CobLoadableRegistrationAppExt
{
    /// Registers a command that will be applied to the Bevy world when it is loaded.
    fn register_command<T: Command + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`CobLoadableRegistrationAppExt::register_command`].
    fn register_command_type<T: TypePath + GetTypeRegistration + Command + Loadable>(&mut self) -> &mut Self;

    /// Registers a component that can be inserted on entities via COB loadables.
    fn register_component<T: Component + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`CobLoadableRegistrationAppExt::register_component`].
    fn register_component_type<T: TypePath + GetTypeRegistration + Component + Loadable>(&mut self) -> &mut Self;

    /// Registers a bundle that can be inserted on entities via COB loadables.
    fn register_bundle<T: Bundle + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`CobLoadableRegistrationAppExt::register_bundle`].
    fn register_bundle_type<T: TypePath + GetTypeRegistration + Bundle + Loadable>(&mut self) -> &mut Self;

    /// Registers a [`React<T>`] component that can be inserted on entities via COB loadables.
    fn register_reactive<T: ReactComponent + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`CobLoadableRegistrationAppExt::register_reactive`].
    fn register_reactive_type<T: TypePath + GetTypeRegistration + ReactComponent + Loadable>(
        &mut self,
    ) -> &mut Self;

    /// Registers an instruction that can be applied to entities via COB loadables.
    fn register_instruction<T: Instruction + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`CobLoadableRegistrationAppExt::register_instruction`].
    fn register_instruction_type<T: TypePath + GetTypeRegistration + Instruction + Loadable>(
        &mut self,
    ) -> &mut Self;
}

impl CobLoadableRegistrationAppExt for App
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

    fn register_component<T: Component + Loadable>(&mut self) -> &mut Self
    {
        register_node_loadable::<T>(self, bundle_loader::<T>, revert_bundle::<T>, "component");
        self
    }

    fn register_component_type<T: TypePath + GetTypeRegistration + Component + Loadable>(&mut self) -> &mut Self
    {
        self.register_type::<T>().register_component::<T>()
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

    fn register_reactive_type<T: TypePath + GetTypeRegistration + ReactComponent + Loadable>(
        &mut self,
    ) -> &mut Self
    {
        self.register_type::<T>().register_reactive::<T>()
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
pub trait CobLoadingEntityCommandsExt
{
    /// Registers the current entity to load loadables from `scene_ref`.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load(&mut self, scene_ref: SceneRef) -> &mut Self;

    /// Registers the current entity to load loadables from `scene_ref`.
    ///
    /// The `initializer` callback will be called before refreshing the `scene_ref` loadable set on the entity.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load_with_initializer(&mut self, scene_ref: SceneRef, initializer: fn(&mut EntityCommands)) -> &mut Self;
}

impl CobLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn load(&mut self, scene_ref: SceneRef) -> &mut Self
    {
        self.load_with_initializer(scene_ref, |_| {})
    }

    fn load_with_initializer(&mut self, scene_ref: SceneRef, initializer: fn(&mut EntityCommands)) -> &mut Self
    {
        self.insert(HasLoadables);

        let id = self.id();
        self.commands()
            .syscall((id, scene_ref, NodeInitializer { initializer }), load_from_ref);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadExtPlugin;

impl Plugin for LoadExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<LoadableRegistry>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// We must add a separate reactor for commands when `hot_reload` is disabled because otherwise commands won't be
/// applied.
///
/// It's not a problem for entity loadables, where we expect all entities to be spawned after loading is done.
/// There's no case of needing to load something into a pre-existing entity after a file is loaded.
#[cfg(not(feature = "hot_reload"))]
fn apply_commands_manual(mut c: Commands, caf_cache: ReactRes<CobwebAssetCache>, loaders: Res<LoaderCallbacks>)
{
    caf_cache.apply_pending_commands(&mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

fn register_loadable_impl<M, T: 'static>(
    app: &mut App,
    callback: impl IntoSystem<(), (), M> + Send + Sync + 'static + Copy,
    _p: PhantomData<T>,
    register_type: &'static str,
)
{
    let mut loaders = app
        .world_mut()
        .remove_resource::<LoaderCallbacks>()
        .unwrap_or_default();

    let mut c = app.world_mut().commands();

    let entry = loaders.callbacks.entry(TypeId::of::<T>());
    if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
        tracing::warn!("tried registering {register_type} loadable {} multiple times", std::any::type_name::<T>());
    }

    #[cfg(feature = "hot_reload")]
    {
        entry.or_insert_with(|| {
            c.react()
                .on_persistent(resource_mutation::<CobwebAssetCache>(), callback)
        });
    }
    #[cfg(not(feature = "hot_reload"))]
    {
        entry.or_insert_with(|| c.spawn_system_command(callback));
    }

    app.world_mut().flush();
    app.insert_resource(loaders);
}

//-------------------------------------------------------------------------------------------------------------------

/// Applies loadable commands of type `T`.
fn command_loader<T: Command + Loadable>(world: &mut World)
{
    let mut commands = world
        .react_resource_mut_noreact::<CobwebAssetCache>()
        .take_commands::<T>();

    for (loadable, loadable_ref) in commands.drain(..) {
        let Some(command) = loadable.get_value::<T>(&loadable_ref) else { continue };
        command.apply(world);
    }

    world.flush();
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable bundle `T` on entities.
fn bundle_loader<T: Bundle + Loadable>(world: &mut World)
{
    let mut loadables = world
        .react_resource_mut_noreact::<CobwebAssetCache>()
        .take_loadables::<T>();

    for (loadable, loadable_ref, mut subscriptions) in loadables.drain(..) {
        let Some(bundle) = loadable.get_value::<T>(&loadable_ref) else { continue };

        let mut applier = |entity: Entity, setter: ContextSetter, bundle: T| {
            let Some(mut ec) = world.get_entity_mut(entity) else { return };
            ec.world_scope(|w| (setter.setter)(entity, w));
            ec.insert(bundle);

            #[cfg(feature = "hot_reload")]
            {
                world.react(|rc| rc.entity_event(entity, Loaded));
            }
        };

        // Do a little dance to avoid an extra clone.
        for SubscriptionRef { entity, setter } in subscriptions.drain(0..subscriptions.len().saturating_sub(1)) {
            (applier)(entity, setter, bundle.clone());
        }
        if subscriptions.len() == 1 {
            let SubscriptionRef { entity, setter } = subscriptions.remove(0);
            (applier)(entity, setter, bundle);
        }
    }

    world.flush();
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable `React<T>` on entities.
fn reactive_loader<T: ReactComponent + Loadable>(
    mut c: Commands,
    mut caf_cache: ReactResMut<CobwebAssetCache>,
    mut entities: Query<Option<&mut React<T>>>,
)
{
    caf_cache
        .get_noreact()
        .update_loadables::<T>(|entity, context_setter, loadable_ref, loadable| {
            let Ok(component) = entities.get_mut(entity) else { return };
            let Some(new_val) = loadable.get_value(loadable_ref) else { return };

            let mut ec = c.entity(entity);
            ec.add(move |entity: Entity, world: &mut World| (context_setter.setter)(entity, world));

            match component {
                Some(mut component) => {
                    *component.get_mut(&mut c) = new_val;
                }
                None => {
                    c.react().insert(entity, new_val);
                }
            }

            #[cfg(feature = "hot_reload")]
            {
                c.react().entity_event(entity, Loaded);
            }
        });
}

//-------------------------------------------------------------------------------------------------------------------

/// Uses `T` to derive changes on subscribed entities.
fn instruction_loader<T: Instruction + Loadable>(world: &mut World)
{
    let mut loadables = world
        .react_resource_mut_noreact::<CobwebAssetCache>()
        .take_loadables::<T>();

    for (loadable, loadable_ref, mut subscriptions) in loadables.drain(..) {
        let Some(value) = loadable.get_value::<T>(&loadable_ref) else { continue };

        let mut applier = |entity: Entity, setter: ContextSetter, value: T| {
            (setter.setter)(entity, world);
            value.apply(entity, world);

            #[cfg(feature = "hot_reload")]
            {
                world.react(|rc| rc.entity_event(entity, Loaded));
            }
        };

        // Do a little dance to avoid an extra clone.
        for SubscriptionRef { entity, setter } in subscriptions.drain(0..subscriptions.len().saturating_sub(1)) {
            (applier)(entity, setter, value.clone());
        }
        if subscriptions.len() == 1 {
            let SubscriptionRef { entity, setter } = subscriptions.remove(0);
            (applier)(entity, setter, value);
        }
    }

    world.flush();
}

//-------------------------------------------------------------------------------------------------------------------

fn load_from_ref(
    In((id, loadable_ref, setter)): In<(Entity, SceneRef, ContextSetter)>,
    mut c: Commands,
    loaders: Res<LoaderCallbacks>,
    mut caf_cache: ReactResMut<CobwebAssetCache>,
    load_state: Res<State<LoadState>>,
)
{
    if *load_state.get() != LoadState::Done {
        tracing::error!("failed loading scene node {loadable_ref:?} into {id:?}, app state is not LoadState::Done");
        return;
    }

    caf_cache
        .get_noreact()
        .track_entity(id, loadable_ref, setter, &mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
pub(crate) struct LoaderCallbacks
{
    callbacks: HashMap<TypeId, SystemCommand>,
}

impl LoaderCallbacks
{
    pub(crate) fn get(&self, type_id: TypeId) -> Option<SystemCommand>
    {
        self.callbacks.get(&type_id).cloned()
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub(crate) struct ContextSetter
{
    pub(crate) setter: fn(Entity, &mut World),
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
        register_loadable_impl(self, command_loader::<T>, PhantomData::<T>, "command");
        self
    }

    fn register_command_type<T: TypePath + GetTypeRegistration + Command + Loadable>(&mut self) -> &mut Self
    {
        self.register_type::<T>().register_command::<T>()
    }

    fn register_bundle<T: Bundle + Loadable>(&mut self) -> &mut Self
    {
        register_loadable_impl(self, bundle_loader::<T>, PhantomData::<T>, "bundle");
        self
    }

    fn register_bundle_type<T: TypePath + GetTypeRegistration + Bundle + Loadable>(&mut self) -> &mut Self
    {
        self.register_type::<T>().register_bundle::<T>()
    }

    fn register_reactive<T: ReactComponent + Loadable>(&mut self) -> &mut Self
    {
        register_loadable_impl(self, reactive_loader::<T>, PhantomData::<T>, "reactive");
        self
    }

    fn register_instruction<T: Instruction + Loadable>(&mut self) -> &mut Self
    {
        register_loadable_impl(self, instruction_loader::<T>, PhantomData::<T>, "instruction");
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
    /// The `setter` callback will be called every time a loadable is applied from the `loadable_ref` for this
    /// entity.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load_with_context_setter(&mut self, loadable_ref: SceneRef, setter: fn(Entity, &mut World)) -> &mut Self;
}

impl CafLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn load(&mut self, loadable_ref: SceneRef) -> &mut Self
    {
        self.load_with_context_setter(loadable_ref, |_, _| {})
    }

    fn load_with_context_setter(&mut self, loadable_ref: SceneRef, setter: fn(Entity, &mut World)) -> &mut Self
    {
        self.insert(HasLoadables);

        let id = self.id();
        self.commands()
            .syscall((id, loadable_ref, ContextSetter { setter }), load_from_ref);
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

        #[cfg(not(feature = "hot_reload"))]
        app.react(|rc| rc.on_persistent(resource_mutation::<CobwebAssetCache>(), apply_commands_manual));
    }
}

//-------------------------------------------------------------------------------------------------------------------

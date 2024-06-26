use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::ecs::system::{CommandQueue, EntityCommands};
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// We must add a separate reactor for commands when `hot_reload` is disabled because otherwise commands won't be
/// applied.
///
/// It's not a problem for entity loadables, where we expect all entities to be spawned after loading is done so
/// there's no case of needing to load something into an entity after a file is loaded.
#[cfg(not(feature = "hot_reload"))]
fn apply_commands_manual(mut c: Commands)
{
    c.react().on_persistent(
        resource_mutation::<LoadableSheet>(),
        |mut c: Commands, loadables: ReactRes<LoadableSheet>, loaders: Res<LoaderCallbacks>| {
            loadables.apply_pending_commands(&mut c, &loaders);
        },
    );
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
        .world
        .remove_resource::<LoaderCallbacks>()
        .unwrap_or_default();

    //todo: use commands from the world directly.
    let mut queue = CommandQueue::default();
    let mut c = Commands::new(&mut queue, &mut app.world);

    let entry = loaders.callbacks.entry(TypeId::of::<T>());
    if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
        tracing::warn!("tried registering {register_type} loadable {} multiple times", std::any::type_name::<T>());
    }

    #[cfg(feature = "hot_reload")]
    {
        entry.or_insert_with(|| {
            c.react()
                .on_persistent(resource_mutation::<LoadableSheet>(), callback)
        });
    }
    #[cfg(not(feature = "hot_reload"))]
    {
        entry.or_insert_with(|| c.spawn_system_command(callback));
    }

    queue.apply(&mut app.world);
    app.world.insert_resource(loaders);
}

//-------------------------------------------------------------------------------------------------------------------

/// Applies loadable commands of type `T`.
fn command_loader<T: ApplyCommand + Loadable>(mut c: Commands, mut loadables: ReactResMut<LoadableSheet>)
{
    loadables
        .get_noreact()
        .apply_commands::<T>(|loadable_ref, loadable| {
            let Some(command) = loadable.get_value::<T>(loadable_ref) else { return };
            command.apply(&mut c);
        });
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable bundle `T` on entities.
fn bundle_loader<T: Bundle + Loadable>(mut c: Commands, mut loadables: ReactResMut<LoadableSheet>)
{
    loadables
        .get_noreact()
        .update_loadables::<T>(|entity, context_setter, loadable_ref, loadable| {
            let Some(bundle) = loadable.get_value::<T>(loadable_ref) else { return };
            let Some(mut ec) = c.get_entity(entity) else { return };

            (context_setter.setter)(&mut ec);
            ec.try_insert(bundle);

            #[cfg(feature = "hot_reload")]
            {
                c.react().entity_event(entity, Loaded);
            }
        });
}

//-------------------------------------------------------------------------------------------------------------------

/// Updates the loadable `React<T>` on entities.
fn reactive_loader<T: ReactComponent + Loadable>(
    mut c: Commands,
    mut loadables: ReactResMut<LoadableSheet>,
    mut entities: Query<Option<&mut React<T>>>,
)
{
    loadables
        .get_noreact()
        .update_loadables::<T>(|entity, context_setter, loadable_ref, loadable| {
            let Ok(component) = entities.get_mut(entity) else { return };
            let Some(new_val) = loadable.get_value(loadable_ref) else { return };

            let mut ec = c.entity(entity);
            (context_setter.setter)(&mut ec);

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
fn derived_loader<T: ApplyLoadable + Loadable>(mut c: Commands, mut loadables: ReactResMut<LoadableSheet>)
{
    loadables
        .get_noreact()
        .update_loadables::<T>(|entity, context_setter, loadable_ref, loadable| {
            let Some(value) = loadable.get_value::<T>(loadable_ref) else { return };
            let Some(mut ec) = c.get_entity(entity) else { return };

            (context_setter.setter)(&mut ec);
            value.apply(&mut ec);

            #[cfg(feature = "hot_reload")]
            {
                c.react().entity_event(entity, Loaded);
            }
        });
}

//-------------------------------------------------------------------------------------------------------------------

fn load_from_ref(
    In((id, loadable_ref, setter)): In<(Entity, LoadableRef, ContextSetter)>,
    mut c: Commands,
    loaders: Res<LoaderCallbacks>,
    mut loadablesheet: ReactResMut<LoadableSheet>,
)
{
    loadablesheet
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

#[derive(Copy, Clone)]
pub(crate) struct ContextSetter
{
    pub(crate) setter: fn(&mut EntityCommands),
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting [`LoadableSheet`] use.
pub trait LoadableRegistrationAppExt
{
    /// Registers a loadable type that will be applied to the Bevy world when it is loaded.
    fn register_command_loadable<T: ApplyCommand + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`Self::register_command_loadable`].
    fn register_command<T: TypePath + GetTypeRegistration + ApplyCommand + Loadable>(&mut self) -> &mut Self;

    /// Registers a loadable type that will be inserted as [`T`] bundles on entities that subscribe to
    /// loadablesheet paths containing the type.
    fn register_loadable<T: Bundle + Loadable>(&mut self) -> &mut Self;

    /// Registers a loadable type that will be inserted as [`React<T>`] components on entities that subscribe to
    /// loadablesheet paths containing the type.
    fn register_reactive_loadable<T: ReactComponent + Loadable>(&mut self) -> &mut Self;

    /// Registers a loadable type that will be inserted as [`T`] bundles on entities that subscribe to
    /// loadablesheet paths containing the type.
    fn register_derived_loadable<T: ApplyLoadable + Loadable>(&mut self) -> &mut Self;

    /// Combines [`App::register_type`] with [`Self::register_derived_loadable`].
    fn register_derived<T: TypePath + GetTypeRegistration + ApplyLoadable + Loadable>(&mut self) -> &mut Self;
}

impl LoadableRegistrationAppExt for App
{
    fn register_command_loadable<T: ApplyCommand + Loadable>(&mut self) -> &mut Self
    {
        register_loadable_impl(self, command_loader::<T>, PhantomData::<T>, "command");
        self
    }

    fn register_command<T: TypePath + GetTypeRegistration + ApplyCommand + Loadable>(&mut self) -> &mut Self
    {
        self.register_type::<T>()
            .register_type::<Vec<T>>()
            .register_type::<Multi<T>>()
            .register_command_loadable::<T>()
            .register_command_loadable::<Multi<T>>()
    }

    fn register_loadable<T: Bundle + Loadable>(&mut self) -> &mut Self
    {
        register_loadable_impl(self, bundle_loader::<T>, PhantomData::<T>, "bundle");
        self
    }

    fn register_reactive_loadable<T: ReactComponent + Loadable>(&mut self) -> &mut Self
    {
        register_loadable_impl(self, reactive_loader::<T>, PhantomData::<T>, "reactive");
        self
    }

    fn register_derived_loadable<T: ApplyLoadable + Loadable>(&mut self) -> &mut Self
    {
        register_loadable_impl(self, derived_loader::<T>, PhantomData::<T>, "derived");
        self
    }

    fn register_derived<T: TypePath + GetTypeRegistration + ApplyLoadable + Loadable>(&mut self) -> &mut Self
    {
        self.register_type::<T>()
            .register_type::<Vec<T>>()
            .register_type::<Multi<T>>()
            .register_derived_loadable::<T>()
            .register_derived_loadable::<Multi<T>>()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering entities for loadable loading.
pub trait StyleLoadingEntityCommandsExt
{
    /// Registers the current entity to load loadables from `loadable_ref`.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load(&mut self, loadable_ref: LoadableRef) -> &mut Self;

    /// Registers the current entity to load loadables from `loadable_ref`.
    ///
    /// The `setter` callback will be called every time a loadable is applied from the `loadable_ref` for this
    /// entity.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load_with_context_setter(
        &mut self,
        loadable_ref: LoadableRef,
        setter: fn(&mut EntityCommands),
    ) -> &mut Self;
}

impl StyleLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn load(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        self.load_with_context_setter(loadable_ref, |_| {})
    }

    fn load_with_context_setter(&mut self, loadable_ref: LoadableRef, setter: fn(&mut EntityCommands))
        -> &mut Self
    {
        self.insert(HasLoadables);

        let id = self.id();
        self.commands()
            .syscall((id, loadable_ref, ContextSetter { setter }), load_from_ref);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoaderPlugin;

impl Plugin for LoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<LoaderCallbacks>();

        #[cfg(not(feature = "hot_reload"))]
        app.add_systems(Startup, apply_commands_manual);
    }
}

//-------------------------------------------------------------------------------------------------------------------

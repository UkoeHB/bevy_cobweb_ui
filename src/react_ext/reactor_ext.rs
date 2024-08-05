use std::any::TypeId;
use std::marker::PhantomData;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
fn register_update_on_reactor<Triggers: ReactionTriggerBundle>(
    In((entity, syscommand, triggers)): In<(Entity, SystemCommand, Triggers)>,
    mut c: Commands,
    loaded: Query<(), With<HasLoadables>>,
)
{
    // If there are no triggers then we should despawn the reactor immediately.
    let is_loaded = loaded.contains(entity);
    if !is_loaded && (TypeId::of::<Triggers>() == TypeId::of::<()>()) {
        c.add(syscommand);
        c.add(move |world: &mut World| {
            world.despawn(*syscommand);
        });
        return;
    }

    // Otherwise, prepare the reactor.
    let revoke_token = if is_loaded {
        let triggers = (triggers, entity_event::<Loaded>(entity));

        c.react()
            .with(triggers, syscommand, ReactorMode::Revokable)
            .unwrap()
    } else {
        c.react()
            .with(triggers, syscommand, ReactorMode::Revokable)
            .unwrap()
    };

    //todo: more efficient cleanup mechanism
    cleanup_reactor_on_despawn(&mut c, entity, revoke_token);

    // Run the system to apply it.
    c.add(syscommand);
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(not(feature = "hot_reload"))]
fn register_update_on_reactor<Triggers: ReactionTriggerBundle>(
    c: &mut Commands,
    entity: Entity,
    syscommand: SystemCommand,
    triggers: Triggers,
)
{
    // If there are no triggers then we should despawn the reactor immediately after running it.
    if TypeId::of::<Triggers>() == TypeId::of::<()>() {
        c.add(syscommand);
        c.add(move |world: &mut World| {
            world.despawn(*syscommand);
        });
        return;
    }

    // Otherwise, prepare reactor.
    let revoke_token = c
        .react()
        .with(triggers, syscommand, ReactorMode::Revokable)
        .unwrap();

    //todo: more efficient cleanup mechanism
    cleanup_reactor_on_despawn(c, entity, revoke_token);

    // Run the system to apply it.
    c.add(syscommand);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper struct returned by [`on_event`](UiReactEntityCommandsExt::on_event).
///
/// Call [`Self::r`] to add a reactor.
pub struct OnEventExt<'a, T: Send + Sync + 'static>
{
    ec: EntityCommands<'a>,
    _p: PhantomData<T>,
}

impl<'a, T: Send + Sync + 'static> OnEventExt<'a, T>
{
    pub(crate) fn new(ec: EntityCommands<'a>) -> OnEventExt<'a, T>
    {
        Self { ec, _p: PhantomData }
    }

    /// Adds a reactor to an [`on_event`](UiReactEntityCommandsExt::on_event) request.
    pub fn r<M>(mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> EntityCommands<'a>
    {
        let id = self.ec.id();
        self.ec.react().on(entity_event::<T>(id), callback);

        self.ec
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering reactors for node entities.
pub trait UiReactEntityCommandsExt
{
    /// Inserts a reactive component to the entity.
    ///
    /// The component can be accessed with the [`React<T>`] component, or with the
    /// [`Reactive`]/[`ReactiveMut`] system parameters.
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self;

    /// Inserts a derived value to the entity.
    ///
    /// Uses `T::ApplyLoadable` to convert the value into entity mutations.
    fn insert_derived<T: ApplyLoadable>(&mut self, value: T) -> &mut Self;

    /// Registers an [`entity_event`] reactor for the current entity.
    ///
    /// Use [`OnEventExt::r`] to register the reactor.
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>;

    /// Recursively despawns the current entity on entity event `T`.
    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Recursively despawns the current entity on broadcast event `T`.
    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Updates an entity with a reactor system.
    ///
    /// The system runs:
    /// - Immediately after being registered.
    /// - Whenever the triggers fire.
    /// - When an entity with the internal `HasLoadables` component receives `Loaded` events (`hot_reload` feature
    ///   only).
    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: R) -> &mut Self
    where
        C: IntoSystem<(), (), M> + Send + Sync + 'static,
        T: ReactionTriggerBundle,
        R: FnOnce(Entity) -> C;
}

impl UiReactEntityCommandsExt for EntityCommands<'_>
{
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self
    {
        let id = self.id();
        self.commands().react().insert(id, component);
        self
    }

    fn insert_derived<T: ApplyLoadable>(&mut self, value: T) -> &mut Self
    {
        self.add(move |entity: Entity, world: &mut World| value.apply(entity, world));
        self
    }

    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>
    {
        OnEventExt::new(self.reborrow())
    }

    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        let entity = self.id();
        self.on_event::<T>().r(move |mut c: Commands| {
            c.get_entity(entity).map(|e| e.despawn_recursive());
        });
        self
    }

    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        let entity = self.id();
        self.react().once(broadcast::<T>(), move |mut c: Commands| {
            c.get_entity(entity).map(|e| e.despawn_recursive());
        });
        self
    }

    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: R) -> &mut Self
    where
        C: IntoSystem<(), (), M> + Send + Sync + 'static,
        T: ReactionTriggerBundle,
        R: FnOnce(Entity) -> C,
    {
        let id = self.id();
        let callback = (reactor)(id);
        let syscommand = self.commands().spawn_system_command(callback);
        #[cfg(feature = "hot_reload")]
        {
            self.commands()
                .syscall((id, syscommand, triggers), register_update_on_reactor);
        }
        #[cfg(not(feature = "hot_reload"))]
        {
            register_update_on_reactor(&mut self.commands(), id, syscommand, triggers);
        }

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ReactorExtPlugin;

impl Plugin for ReactorExtPlugin
{
    fn build(&self, _app: &mut App) {}
}

//-------------------------------------------------------------------------------------------------------------------

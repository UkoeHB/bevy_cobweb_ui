use std::marker::PhantomData;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn register_update_on_reactor<Triggers: ReactionTriggerBundle>(
    In((entity, syscommand, triggers)): In<(Entity, SystemCommand, Triggers)>,
    mut c: Commands,
    loaded: Query<(), With<HasLoadables>>,
)
{
    // Detect loadables if appropriate.
    let revoke_token = if loaded.contains(entity) {
        let triggers = (triggers, entity_event::<Loaded>(entity));
        c.react().with(triggers, syscommand, ReactorMode::Revokable).unwrap()
    } else {
        c.react().with(triggers, syscommand, ReactorMode::Revokable).unwrap()
    };

    //todo: more efficient cleanup mechanism
    cleanup_reactor_on_despawn(&mut c, entity, revoke_token);
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
        Self { ec, _p: PhantomData::default() }
    }

    /// Adds a reactor to an [`on_event`](UiReactEntityCommandsExt::on_event) request.
    pub fn r<M>(mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> EntityCommands<'a>
    {
        let syscommand = self.ec.commands().spawn_system_command(callback);
        let id = self.ec.id();
        self.ec
            .commands()
            .react()
            .with(entity_event::<T>(id), syscommand, ReactorMode::Cleanup);

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
    /// Uses [`T::ApplyLoadable`] to convert the value into entity mutations.
    fn insert_derived<T: ApplyLoadable>(&mut self, value: T) -> &mut Self;

    /// Registers an [`entity_event`] reactor for the current entity.
    ///
    /// Use [`OnEventExt::r`] to register the reactor.
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>;

    /// Updates an entity with a reactor system.
    ///
    /// The system runs:
    /// - Immediately after being registered.
    /// - Whenever the triggers fire.
    /// - When an entity with [`HasLoadables`] receives [`Loaded`] events.
    fn update_on<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        triggers: impl ReactionTriggerBundle,
        reactor: impl FnOnce(Entity) -> C,
    ) -> &mut Self;
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
        value.apply(self);
        self
    }

    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>
    {
        OnEventExt::new(self.reborrow())
    }

    fn update_on<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        triggers: impl ReactionTriggerBundle,
        reactor: impl FnOnce(Entity) -> C,
    ) -> &mut Self
    {
        let id = self.id();
        let callback = (reactor)(id);
        let syscommand = self.commands().spawn_system_command(callback);
        self.commands()
            .syscall((id, syscommand, triggers), register_update_on_reactor);
        self.commands().add(syscommand);

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

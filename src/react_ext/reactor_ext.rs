/*
container.load(path.e("button"), |button, path| {
    let button_id = button.id();
    button.insert_reactive(Counter::default())
        .on_pressed(move |mut c: Commands, mut counters: ReactiveMut<Counter>| {
            counters.get_mut(&mut c, button_id).map(Counter::increment);
        });

    button.load(path.e("text"), |text, _| {
        text.update_on(entity_mutation::<Counter>(button_id),
            |text_id| move |mut editor: TextEditor, counters: Reactive<Counter>| {
                let Some(counter) = counters.get(button_id) else { return };
                editor.write(text_id, |t| write!(t, "Count: {}", *counter));
            }
        );
    });
});

*/

use crate::*;

use std::marker::PhantomData;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_cobweb::prelude::*;
use sickle_ui::ui_builder::*;

//-------------------------------------------------------------------------------------------------------------------

fn register_update_on_reactor<Triggers: ReactionTriggerBundle>(
    In((entity, syscommand, triggers)): In<(Entity, SystemCommand, Triggers)>,
    mut c: Commands,
    loaded: Query<(), With<LoadedStyles>>
){
    // Detect styles loaded if appropriate.
    let revoke_token = if loaded.contains(entity)
    {
        let triggers = (triggers, entity_event::<StylesLoaded>(entity));
        c.react().with(triggers, syscommand, ReactorMode::Revokable).unwrap()
    }
    else
    {
        c.react().with(triggers, syscommand, ReactorMode::Revokable).unwrap()
    };

    //todo: more efficient cleanup mechanism
    cleanup_reactor_on_despawn(&mut c, entity, revoke_token);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper struct returned by [`on_event`](NodeReactEntityCommandsExt::on_event).
///
/// Call [`Self::r`] to add a reactor.
//todo: Use UiBuilder once reborrowing is implemented
pub struct OnEventExt<'a, T: Send + Sync + 'static>
{
    ec: EntityCommands<'a>,
    _p: PhantomData<T>,
}

impl<'a, T: Send + Sync + 'static> OnEventExt<'a, T>
{
    fn new(ec: EntityCommands<'a>) -> OnEventExt<'a, T>
    {
        Self{ ec, _p: PhantomData::default() }
    }

    /// Adds a reactor to an [`on_event`](NodeReactEntityCommandsExt::on_event) request.
    pub fn r<M>(
        mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'a>
    {
        let syscommand = self.ec.commands().spawn_system_command(callback);
        let id = self.ec.id();
        self.ec.commands().react().with(entity_event::<T>(id), syscommand, ReactorMode::Cleanup);

        self.ec
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering reactors for node entities.
pub trait NodeReactEntityCommandsExt
{
    /// Inserts a reactive component to the entity.
    ///
    /// The component can be accessed with the [`React<T>`] component, or with the
    /// [`Reactive`]/[`ReactiveMut`] system parameters.
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self;

    /// Registers an [`entity_event`] reactor for the current entity.
    ///
    /// Use [`OnEventExt::r`] to register the reactor.
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>;

    /// Updates an entity with a reactor system.
    ///
    /// The system runs:
    /// - Immediately after being registered.
    /// - Whenever the triggers fire.
    /// - When entities with loaded styles receive [`StylesLoaded`] events.
    fn update_on<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        triggers : impl ReactionTriggerBundle,
        reactor  : impl FnOnce(Entity) -> C
    ) -> &mut Self;
}

impl NodeReactEntityCommandsExt for UiBuilder<'_, '_, '_, Entity>
{
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self
    {
        let id = self.id();
        self.commands().react().insert(id, component);
        self
    }

    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>
    {
        OnEventExt::new(self.entity_commands())
    }

    fn update_on<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        triggers : impl ReactionTriggerBundle,
        reactor  : impl FnOnce(Entity) -> C
    ) -> &mut Self
    {
        let id = self.id();
        let callback = (reactor)(id);
        let syscommand = self.commands().spawn_system_command(callback);
        self.commands().syscall((id, syscommand, triggers), register_update_on_reactor);
        self.commands().add(syscommand);

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

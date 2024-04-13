/*
container.load_child(path.e("button"), |button, path| {
    let id = button.id();
    let counter = button.reactive_data(Counter::default()).id();
    button.on_event::<Pressed>().r(move |mut cmds: Commands, mut counters: ReactiveMut<Counter>| {
        counters.get_mut(&mut cmds, counter).map(Counter::increment);
    });

    button.load_child(path.e("text"), |text, _| {
        text.update_on(entity_mutation::<Counter>(counter),
            |id| move |mut editor: TextEditor, counters: Reactive<Counter>| {
                let Some(counter) = counters.get(counter) else { return };
                let _ = editor.write(id, |t| write!("Count: {}", *counter));
            }
        );
    });
});

*/

use crate::*;

use std::marker::PhantomData;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_cobweb::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn register_update_on_reactor<Triggers: ReactionTriggerBundle>(
    In((entity, syscommand, triggers)): In<(Entity, SystemCommand, Triggers)>,
    mut rc: ReactCommands,
    loaded: Query<(), With<LoadedStyles>>
){
    // Detect styles loaded if appropriate.
    let revoke_token = if loaded.contains(entity)
    {
        let triggers = (triggers, entity_event::<StylesLoaded>(entity));
        rc.with(triggers, syscommand, ReactorMode::Revokable).unwrap()
    } else {
        rc.with(triggers, syscommand, ReactorMode::Revokable).unwrap()
    };

    //todo: more efficient cleanup mechanism
    cleanup_reactor_on_despawn(&mut rc, entity, revoke_token);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper struct returned by [`on_event`](NodeReactEntityCommandsExt::on_event).
///
/// Call [`Self::r`] to add a reactor.
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
        //todo: register this reactor directly
        self.ec.commands().syscall((id, syscommand),
                |In((entity, syscommand)): In<(Entity, SystemCommand)>, mut rc: ReactCommands|
                {
                    // ReactorMode::Cleanup will clean up the reactor when `entity` is despawned (or if it was already
                    // despawned).
                    rc.with(entity_event::<T>(entity), syscommand, ReactorMode::Cleanup);
                }
            );

        self.ec
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering reactors for node entities.
pub trait NodeReactEntityCommandsExt
{
    /// Registers an [`entity_event`] reactor for the current entity.
    ///
    /// Use [`OnEventExt::r`] to register the reactor.
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<T>;

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
    ) -> EntityCommands<'_>;
}

impl NodeReactEntityCommandsExt for EntityCommands<'_>
{
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<T>
    {
        OnEventExt::new(self.reborrow())
    }

    fn update_on<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        triggers : impl ReactionTriggerBundle,
        reactor  : impl FnOnce(Entity) -> C
    ) -> EntityCommands<'_>
    {
        let id = self.id();
        let callback = (reactor)(id);
        let syscommand = self.commands().spawn_system_command(callback);
        self.commands().syscall((id, syscommand, triggers), register_update_on_reactor);
        self.commands().add(syscommand);

        self.reborrow()
    }
}

//-------------------------------------------------------------------------------------------------------------------

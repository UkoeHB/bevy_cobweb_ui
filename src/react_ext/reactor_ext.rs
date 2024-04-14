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

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::ui_builder::*;

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
pub struct OnEventExt<'w, 's, 'a, T: Send + Sync + 'static>
{
    eb: &'a mut UiBuilder<'w, 's, 'a, Entity>,
    _p: PhantomData<T>,
}

impl<'w, 's, 'a, T: Send + Sync + 'static> OnEventExt<'w, 's, 'a, T>
{
    fn new(eb: &'a mut UiBuilder<'w, 's, 'a, Entity>) -> OnEventExt<'w, 's, 'a, T>
    {
        Self{ eb, _p: PhantomData::default() }
    }

    /// Adds a reactor to an [`on_event`](NodeReactEntityCommandsExt::on_event) request.
    pub fn r<M>(
        self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> &'a mut UiBuilder<'w, 's, 'a, Entity>
    {
        let syscommand = self.eb.commands().spawn_system_command(callback);
        let id = self.eb.id();
        //todo: register this reactor directly
        self.eb.commands().syscall((id, syscommand),
                |In((entity, syscommand)): In<(Entity, SystemCommand)>, mut rc: ReactCommands|
                {
                    // ReactorMode::Cleanup will clean up the reactor when `entity` is despawned (or if it was already
                    // despawned).
                    rc.with(entity_event::<T>(entity), syscommand, ReactorMode::Cleanup);
                }
            );

        self.eb
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering reactors for node entities.
pub trait NodeReactEntityCommandsExt<'w, 's, 'a>
{
    /// Registers an [`entity_event`] reactor for the current entity.
    ///
    /// Use [`OnEventExt::r`] to register the reactor.
    fn on_event<T: Send + Sync + 'static>(&'a mut self) -> OnEventExt<'w, 's, 'a, T>;

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

impl<'w, 's, 'a> NodeReactEntityCommandsExt<'w, 's, 'a> for UiBuilder<'w, 's, 'a, Entity>
{
    fn on_event<T: Send + Sync + 'static>(&'a mut self) -> OnEventExt<'w, 's, 'a, T>
    {
        OnEventExt::new(self)
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

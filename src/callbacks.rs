//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

/// A system event sent to UI event reactors added to UI nodes with [`On::<E>::new()`].
pub struct UiEvent<E: Clone + Send + Sync + 'static>
{
    /// The event that occurred.
    pub event: E,
    /// The node reacting to an event.
    pub node: Entity,
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a reactor to keyboard inputs to a UI node.
pub struct OnEvent<E, T, M>
where
    E: Clone + Send + Sync + 'static,
    T: IntoSystem<(), (), M> + Send + Sync + 'static
{
    callback: T,
    phantom: PhantomData<(E, M)>
}

impl<E, T, M> OnEvent<E, T, M>
where
    E: Clone + Send + Sync + 'static,
    T: IntoSystem<(), (), M> + Send + Sync + 'static
{
    pub fn new(callback: T) -> Self
    {
        Self{ callback, phantom: PhantomData::default() }
    }
}

impl<E, T, M> UiInstruction for OnEvent<E, T, M>
where
    E: Clone + Send + Sync + 'static,
    T: IntoSystem<(), (), M> + Send + Sync + 'static
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        let reactor = rc.commands().spawn_system_command(self.callback);

        let token = rc.on(broadcast::<E>(),
            move
            |
                mut commands : Commands,
                event        : BroadcastEvent<E>
            |
            {
                let Some(event) = event.read() else { return; };
                commands.send_system_event(reactor, UiEvent{ event: event.clone(), node });
            }
        );
        rc.on(despawn(node),
            move |mut rc: ReactCommands|
            {
                rc.revoke(token.clone());
                rc.commands().add(move |world: &mut World| { world.despawn(*reactor); });
            }
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A helper struct for creating [`OnEvent`] UI instructions.
///
/// Example: `On::<KeyboardInput>::new(my_callback_system)`.
pub struct On<E: Clone + Send + Sync + 'static>(PhantomData<E>);

impl<E: Clone + Send + Sync + 'static> On<E>
{
    pub fn new<T, M>(callback: T) -> OnEvent<E, T, M>
    where
        T: IntoSystem<(), (), M> + Send + Sync + 'static
    {
        OnEvent{ callback, phantom: PhantomData::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

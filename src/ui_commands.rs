//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Entity event sent by [`UiCommands::build`] to a node after its instructions are built.
///
/// [`UiInstruction`] and [`CobwebStyle`] implementations should listen to `NodeBuilt` events if they have reactors
/// that must run in order to initialize the node.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct NodeBuilt;

//-------------------------------------------------------------------------------------------------------------------

/// Wrapper around `EntityCommands` for UI nodes being constructed.
///
/// When this wrapper is dropped, a [`NodeBuilt`] entity event will be sent to the node. We wait to send this event
/// until the wrapper is dropped so that entity modifications using this wrapper (e.g. component insertions) will be
/// visible to reactors triggered by the event.
#[derive(Deref, DerefMut)]
pub struct UiEntityCommands<'a>(EntityCommands<'a>);

impl<'a> Drop for UiEntityCommands<'a>
{
    fn drop(&mut self)
    {
        let node = self.id();
        self.commands().add(move |world: &mut World| world.entity_event(node, NodeBuilt));
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemParam)]
pub struct UiCommands<'w, 's>
{
    pub rc: ReactCommands<'w, 's>,
    pub tracker: ResMut<'w, DirtyNodeTracker>,
}

impl<'w, 's> UiCommands<'w, 's>
{
    pub fn build(&mut self, instructions: impl UiInstructionBundle) -> UiEntityCommands
    {
        let node = self.rc.commands().spawn(CobwebNode).id();
        self.tracker.insert(node);
        instructions.build(&mut self.rc, node);
        UiEntityCommands(self.rc.commands().entity(node))
    }
}

//-------------------------------------------------------------------------------------------------------------------

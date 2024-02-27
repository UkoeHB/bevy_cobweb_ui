//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemParam)]
pub struct UiCommands<'w, 's>
{
    pub rcommands: ReactCommands<'w, 's>,
}

impl<'w, 's> UiCommands<'w, 's>
{
    pub fn build(&mut self, instructions: impl UiInstructionBundle) -> EntityCommands
    {
        let node = self.rcommands.commands().spawn_empty().id();
        instructions.build(&mut self.rcommands, node);
        self.rcommands.commands().entity(node)
    }
}

//-------------------------------------------------------------------------------------------------------------------

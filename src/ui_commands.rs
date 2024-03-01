//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemParam)]
pub struct UiCommands<'w, 's>
{
    pub rcommands: ReactCommands<'w, 's>,
    finishers: ResMut<'w, UiInstructionFinishersCrate>,
}

impl<'w, 's> UiCommands<'w, 's>
{
    pub fn build(&mut self, instructions: impl UiInstructionBundle) -> EntityCommands
    {
        let finishers = &mut self.finishers.inner;
        finishers.buffer.clear();

        let node = self.rcommands.commands().spawn_empty().id();
        instructions.build(&mut self.rcommands, node, finishers);

        for finisher in finishers.buffer.drain(..)
        {
            self.rcommands.commands().add(finisher);
        }

        self.rcommands.commands().entity(node)
    }
}

//-------------------------------------------------------------------------------------------------------------------

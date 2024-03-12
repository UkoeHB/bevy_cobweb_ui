//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy::utils::all_tuples;
use bevy_cobweb::prelude::ReactCommands;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for applying a UI instruction to a node entity.
pub trait UiInstruction
{
    fn apply(self, rcommands: &mut ReactCommands, node_entity: Entity);
}

impl<I: UiInstruction> UiInstructionBundle for I
{
    fn len(&self) -> usize { 1 }

    fn build(self, rcommands: &mut ReactCommands, node_entity: Entity)
    {
        self.apply(rcommands, node_entity);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for building UI nodes with [`UiCommands`](crate::UiCommands).
///
/// All members of a bundle must implement [`UiInstructionBundle`]. You should implement [`UiInstruction`]
/// on the root members of a bundle.
pub trait UiInstructionBundle
{
    /// Gets the number of instructions in the bundle.
    fn len(&self) -> usize;

    /// Builds the bundle's instructions.
    fn build(self, rcommands: &mut ReactCommands, node_entity: Entity);
}

//-------------------------------------------------------------------------------------------------------------------

// Implements [`UiInstructionBundle`] for tuples of instructions.
macro_rules! tuple_impl
{
    ($($name: ident),*) =>
    {
        impl<$($name: UiInstructionBundle),*> UiInstructionBundle for ($($name,)*)
        {
            #[allow(unused_variables, unused_mut)]
            #[inline(always)]
            fn len(&self) -> usize
            {
                let mut len = 0;
                #[allow(non_snake_case)]
                let ($($name,)*) = self;
                $(
                    len += $name.len();
                )*

                len
            }

            #[allow(unused_variables, unused_mut)]
            #[inline(always)]
            fn build(self, rcommands: &mut ReactCommands, node_entity: Entity)
            {
                #[allow(non_snake_case)]
                let ($(mut $name,)*) = self;
                $(
                    $name.build(rcommands, node_entity);
                )*
            }
        }
    }
}

all_tuples!(tuple_impl, 0, 15, B);

//-------------------------------------------------------------------------------------------------------------------

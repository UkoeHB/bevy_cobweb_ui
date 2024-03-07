//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct PrimitivesPlugin;

impl Plugin for PrimitivesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BlockPrimitivePlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

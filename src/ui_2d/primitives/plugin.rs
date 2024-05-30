use crate::*;

use bevy::prelude::*;


//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct PrimitivesPlugin;

impl Plugin for PrimitivesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BasicImagePrimitivePlugin)
            .add_plugins(BlockPrimitivePlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

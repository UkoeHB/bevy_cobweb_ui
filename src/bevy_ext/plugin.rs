use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BevyExtPlugin;

impl Plugin for BevyExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BevySpriteExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BevyExtPlugin;

impl Plugin for BevyExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(CursorPlugin)
            .add_plugins(FontExtPlugin)
            .add_plugins(TextureAtlasExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

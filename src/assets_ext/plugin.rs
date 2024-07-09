use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct AssetsExtPlugin;

impl Plugin for AssetsExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Vec<String>>()
            .add_plugins(ImageLoadPlugin)
            .add_plugins(FontLoadPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

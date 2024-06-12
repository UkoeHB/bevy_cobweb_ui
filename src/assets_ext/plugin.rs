use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct AssetsExtPlugin;

impl Plugin for AssetsExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(ImageLoadPlugin).add_plugins(FontLoadPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

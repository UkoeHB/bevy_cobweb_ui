use bevy::prelude::*;

use super::fonts::BuiltInFontsPlugin;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinAssetsPlugin;

impl Plugin for BuiltinAssetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BuiltInFontsPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

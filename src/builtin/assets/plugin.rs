use bevy::prelude::*;
pub(crate) use fonts::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinAssetsPlugin;

impl Plugin for BuiltinAssetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BuiltinFontsPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;

#[cfg(feature = "firasans")]
use super::fonts::BuiltInFontsPlugin;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinAssetsPlugin;

impl Plugin for BuiltinAssetsPlugin
{
    fn build(&self, _app: &mut App)
    {
        #[cfg(feature = "firasans")]
        _app.add_plugins(BuiltInFontsPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

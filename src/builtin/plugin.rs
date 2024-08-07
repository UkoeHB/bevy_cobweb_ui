use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinPlugin;

impl Plugin for BuiltinPlugin
{
    fn build(&self, app: &mut App)
    {
        #[cfg(feature = "assets")]
        app.add_plugins(crate::builtin::assets::BuiltinAssetsPlugin);

        #[cfg(feature = "colors")]
        app.add_plugins(crate::builtin::colors::BuiltinColorsPlugin);

        #[cfg(feature = "widgets")]
        app.add_plugins(crate::builtin::widgets::BuiltinWidgetsPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

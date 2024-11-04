use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinPlugin;

impl Plugin for BuiltinPlugin
{
    fn build(&self, _app: &mut App)
    {
        _app.add_plugins(crate::builtin::assets::BuiltinAssetsPlugin);

        #[cfg(feature = "colors")]
        _app.add_plugins(crate::builtin::colors::BuiltinColorsPlugin);

        #[cfg(feature = "widgets")]
        _app.add_plugins(crate::builtin::widgets::BuiltinWidgetsPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

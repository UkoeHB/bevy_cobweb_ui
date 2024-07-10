use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::SickleUiPlugin;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that sets up `bevy_cobweb_ui` in an app.
///
/// Panics if [`bevy_cobweb::prelude::ReactPlugin`] or [`sickle_ui::prelude::SickleUiPlugin`] are missing.
pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        if !app.is_plugin_added::<ReactPlugin>() {
            panic!("failed building CobwebUiPlugin, bevy_cobweb::prelude::ReactPlugin is missing");
        }
        if !app.is_plugin_added::<SickleUiPlugin>() {
            panic!("failed building CobwebUiPlugin, sickle_ui::prelude::SickleUiPlugin is missing");
        }

        app.add_plugins(BuiltinAssetsPlugin)
            .add_plugins(ReactExtPlugin)
            .add_plugins(BevyExtPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(LocalizationPlugin)
            .add_plugins(SickleExtPlugin)
            .add_plugins(AssetsExtPlugin)
            .add_plugins(CobwebBevyUiPlugin)
            ;

        #[cfg(feature = "widgets")]
        {
            app.add_plugins(crate::widgets::CobwebWidgetsPlugin)
            ;
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::SickleUiPlugin;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that sets up `bevy_cobweb_ui` in an app.
///
/// Panics if [`bevy_cobweb::prelude::ReactPlugin`] or [`crate::sickle::SickleUiPlugin`] are missing.
pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        if !app.is_plugin_added::<ReactPlugin>() {
            app.add_plugins(ReactPlugin);
        }
        if !app.is_plugin_added::<SickleUiPlugin>() {
            app.add_plugins(SickleUiPlugin);
        }

        app
            //todo: remove in bevy v0.15; see https://github.com/bevyengine/bevy/issues/14969
            .register_type_data::<SmolStr, ReflectDeserialize>()
            .add_plugins(crate::builtin::BuiltinPlugin)
            .add_plugins(ReactExtPlugin)
            .add_plugins(BevyExtPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(LocalizationPlugin)
            .add_plugins(SickleExtPlugin)
            .add_plugins(AssetsExtPlugin)
            .add_plugins(CobwebBevyUiPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

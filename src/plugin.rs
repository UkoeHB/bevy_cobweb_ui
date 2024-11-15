use std::borrow::Cow;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that sets up `bevy_cobweb_ui` in an app.
pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        if !app.is_plugin_added::<ReactPlugin>() {
            app.add_plugins(ReactPlugin);
        }

        app.register_type_data::<Cow<str>, ReflectDeserialize>()
            .add_plugins(crate::builtin::BuiltinPlugin)
            .add_plugins(ReactExtPlugin)
            .add_plugins(BevyExtPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(LocalizationPlugin)
            .add_plugins(SickleExtPlugin)
            .add_plugins(AssetsExtPlugin)
            .add_plugins(CobwebBevyUiPlugin);

        #[cfg(feature = "editor")]
        app.add_plugins(crate::editor::CobEditorPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

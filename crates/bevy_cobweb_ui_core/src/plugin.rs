use bevy::prelude::*;

use crate::prelude::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct BevyCobwebUiCorePlugin;

impl Plugin for BevyCobwebUiCorePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(LoadProgressPlugin)
            .add_plugins(LoadExtPlugin)
            .add_plugins(CobAssetLoaderPlugin)
            .add_plugins(AppLoadExtPlugin)
            .add_plugins(CobAssetCachePlugin)
            .add_plugins(SceneBuilderPlugin) // Must be after the COB cache plugin.
            ;

        #[cfg(feature = "editor")]
        app.add_plugins(crate::editor::CobEditorPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

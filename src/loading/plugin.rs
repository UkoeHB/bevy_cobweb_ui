use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadingPlugin;

impl Plugin for LoadingPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(LoadProgressPlugin)
            .add_plugins(LoadExtPlugin)
            .add_plugins(CobAssetLoaderPlugin)
            .add_plugins(AppLoadExtPlugin)
            .add_plugins(CobAssetCachePlugin)
            .add_plugins(SceneLoaderPlugin) // Must be after the COB cache plugin.
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

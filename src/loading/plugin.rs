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
            .add_plugins(CobwebAssetLoaderPlugin)
            .add_plugins(CobwebAssetCachePlugin)
            .add_plugins(SceneLoaderPlugin) // Must be after the CAF cache plugin.
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

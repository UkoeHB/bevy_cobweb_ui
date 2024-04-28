use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadingPlugin;

impl Plugin for LoadingPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(LoaderPlugin)
            .add_plugins(LoadableSheetAssetLoaderPlugin)
            .add_plugins(LoadableSheetPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

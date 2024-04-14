use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadingPlugin;

impl Plugin for LoadingPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(StyleLoaderPlugin)
            .add_plugins(StyleSheetAssetLoaderPlugin)
            .add_plugins(StyleSheetPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

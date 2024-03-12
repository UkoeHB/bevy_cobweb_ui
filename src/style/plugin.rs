//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct StylePlugin;

impl Plugin for StylePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(StyleLoaderPlugin)
            .add_plugins(StyleAssetLoaderPlugin)
            .add_plugins(StyleSheetPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

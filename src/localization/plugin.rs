use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LocalizationPlugin;

impl Plugin for LocalizationPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(FtlBundleAssetLoaderPlugin)
            .add_plugins(LocalePlugin)
            .add_plugins(LocalizationManifestPlugin)
            .add_plugins(LocalizationSetPlugin)
            .add_plugins(LocalizedTextPlugin)
            .add_plugins(RelocalizeTrackerPlugin)
            .add_plugins(TextLocalizerPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

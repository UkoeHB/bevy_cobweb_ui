use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_sickle_ui_default_fonts(mut fonts: ResMut<FontMap>, asset_server: Res<AssetServer>)
{
    fonts.insert("embedded://sickle_ui/fonts/FiraSans-Bold.ttf", &asset_server);
    fonts.insert("embedded://sickle_ui/fonts/FiraSans-BoldItalic.ttf", &asset_server);
    fonts.insert("embedded://sickle_ui/fonts/FiraSans-Italic.ttf", &asset_server);
    fonts.insert("embedded://sickle_ui/fonts/FiraSans-Medium.ttf", &asset_server);
    fonts.insert("embedded://sickle_ui/fonts/FiraSans-MediumItalic.ttf", &asset_server);
    fonts.insert("embedded://sickle_ui/fonts/FiraSans-Regular.ttf", &asset_server);
    fonts.insert("embedded://sickle_ui/fonts/FiraSansCondensed-Bold.ttf", &asset_server);
    fonts.insert(
        "embedded://sickle_ui/fonts/FiraSansCondensed-BoldItalic.ttf",
        &asset_server,
    );
    fonts.insert("embedded://sickle_ui/fonts/FiraSansCondensed-Italic.ttf", &asset_server);
    fonts.insert(
        "embedded://sickle_ui/fonts/FiraSansCondensed-Regular.ttf",
        &asset_server,
    );
    fonts.insert("embedded://sickle_ui/fonts/MaterialIcons-Regular.ttf", &asset_server);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleUiDefaultAssetsPlugin;

impl Plugin for SickleUiDefaultAssetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Startup, load_sickle_ui_default_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

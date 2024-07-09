use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::FontMap;

//-------------------------------------------------------------------------------------------------------------------

fn load_builtin_fonts(mut fonts: ResMut<FontMap>, asset_server: Res<AssetServer>)
{
    fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-Bold.ttf", &asset_server);
    fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-BoldItalic.ttf", &asset_server);
    fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-ExtraBold.ttf", &asset_server);
    fonts.insert(
        "embedded://bevy_cobweb_ui/fonts/OpenSans-ExtraBoldItalic.ttf",
        &asset_server,
    );
    fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-Italic.ttf", &asset_server);
    fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-Light.ttf", &asset_server);
    fonts.insert(
        "embedded://bevy_cobweb_ui/fonts/OpenSans-LightItalic.ttf",
        &asset_server,
    );
    fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-Regular.ttf", &asset_server);
    fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-Semibold.ttf", &asset_server);
    fonts.insert(
        "embedded://bevy_cobweb_ui/fonts/OpenSans-SemiboldItalic.ttf",
        &asset_server,
    );
}

pub(crate) struct BuiltinFontsPlugin;

impl Plugin for BuiltinFontsPlugin
{
    fn build(&self, app: &mut App)
    {
        embedded_asset!(app, "src/assets/", "OpenSans-Bold.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-BoldItalic.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-ExtraBold.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-ExtraBoldItalic.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-Italic.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-Light.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-LightItalic.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-Regular.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-Semibold.ttf");
        embedded_asset!(app, "src/assets/", "OpenSans-SemiboldItalic.ttf");

        app.add_systems(Startup, load_builtin_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

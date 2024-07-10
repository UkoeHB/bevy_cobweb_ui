//use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::FontMap;

//-------------------------------------------------------------------------------------------------------------------

fn load_builtin_fonts(mut _fonts: ResMut<FontMap>, _asset_server: Res<AssetServer>)
{
    // note: removed OpenSans because it had a 1-pixel offset
    //fonts.insert("embedded://bevy_cobweb_ui/fonts/OpenSans-Bold.ttf", &asset_server);
}

pub(crate) struct BuiltinFontsPlugin;

impl Plugin for BuiltinFontsPlugin
{
    fn build(&self, app: &mut App)
    {
        //embedded_asset!(app, "src/assets/", "OpenSans-Bold.ttf");

        app.add_systems(Startup, load_builtin_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

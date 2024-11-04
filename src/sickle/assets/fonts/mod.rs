use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub(crate) struct BuiltInFontsPlugin;

impl Plugin for BuiltInFontsPlugin
{
    fn build(&self, app: &mut App)
    {
        embedded_asset!(app, "src/sickle/assets/", "FiraSans-Bold.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSans-BoldItalic.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSans-Italic.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSans-Medium.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSans-MediumItalic.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSans-Regular.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSansCondensed-Bold.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSansCondensed-BoldItalic.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSansCondensed-Italic.ttf");
        embedded_asset!(app, "src/sickle/assets/", "FiraSansCondensed-Regular.ttf");
        embedded_asset!(app, "src/sickle/assets/", "MaterialIcons-Regular.ttf");
    }
}

use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_sickle_ui_default_fonts(mut c: Commands)
{
    c.add(RegisterFontFamilies(vec![
        RegisterFontFamily {
            family: "Fira Sans".into(),
            fonts: vec![
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSans-Bold.ttf".into(),
                    width: FontWidth::Normal,
                    style: FontStyle::Normal,
                    weight: FontWeight::Bold,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSans-BoldItalic.ttf".into(),
                    width: FontWidth::Normal,
                    style: FontStyle::Italic,
                    weight: FontWeight::Bold,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSans-Italic.ttf".into(),
                    width: FontWidth::Normal,
                    style: FontStyle::Italic,
                    weight: FontWeight::Normal,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSans-Medium.ttf".into(),
                    width: FontWidth::Normal,
                    style: FontStyle::Normal,
                    weight: FontWeight::Medium,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSans-MediumItalic.ttf".into(),
                    width: FontWidth::Normal,
                    style: FontStyle::Italic,
                    weight: FontWeight::Medium,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSans-Regular.ttf".into(),
                    width: FontWidth::Normal,
                    style: FontStyle::Normal,
                    weight: FontWeight::Normal,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSansCondensed-Bold.ttf".into(),
                    width: FontWidth::Condensed,
                    style: FontStyle::Normal,
                    weight: FontWeight::Bold,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSansCondensed-BoldItalic.ttf".into(),
                    width: FontWidth::Condensed,
                    style: FontStyle::Italic,
                    weight: FontWeight::Bold,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSansCondensed-Italic.ttf".into(),
                    width: FontWidth::Condensed,
                    style: FontStyle::Italic,
                    weight: FontWeight::Normal,
                },
                FontVariant {
                    path: "embedded://bevy_cobweb_ui/fonts/FiraSansCondensed-Regular.ttf".into(),
                    width: FontWidth::Condensed,
                    style: FontStyle::Normal,
                    weight: FontWeight::Bold,
                },
            ],
        },
    ]));
    // Now actually load the registered font family.
    c.add(LoadFonts(vec!["Fira Sans".into()]));
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltInFontsPlugin;

impl Plugin for BuiltInFontsPlugin
{
    fn build(&self, app: &mut App)
    {
        embedded_asset!(app, "src/builtin/assets", "FiraSans-Bold.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSans-BoldItalic.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSans-Italic.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSans-Medium.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSans-MediumItalic.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSans-Regular.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSansCondensed-Bold.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSansCondensed-BoldItalic.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSansCondensed-Italic.ttf");
        embedded_asset!(app, "src/builtin/assets", "FiraSansCondensed-Regular.ttf");
        app.add_systems(Startup, load_sickle_ui_default_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

//! Adds built-in `sickle_ui` assets via [`embedding`](https://docs.rs/bevy/latest/bevy/asset/macro.embedded_asset.html).
//!
//! Access these assets with `asset_server.load("embedded://bevy_cobweb_ui/icons/exit.png")`.

mod fonts;

use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::prelude::*;
use fonts::BuiltInFontsPlugin;

pub(crate) struct BuiltInAssetsPlugin;

impl Plugin for BuiltInAssetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<EmbeddedAssetRegistry>();
        app.add_plugins(BuiltInFontsPlugin);
    }
}

//! Adds built-in `bevy_cobweb_ui` assets via [`embedding`](https://docs.rs/bevy/latest/bevy/asset/macro.embedded_asset.html).
//!
//! Access these assets with `asset_server.load("embedded://bevy_cobweb_ui/fonts/OpenSans-Regular.ttf")`.

mod fonts;
mod plugin;

pub(crate) use fonts::*;
pub(crate) use plugin::*;

#![doc = include_str!("ASSETS_EXT.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

mod audio;
mod fonts;
mod images;
mod plugin;
mod texture_atlases;

pub use audio::*;
pub use fonts::*;
pub use images::*;
pub(crate) use plugin::*;
pub use texture_atlases::*;

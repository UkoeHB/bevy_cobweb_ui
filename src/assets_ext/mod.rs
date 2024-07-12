#![doc = include_str!("ASSETS_EXT.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

mod fonts;
mod images;
mod plugin;

pub use fonts::*;
pub use images::*;
pub(crate) use plugin::*;

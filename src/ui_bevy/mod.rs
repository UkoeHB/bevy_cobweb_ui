#![doc = include_str!("UI_BEVY.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

mod plugin;
mod ui_ext;

pub(crate) use plugin::*;
pub use ui_ext::*;

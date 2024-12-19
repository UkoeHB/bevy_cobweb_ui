//! Built-in widgets
//!
//! If the `widgets` feature is enabled, then built-in widgets will be automatically loaded and ready to use.

pub mod checkbox;
pub mod radio_button;
pub mod scroll;
pub mod slider;
//pub mod tooltip;

mod plugin;
pub(crate) use plugin::*;

//! Built-in widgets
//!
//! If the `widgets` feature is enabled, then built-in widgets will be automatically loaded and ready to use.
//!
//! **Disclaimer**: Default widget styling should not be considered stable at this time.
//!
//! Currently implemented:
//! - [radio_button]
//! - [slider]

pub mod radio_button;
pub mod slider;

mod plugin;
pub(crate) use plugin::*;

//-------------------------------------------------------------------------------------------------------------------

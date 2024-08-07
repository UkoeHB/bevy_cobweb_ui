//! Built-in widgets
//!
//! If the `widgets` feature is enabled, then built-in widgets will be automatically loaded and ready to use.
//!
//! **Disclaimer**: Default widget styling should not be considered stable at this time.
//!
//! Currently implemented:
//! - [radio_button]: Includes [`RadioButtonBuilder`](radio_button::RadioButtonBuilder) for making radio buttons,
//!   and [`RadioButtonManager`](radio_button::RadioButtonManager) for coordinating button selection.

pub mod radio_button;

mod plugin;
pub(crate) use plugin::*;

//-------------------------------------------------------------------------------------------------------------------

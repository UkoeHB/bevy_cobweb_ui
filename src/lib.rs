//documentation
#![doc = include_str!("../README.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

//module tree
mod app_events;
mod callbacks;
mod layout;
//mod layout_helpers;
mod parents;
mod plugin;
mod primitives;
mod ui_commands;
mod ui_instruction;
mod ui_instruction_utils;

//API exports
pub use crate::app_events::*;
pub use crate::callbacks::*;
pub use crate::layout::*;
//pub use crate::layout_helpers::*;
pub use crate::parents::*;
pub use crate::plugin::*;
pub use crate::primitives::*;
pub use crate::ui_commands::*;
pub use crate::ui_instruction::*;
pub use crate::ui_instruction_utils::*;

//pub use bevy_cobweb_ui_derive::*;

pub mod prelude
{
    pub use crate::*;
}

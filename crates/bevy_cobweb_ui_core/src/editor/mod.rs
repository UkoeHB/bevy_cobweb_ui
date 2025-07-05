#![doc = include_str!("EDITOR.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui_core;

mod death_signal;
mod editor;
mod editor_commands;
mod editor_events;
//mod editor_stack;
mod hash_registry;
mod plugin;
mod utils;
mod widget_interop;
mod widget_registry;

pub use death_signal::*;
pub use editor::*;
pub use editor_commands::*;
pub use editor_events::*;
//pub(self) use editor_stack::*;
pub use hash_registry::*;
pub(crate) use plugin::*;
pub use utils::*;
pub use widget_interop::*;
pub use widget_registry::*;

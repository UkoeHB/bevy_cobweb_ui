#![doc = include_str!("EDITOR.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

mod build;
mod death_signal;
mod editor;
mod editor_commands;
mod editor_events;
//mod editor_stack;
mod hash_registry;
mod plugin;
mod template;
mod utils;
mod widget_interop;
mod widget_registry;

pub(self) use build::*;
pub(self) use death_signal::*;
pub(crate) use editor::*;
pub use editor_commands::*;
pub use editor_events::*;
//pub(self) use editor_stack::*;
pub(crate) use hash_registry::*;
pub(crate) use plugin::*;
pub(self) use template::*;
pub(self) use utils::*;
pub use widget_interop::*;
pub use widget_registry::*;

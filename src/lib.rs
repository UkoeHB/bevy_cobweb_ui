#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(rustdoc::redundant_explicit_links)]
#![doc = include_str!("../README.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

pub mod assets_ext;
pub mod bevy_ext;
pub mod builtin;
pub mod loading;
pub mod localization;
mod plugin;
pub mod react_ext;
pub mod sickle_ext;
pub mod tools;
pub mod ui_bevy;

pub mod sickle;

pub mod prelude {
    pub use bevy_cobweb_ui_derive::*;

    pub use crate::assets_ext::*;
    pub use crate::bevy_ext::*;
    pub use crate::loading::*;
    pub use crate::localization::*;
    pub use crate::plugin::*;
    pub use crate::react_ext::*;
    pub use crate::sickle::*;
    pub use crate::sickle_ext::*;
    pub use crate::tools::*;
    pub use crate::ui_bevy::*;
}

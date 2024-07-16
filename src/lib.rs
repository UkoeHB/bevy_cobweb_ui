#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

mod assets;
pub mod assets_ext;
pub mod loading;
pub mod localization;
mod plugin;
pub mod react_ext;
pub mod sickle_ext;
pub mod tools;
pub mod ui_bevy;

#[cfg(feature = "widgets")]
pub mod widgets;

pub mod sickle
{
    // Re-export sickle_ui so the dependency doesn't need to be tracked by users of this crate.
    pub use sickle_ui::*;
}

pub mod prelude
{
    pub use bevy_cobweb_ui_derive::*;

    pub(crate) use crate::assets::*;
    pub use crate::assets_ext::*;
    pub use crate::loading::*;
    pub use crate::localization::*;
    pub use crate::plugin::*;
    pub use crate::react_ext::*;
    pub use crate::sickle_ext::*;
    pub use crate::tools::*;
    pub use crate::ui_bevy::*;
}

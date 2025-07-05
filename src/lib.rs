#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(rustdoc::redundant_explicit_links)]
#![doc = include_str!("../README.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

pub mod assets_ext;
pub mod bevy_ext;
pub mod builtin;
pub mod localization;
mod plugin;
pub mod react_ext;
pub mod sickle_ext;
pub mod tools;
pub mod ui_bevy;

#[cfg(feature = "editor")]
pub mod editor;

pub mod cob
{
    pub use cobweb_asset_format::prelude::*;
}

pub mod sickle
{
    pub use cob_sickle_macros::*;
    pub use cob_sickle_math::*;
    pub use cob_sickle_ui_scaffold::*;
}

pub mod prelude
{
    pub use bevy_cobweb::prelude::{CobwebResult, DropErr, OptionToNoneErr, WarnErr, DONE, OK};
    pub use bevy_cobweb_ui_core::prelude::*;
    pub use bevy_cobweb_ui_core::ui::*;
    pub use bevy_cobweb_ui_derive::*;
    pub use cob_sickle_ui_scaffold::{UiBuilder, UiBuilderExt};

    pub use crate::assets_ext::*;
    pub use crate::bevy_ext::*;
    pub use crate::localization::*;
    pub use crate::plugin::*;
    pub use crate::react_ext::*;
    pub use crate::sickle_ext::*;
    pub use crate::tools::*;
    pub use crate::ui_bevy::*;
}

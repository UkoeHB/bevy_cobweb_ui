#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(rustdoc::redundant_explicit_links)]
#![doc = include_str!("../README.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui_core;

mod app_load_ext;
mod cache;
mod cob_asset_loader;
#[cfg(feature = "editor")]
pub mod editor;
mod extract;
mod load_ext;
mod load_progress;
mod loadable;
mod plugin;
mod scene;
#[cfg(feature = "ui")]
pub mod ui;

pub(crate) use cob_asset_loader::*;
pub(crate) use extract::*;

pub mod cob
{
    pub use cobweb_asset_format::prelude::*;
}

pub mod prelude
{
    pub use cobweb_asset_format::prelude::{SceneFile, ScenePath, SceneRef};

    pub use crate::app_load_ext::*;
    pub use crate::cache::*;
    pub use crate::load_ext::*;
    pub use crate::load_progress::*;
    pub use crate::loadable::*;
    pub use crate::plugin::*;
    pub use crate::scene::*;
}

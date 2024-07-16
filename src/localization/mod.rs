#![doc = include_str!("LOCALIZATION.md")]
#[allow(unused_imports)]
use crate as bevy_cobweb_ui;

mod ftl_bundle;
mod locale;
mod localization_manifest;
mod localization_set;
mod localized_text;
mod plugin;
mod relocalize_tracker;
mod text_localizer;

pub(crate) use ftl_bundle::*;
pub use locale::*;
pub use localization_manifest::*;
pub use localization_set::*;
pub use localized_text::*;
pub(crate) use plugin::*;
pub use relocalize_tracker::*;
pub use text_localizer::*;

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(rustdoc::redundant_explicit_links)]
#![doc = include_str!("../README.md")]
#[allow(unused_imports)]
use crate as cobweb_asset_format;

#[cfg(feature = "full")]
mod cob;
mod data;
mod parsing;
mod raw_serializer;
#[cfg(feature = "full")]
mod resolver;

pub mod prelude
{
    #[cfg(feature = "full")]
    pub use crate::cob::*;
    pub use crate::data::*;
    pub use crate::parsing::*;
    pub use crate::raw_serializer::*;
    #[cfg(feature = "full")]
    pub use crate::resolver::*;
}

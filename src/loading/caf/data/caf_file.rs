use std::borrow::Borrow;
use std::sync::Arc;

use bevy::asset::AssetPath;
use bevy::prelude::Deref;
use nom::bytes::complete::{tag, take_until};
use nom::sequence::delimited;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a cobweb asset file in the `asset` directory.
///
/// Cobweb asset files use the `.caf` extension. If your original path includes an asset source, then
/// the asset source must be included in the name (e.g. `embedded://scene.caf` -> `scene.caf`).
///
/// Example: `ui/home.caf` for a `home` cobweb asset in `assets/ui`.
#[derive(Debug, Clone, Deref, Eq, PartialEq, Hash)]
pub struct CafFile(Arc<str>);

impl CafFile
{
    /// Tries to create a new CAF file reference.
    ///
    /// Fails if the file doesn't end with `.caf`.
    pub fn try_new(file: impl AsRef<str>) -> Option<Self>
    {
        let file = file.as_ref();
        if !file.ends_with(".caf") {
            return None;
        }
        Some(Self(Arc::from(file)))
    }

    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("\"".as_bytes())?;
        writer.write_bytes(self.as_bytes())?;
        writer.write_bytes("\"".as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (remaining, path) = delimited(tag("\""), take_until("\""), tag("\"")).parse(content)?;

        // Validate
        if let Err(err) = AssetPath::try_parse(*path.fragment()) {
            tracing::warn!("failed parsing CAF file path at {}; path is invalid {:?}",
                get_location(content).as_str(), err);
            return Err(span_verify_error(content));
        }
        if !path.ends_with(".caf") {
            tracing::warn!("failed parsing CAF file path at {}; file does not end with '.caf' extension",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        Ok((Self(Arc::from(*path.fragment())), remaining))
    }

    pub fn as_str(&self) -> &str
    {
        &self.0
    }
}

impl Default for CafFile
{
    fn default() -> Self
    {
        Self(Arc::from(""))
    }
}

impl Borrow<str> for CafFile
{
    fn borrow(&self) -> &str
    {
        &*self
    }
}

//-------------------------------------------------------------------------------------------------------------------

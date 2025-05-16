use std::borrow::Borrow;
use std::sync::Arc;

use nom::bytes::complete::{tag, take_until};
use nom::sequence::delimited;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a cobweb asset file in the `asset` directory.
///
/// Cobweb asset files use the `.cob` extension. If your original path includes an asset source, then
/// the asset source must be included in the name (e.g. `embedded://scene.cob` -> `scene.cob`).
///
/// Example: `ui/home.cob` for a `home` cobweb asset in `assets/ui`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CobFile(Arc<str>);

impl CobFile
{
    /// Tries to create a new COB file reference.
    ///
    /// Fails if the file doesn't end with `.cob` or `.cobweb`.
    pub fn try_new(file: impl AsRef<str>) -> Option<Self>
    {
        let file = file.as_ref();
        if !file.ends_with(".cob") && !file.ends_with(".cobweb") {
            return None;
        }
        if file.find("\\").is_some() {
            let file = file.replace("\\", "/");
            Some(Self(Arc::from(file.as_str())))
        } else {
            Some(Self(Arc::from(file)))
        }
    }

    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("\"".as_bytes())?;
        writer.write_bytes(self.0.as_bytes())?;
        writer.write_bytes("\"".as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (remaining, path) = delimited(tag("\""), take_until("\""), tag("\"")).parse(content)?;

        // Validate
        if !path.ends_with(".cob") && !path.ends_with(".cobweb") {
            tracing::warn!("failed parsing COB file path at {}; file does not end with '.cob' or '.cobweb' extension",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        }

        Ok((Self(Arc::from(*path.fragment())), remaining))
    }

    pub fn as_str(&self) -> &str
    {
        &self.0
    }

    pub fn get(&self) -> &Arc<str>
    {
        &self.0
    }
}

impl Default for CobFile
{
    fn default() -> Self
    {
        Self(Arc::from(""))
    }
}

impl Borrow<str> for CobFile
{
    fn borrow(&self) -> &str
    {
        self.as_str()
    }
}

//-------------------------------------------------------------------------------------------------------------------

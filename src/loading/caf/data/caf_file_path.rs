use std::sync::Arc;

use bevy::asset::AssetPath;
use bevy::prelude::Deref;
use nom::bytes::complete::{tag, take_until};
use nom::sequence::delimited;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafFilePath(pub Arc<str>);

impl CafFilePath
{
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
}

impl Default for CafFilePath
{
    fn default() -> Self
    {
        Self(Arc::from(""))
    }
}

//-------------------------------------------------------------------------------------------------------------------

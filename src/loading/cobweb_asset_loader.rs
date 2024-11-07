use bevy::asset::io::{AssetSourceId, Reader};
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use thiserror::Error;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

struct CobwebAssetLoader;

impl AssetLoader for CobwebAssetLoader
{
    type Asset = CobwebAssetFile;
    type Settings = ();
    type Error = CobwebAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error>
    {
        // Get file name including source.
        let file = load_context.asset_path().path().to_string_lossy();
        let file = match load_context.asset_path().source() {
            AssetSourceId::Default => String::from(&*file),
            AssetSourceId::Name(name) => format!("{}://{}", *name, &*file),
        };

        // Read the file.
        let mut string = String::default();
        reader.read_to_string(&mut string).await?;

        // Parse the raw file data.
        let data = match Caf::parse(Span::new_extra(&string, CafLocationMetadata { file: file.as_str() })) {
            Ok(data) => data,
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                let nom::error::Error { input, code } = err;
                return Err(CobwebAssetLoaderError::CafParsing(
                    format!("error at {}: {:?}", get_location(input).as_str(), code),
                ));
            }
            Err(nom::Err::Incomplete(err)) => {
                return Err(CobwebAssetLoaderError::CafParsing(
                    format!("insufficient data in {}: {:?}", file.as_str(), err),
                ));
            }
        };

        Ok(CobwebAssetFile(data))
    }

    fn extensions(&self) -> &[&str]
    {
        &[".caf"]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Possible errors that can be produced by the internal `CobwebAssetLoader`.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CobwebAssetLoaderError
{
    /// An [IO Error](std::io::Error).
    #[error("Could not read the CobwebAssetFile file: {0}")]
    Io(#[from] std::io::Error),
    /// A CAF Error.
    #[error("Could not parse the CobwebAssetFile data: {0}")]
    CafParsing(String),
}

//-------------------------------------------------------------------------------------------------------------------

/// A deserialized CAF file.
// TODO: for editor feature, save hash of file data so editor can detect manual updates vs updates triggered by a
// save event
#[derive(Debug, Asset, TypePath)]
pub(crate) struct CobwebAssetFile(pub(crate) Caf);

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebAssetLoaderPlugin;

impl Plugin for CobwebAssetLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_asset_loader(CobwebAssetLoader);
    }
}

//-------------------------------------------------------------------------------------------------------------------

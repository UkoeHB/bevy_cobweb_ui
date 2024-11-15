use bevy::asset::io::{AssetSourceId, Reader};
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use thiserror::Error;

#[cfg(feature = "editor")]
use crate::editor::{CobFileHash, CobHashRegistry};
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

struct CobAssetLoader
{
    #[cfg(feature = "editor")]
    registry: CobHashRegistry,
}

impl AssetLoader for CobAssetLoader
{
    type Asset = CobAssetFile;
    type Settings = ();
    type Error = CobAssetLoaderError;

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

        // When using the editor, we may be able to discard incoming files if they were saved by the editor.
        #[cfg(feature = "editor")]
        let hash = CobFileHash::new(string.as_bytes());
        #[cfg(feature = "editor")]
        {
            if !self.registry.try_refresh_file(&file, hash) {
                tracing::info!("ignoring file reload for {}; file did not change", file.as_str());
                return Ok(CobAssetFile::Ignore);
            }
        }

        // Parse the raw file data.
        let data = match Cob::parse(Span::new_extra(&string, CobLocationMetadata { file: file.as_str() })) {
            Ok(data) => data,
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                let nom::error::Error { input, code } = err;
                return Err(CobAssetLoaderError::CobParsing(
                    format!("error at {}: {:?}", get_location(input).as_str(), code),
                ));
            }
            Err(nom::Err::Incomplete(err)) => {
                return Err(CobAssetLoaderError::CobParsing(
                    format!("insufficient data in {}: {:?}", file.as_str(), err),
                ));
            }
        };

        #[cfg(not(feature = "editor"))]
        {
            return Ok(CobAssetFile::File { data });
        }

        #[cfg(feature = "editor")]
        {
            return Ok(CobAssetFile::File { hash, data });
        }
    }

    fn extensions(&self) -> &[&str]
    {
        &[".cob"]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Possible errors that can be produced by the internal `CobAssetLoader`.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CobAssetLoaderError
{
    /// An [IO Error](std::io::Error).
    #[error("Could not read the CobAssetFile file: {0}")]
    Io(#[from] std::io::Error),
    /// A COB Error.
    #[error("Could not parse the CobAssetFile data: {0}")]
    CobParsing(String),
}

//-------------------------------------------------------------------------------------------------------------------

/// A deserialized COB file.
#[derive(Debug, Asset, TypePath)]
#[allow(dead_code)]
pub(crate) enum CobAssetFile
{
    /// Used to ignore files saved by the editor (when the "editor" feature is enabled).
    Ignore,
    /// A parsed file.
    File
    {
        #[cfg(feature = "editor")]
        hash: CobFileHash,
        data: Cob,
    },
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobAssetLoaderPlugin;

impl Plugin for CobAssetLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        #[cfg(not(feature = "editor"))]
        {
            app.register_asset_loader(CobAssetLoader {});
        }

        #[cfg(feature = "editor")]
        {
            let registry = app
                .world_mut()
                .get_resource_or_init::<CobHashRegistry>()
                .clone();
            app.register_asset_loader(CobAssetLoader { registry });
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

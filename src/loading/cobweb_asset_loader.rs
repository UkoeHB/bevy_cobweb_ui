use std::collections::HashMap;

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

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        load_context: &'a mut LoadContext<'_>,
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

/// Instructs the asset server to load all pre-set CobwebAssetCache files.
fn load_cobweb_assets(
    mut files: ResMut<LoadedCobwebAssetFiles>,
    mut caf_cache: ResMut<CobwebAssetCache>,
    asset_server: Res<AssetServer>,
)
{
    for file in files.take_preset_files() {
        files.start_loading(file, &mut caf_cache, &asset_server);
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

/// Stores asset paths for all pre-registered cobweb asset files that should be loaded.
#[derive(Resource, Default)]
pub(crate) struct LoadedCobwebAssetFiles
{
    preset_files: Vec<CafFile>,
    handles: HashMap<AssetId<CobwebAssetFile>, Handle<CobwebAssetFile>>,
}

impl LoadedCobwebAssetFiles
{
    fn add_preset_file(&mut self, file: &str)
    {
        match CafFile::try_new(file) {
            Some(file) => {
                tracing::info!("registered CAF file {}", file.as_str());
                self.preset_files.push(file);
            }
            None => {
                tracing::warn!("failed registering CAF file {}; does not have '.caf' extension", file)
            }
        }
    }

    fn take_preset_files(&mut self) -> Vec<CafFile>
    {
        std::mem::take(&mut self.preset_files)
    }

    pub(crate) fn start_loading(
        &mut self,
        file: CafFile,
        caf_cache: &mut CobwebAssetCache,
        asset_server: &AssetServer,
    )
    {
        let handle = asset_server.load(String::from(file.as_str()));
        self.handles.insert(handle.id(), handle);
        caf_cache.prepare_file(file);
    }

    /// Does not remove the handle in case the asset gets reloaded.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn get_handle(&self, id: AssetId<CobwebAssetFile>) -> Option<Handle<CobwebAssetFile>>
    {
        self.handles.get(&id).cloned()
    }

    /// Removes the handle to clean it up properly.
    #[cfg(not(feature = "hot_reload"))]
    pub(crate) fn get_handle(&mut self, id: AssetId<CobwebAssetFile>) -> Option<Handle<CobwebAssetFile>>
    {
        self.handles.remove(&id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting [`CobwebAssetCache`] use.
pub trait LoadedCobwebAssetFilesAppExt
{
    /// Registers a cobweb asset file to be loaded.
    fn load(&mut self, file: impl AsRef<str>) -> &mut Self;
}

impl LoadedCobwebAssetFilesAppExt for App
{
    fn load(&mut self, file: impl AsRef<str>) -> &mut Self
    {
        if !self.world().contains_resource::<LoadedCobwebAssetFiles>() {
            self.init_resource::<LoadedCobwebAssetFiles>();
        }

        self.world_mut()
            .resource_mut::<LoadedCobwebAssetFiles>()
            .add_preset_file(file.as_ref());
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin to load [`CobwebAssetCache`] files into [`CobwebAssetFiles`](CobwebAssetFile).
pub(crate) struct CobwebAssetLoaderPlugin;

impl Plugin for CobwebAssetLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        if !app.world().contains_resource::<LoadedCobwebAssetFiles>() {
            app.init_resource::<LoadedCobwebAssetFiles>();
        }

        app.init_asset::<CobwebAssetFile>()
            .register_asset_loader(CobwebAssetLoader)
            .add_systems(PreStartup, load_cobweb_assets);
    }
}

//-------------------------------------------------------------------------------------------------------------------

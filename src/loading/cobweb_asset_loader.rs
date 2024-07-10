use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde_json::from_slice;
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
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let data: serde_json::Value = from_slice(&bytes)?;
        Ok(CobwebAssetFile {
            file: LoadableFile::new(&load_context.asset_path().path().to_string_lossy()),
            data,
        })
    }

    fn extensions(&self) -> &[&str]
    {
        &[".caf.json"]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instructs the asset server to load all pre-set CobwebAssetCache files.
fn load_cobweb_assets(
    mut files: ResMut<LoadedCobwebAssetFiles>,
    mut caf_cache: ReactResMut<CobwebAssetCache>,
    asset_server: Res<AssetServer>,
)
{
    for file in files.take_preset_files() {
        files.start_loading(file, caf_cache.get_noreact(), &asset_server);
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
    /// A [JSON Error](serde_json::error::Error).
    #[error("Could not parse the CobwebAssetFile JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),
}

//-------------------------------------------------------------------------------------------------------------------

/// A partially-deserialized CobwebAssetCache file.
#[derive(Debug, Asset, TypePath)]
pub(crate) struct CobwebAssetFile
{
    pub(crate) file: LoadableFile,
    pub(crate) data: serde_json::Value,
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores asset paths for all cobweb asset files that should be loaded.
#[derive(Resource, Default)]
pub(crate) struct LoadedCobwebAssetFiles
{
    preset_files: Vec<LoadableFile>,
    handles: HashMap<AssetId<CobwebAssetFile>, Handle<CobwebAssetFile>>,
}

impl LoadedCobwebAssetFiles
{
    fn add_preset_file(&mut self, file: &str)
    {
        tracing::info!("registered CobwebAssetCache file \"{:?}\"", file);
        self.preset_files.push(LoadableFile::new(file));
    }

    fn take_preset_files(&mut self) -> Vec<LoadableFile>
    {
        std::mem::take(&mut self.preset_files)
    }

    pub(crate) fn start_loading(
        &mut self,
        file: LoadableFile,
        caf_cache: &mut CobwebAssetCache,
        asset_server: &AssetServer,
    )
    {
        caf_cache.prepare_file(file.clone());
        let handle = asset_server.load(String::from(file.as_str()));
        self.handles.insert(handle.id(), handle);
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

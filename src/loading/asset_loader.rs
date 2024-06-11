use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde_json::from_slice;
use thiserror::Error;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

struct LoadableSheetAssetLoader;

impl AssetLoader for LoadableSheetAssetLoader
{
    type Asset = LoadableSheetAsset;
    type Settings = ();
    type Error = LoadableSheetAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>>
    {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            //todo: replace this with custom parsing that only allocates where absolutely necessary
            let data: serde_json::Value = from_slice(&bytes)?;
            Ok(LoadableSheetAsset {
                file: LoadableFile::new(&load_context.asset_path().path().to_string_lossy()),
                data,
            })
        })
    }

    fn extensions(&self) -> &[&str]
    {
        &[".load.json"]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instructs the asset server to load all pre-set loadablesheet files.
fn load_sheets(
    mut sheets: ResMut<LoadableSheetList>,
    mut loadablesheet: ReactResMut<LoadableSheet>,
    asset_server: Res<AssetServer>,
)
{
    for sheet in sheets.take_preset_files() {
        sheets.start_loading_sheet(sheet, loadablesheet.get_noreact(), &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Possible errors that can be produced by the internal `LoadableSheetAssetLoader`.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoadableSheetAssetLoaderError
{
    /// An [IO Error](std::io::Error).
    #[error("Could not read the loadablesheet file: {0}")]
    Io(#[from] std::io::Error),
    /// A [JSON Error](serde_json::error::Error).
    #[error("Could not parse the loadablesheet JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),
}

//-------------------------------------------------------------------------------------------------------------------

/// A partially-deserialized loadablesheet file.
#[derive(Debug, Asset, TypePath)]
pub(crate) struct LoadableSheetAsset
{
    pub(crate) file: LoadableFile,
    pub(crate) data: serde_json::Value,
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores asset paths for all loadablesheets that should be loaded.
#[derive(Resource, Default)]
pub(crate) struct LoadableSheetList
{
    preset_files: Vec<LoadableFile>,
    handles: HashMap<AssetId<LoadableSheetAsset>, Handle<LoadableSheetAsset>>,
}

impl LoadableSheetList
{
    fn add_preset_file(&mut self, file: &str)
    {
        tracing::info!("registered loadablesheet file \"{:?}\"", file);
        self.preset_files.push(LoadableFile::new(file));
    }

    fn take_preset_files(&mut self) -> Vec<LoadableFile>
    {
        std::mem::take(&mut self.preset_files)
    }

    pub(crate) fn start_loading_sheet(
        &mut self,
        file: LoadableFile,
        loadablesheet: &mut LoadableSheet,
        asset_server: &AssetServer,
    )
    {
        loadablesheet.prepare_file(file.clone());
        let handle = asset_server.load(String::from(file.as_str()));
        self.handles.insert(handle.id(), handle);
    }

    pub(crate) fn get_handle(&self, id: AssetId<LoadableSheetAsset>) -> Option<&Handle<LoadableSheetAsset>>
    {
        self.handles.get(&id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting [`LoadableSheet`] use.
pub trait LoadableSheetListAppExt
{
    /// Registers a loadable sheet file to be loaded as a loadablesheet asset.
    fn load_sheet(&mut self, file: impl AsRef<str>) -> &mut Self;
}

impl LoadableSheetListAppExt for App
{
    fn load_sheet(&mut self, file: impl AsRef<str>) -> &mut Self
    {
        if !self.world.contains_resource::<LoadableSheetList>() {
            self.init_resource::<LoadableSheetList>();
        }

        self.world
            .resource_mut::<LoadableSheetList>()
            .add_preset_file(file.as_ref());
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin to load [`LoadableSheet`] files into [`LoadableSheetAssets`](LoadableSheetAsset).
pub(crate) struct LoadableSheetAssetLoaderPlugin;

impl Plugin for LoadableSheetAssetLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        if !app.world.contains_resource::<LoadableSheetList>() {
            app.init_resource::<LoadableSheetList>();
        }

        app.init_asset::<LoadableSheetAsset>()
            .register_asset_loader(LoadableSheetAssetLoader)
            .add_systems(PreStartup, load_sheets);
    }
}

//-------------------------------------------------------------------------------------------------------------------

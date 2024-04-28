use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy::prelude::*;
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

/// Instructs the asset server to load all loadablesheet files.
fn load_sheets(mut sheets: ResMut<LoadableSheetList>, asset_server: Res<AssetServer>)
{
    let mut handles = HashMap::default();

    for sheet in sheets.iter_files() {
        let handle = asset_server.load(sheet.clone());
        handles.insert(handle.id(), handle);
    }

    sheets.set_handles(handles);
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
#[derive(Resource)]
pub(crate) struct LoadableSheetList
{
    files: Vec<String>,
    handles: HashMap<AssetId<LoadableSheetAsset>, Handle<LoadableSheetAsset>>,
}

impl LoadableSheetList
{
    fn add_file(&mut self, file: impl Into<String>)
    {
        let file = file.into();
        tracing::info!("registered loadablesheet file \"{:?}\"", file);
        self.files.push(file);
    }

    fn set_handles(&mut self, handles: HashMap<AssetId<LoadableSheetAsset>, Handle<LoadableSheetAsset>>)
    {
        self.handles = handles;
    }

    pub(crate) fn iter_files(&self) -> impl Iterator<Item = &String> + '_
    {
        self.files.iter()
    }

    pub(crate) fn get_handle(&self, id: AssetId<LoadableSheetAsset>) -> Option<&Handle<LoadableSheetAsset>>
    {
        self.handles.get(&id)
    }
}

impl Default for LoadableSheetList
{
    fn default() -> Self
    {
        Self { files: Vec::default(), handles: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting [`LoadableSheet`] use.
pub trait LoadableSheetListAppExt
{
    /// Registers a loadable sheet file to be loaded as a loadablesheet asset.
    fn add_load_sheet(&mut self, file: impl Into<String>) -> &mut Self;
}

impl LoadableSheetListAppExt for App
{
    fn add_load_sheet(&mut self, file: impl Into<String>) -> &mut Self
    {
        if !self.world.contains_resource::<LoadableSheetList>() {
            self.init_resource::<LoadableSheetList>();
        }

        self.world.resource_mut::<LoadableSheetList>().add_file(file);
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

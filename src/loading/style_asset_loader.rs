use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetApp, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext};
use bevy::prelude::*;
use serde_json::from_slice;
use thiserror::Error;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

struct StyleSheetAssetLoader;

impl AssetLoader for StyleSheetAssetLoader
{
    type Asset = StyleSheetAsset;
    type Settings = ();
    type Error = StyleSheetAssetLoaderError;

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
            Ok(StyleSheetAsset {
                file: StyleFile::new(&load_context.asset_path().path().to_string_lossy()),
                data,
            })
        })
    }

    fn extensions(&self) -> &[&str]
    {
        &[".style.json"]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instructs the asset server to load all stylesheet files.
fn load_stylesheets(mut sheets: ResMut<StyleSheetList>, asset_server: Res<AssetServer>)
{
    let mut handles = HashMap::default();

    for sheet in sheets.iter_files() {
        let handle = asset_server.load(sheet.clone());
        handles.insert(handle.id(), handle);
    }

    sheets.set_handles(handles);
}

//-------------------------------------------------------------------------------------------------------------------

/// Possible errors that can be produced by the internal `StyleSheetAssetLoader`.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum StyleSheetAssetLoaderError
{
    /// An [IO Error](std::io::Error).
    #[error("Could not read the stylesheet file: {0}")]
    Io(#[from] std::io::Error),
    /// A [JSON Error](serde_json::error::Error).
    #[error("Could not parse the stylesheet JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),
}

//-------------------------------------------------------------------------------------------------------------------

/// A partially-deserialized stylesheet file.
#[derive(Debug, Asset, TypePath)]
pub(crate) struct StyleSheetAsset
{
    pub(crate) file: StyleFile,
    pub(crate) data: serde_json::Value,
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores asset paths for all stylesheets that should be loaded.
#[derive(Resource)]
pub(crate) struct StyleSheetList
{
    files: Vec<String>,
    handles: HashMap<AssetId<StyleSheetAsset>, Handle<StyleSheetAsset>>,
}

impl StyleSheetList
{
    fn add_file(&mut self, file: impl Into<String>)
    {
        let file = file.into();
        tracing::info!("registered stylesheet file \"{:?}\"", file);
        self.files.push(file);
    }

    fn set_handles(&mut self, handles: HashMap<AssetId<StyleSheetAsset>, Handle<StyleSheetAsset>>)
    {
        self.handles = handles;
    }

    pub(crate) fn iter_files(&self) -> impl Iterator<Item = &String> + '_
    {
        self.files.iter()
    }

    pub(crate) fn get_handle(&self, id: AssetId<StyleSheetAsset>) -> Option<&Handle<StyleSheetAsset>>
    {
        self.handles.get(&id)
    }
}

impl Default for StyleSheetList
{
    fn default() -> Self
    {
        Self { files: Vec::default(), handles: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting [`StyleSheet`] use.
pub trait StyleSheetListAppExt
{
    /// Registers a style sheet file to be loaded as a stylesheet asset.
    fn add_style_sheet(&mut self, file: impl Into<String>) -> &mut Self;
}

impl StyleSheetListAppExt for App
{
    fn add_style_sheet(&mut self, file: impl Into<String>) -> &mut Self
    {
        if !self.world.contains_resource::<StyleSheetList>() {
            self.init_resource::<StyleSheetList>();
        }

        self.world.resource_mut::<StyleSheetList>().add_file(file);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin to load [`StyleSheet`] files into [`StyleSheetAssets`](StyleSheetAsset).
pub(crate) struct StyleSheetAssetLoaderPlugin;

impl Plugin for StyleSheetAssetLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        if !app.world.contains_resource::<StyleSheetList>() {
            app.init_resource::<StyleSheetList>();
        }

        app.init_asset::<StyleSheetAsset>()
            .register_asset_loader(StyleSheetAssetLoader)
            .add_systems(PreStartup, load_stylesheets);
    }
}

//-------------------------------------------------------------------------------------------------------------------

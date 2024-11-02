use std::collections::HashMap;

use bevy::asset::AssetApp;
use bevy::prelude::*;

use crate::prelude::*;

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

pub(crate) struct AppLoadExtPlugin;

impl Plugin for AppLoadExtPlugin
{
    fn build(&self, app: &mut App)
    {
        if !app.world().contains_resource::<LoadedCobwebAssetFiles>() {
            app.init_resource::<LoadedCobwebAssetFiles>();
        }

        app.init_asset::<CobwebAssetFile>()
            .add_systems(PreStartup, load_cobweb_assets);
    }
}

//-------------------------------------------------------------------------------------------------------------------

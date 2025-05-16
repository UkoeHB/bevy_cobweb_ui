use std::collections::HashMap;

use bevy::asset::AssetApp;
use bevy::prelude::*;

use crate::cob::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Instructs the asset server to load all pre-set CobAssetCache files.
fn load_cobweb_assets(
    mut files: ResMut<LoadedCobAssetFiles>,
    mut cob_cache: ResMut<CobAssetCache>,
    mut commands_buffer: ResMut<CommandsBuffer>,
    asset_server: Res<AssetServer>,
)
{
    let presets = files.take_preset_files();

    // Loads presets.
    for file in presets.iter().cloned() {
        files.start_loading(file, &mut cob_cache, &asset_server);
    }

    // Initialize commands buffer.
    commands_buffer.set_root_file(presets);
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores asset paths for all pre-registered cobweb asset files that should be loaded.
#[derive(Resource, Default)]
pub(crate) struct LoadedCobAssetFiles
{
    preset_files: Vec<CobFile>,
    handles: HashMap<AssetId<CobAssetFile>, Handle<CobAssetFile>>,
}

impl LoadedCobAssetFiles
{
    fn add_preset_file(&mut self, file: &str)
    {
        match CobFile::try_new(file) {
            Some(file) => {
                tracing::info!("registered COB file {}", file.as_str());
                self.preset_files.push(file);
            }
            None => {
                tracing::warn!("failed registering COB file {}; does not have '.cob' extension", file)
            }
        }
    }

    fn take_preset_files(&mut self) -> Vec<CobFile>
    {
        std::mem::take(&mut self.preset_files)
    }

    pub(crate) fn start_loading(
        &mut self,
        file: CobFile,
        cob_cache: &mut CobAssetCache,
        asset_server: &AssetServer,
    )
    {
        let handle = asset_server.load(String::from(file.as_str()));
        self.handles.insert(handle.id(), handle);
        cob_cache.prepare_file(file);
    }

    /// Does not remove the handle in case the asset gets reloaded.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn get_handle(&self, id: AssetId<CobAssetFile>) -> Option<Handle<CobAssetFile>>
    {
        self.handles.get(&id).cloned()
    }

    /// Removes the handle to clean it up properly.
    #[cfg(not(feature = "hot_reload"))]
    pub(crate) fn get_handle(&mut self, id: AssetId<CobAssetFile>) -> Option<Handle<CobAssetFile>>
    {
        self.handles.remove(&id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extends `App` with methods supporting cob file loading.
pub trait LoadedCobAssetFilesAppExt
{
    /// Registers a cobweb asset file to be loaded.
    fn load(&mut self, file: impl AsRef<str>) -> &mut Self;
}

impl LoadedCobAssetFilesAppExt for App
{
    fn load(&mut self, file: impl AsRef<str>) -> &mut Self
    {
        if !self.world().contains_resource::<LoadedCobAssetFiles>() {
            self.init_resource::<LoadedCobAssetFiles>();
        }

        self.world_mut()
            .resource_mut::<LoadedCobAssetFiles>()
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
        if !app.world().contains_resource::<LoadedCobAssetFiles>() {
            app.init_resource::<LoadedCobAssetFiles>();
        }

        app.init_asset::<CobAssetFile>()
            .add_systems(PreStartup, load_cobweb_assets);
    }
}

//-------------------------------------------------------------------------------------------------------------------

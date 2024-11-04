use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn preprocess_cobweb_asset_files(
    asset_server: Res<AssetServer>,
    mut events: EventReader<AssetEvent<CobwebAssetFile>>,
    mut caf_files: ResMut<LoadedCobwebAssetFiles>,
    mut assets: ResMut<Assets<CobwebAssetFile>>,
    mut caf_cache: ResMut<CobwebAssetCache>,
    mut commands_buffer: ResMut<CommandsBuffer>,
)
{
    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => id,
            _ => {
                tracing::debug!("ignoring CobwebAssetCache asset event {:?}", event);
                continue;
            }
        };

        let Some(handle) = caf_files.get_handle(*id) else {
            tracing::warn!("encountered CobwebAssetCache asset event {:?} for an untracked asset", id);
            continue;
        };

        let Some(asset) = assets.remove(&handle) else {
            tracing::error!("failed to remove CobwebAssetCache asset {:?}", handle);
            continue;
        };

        preprocess_caf_file(
            &asset_server,
            &mut caf_files,
            &mut caf_cache,
            &mut commands_buffer,
            asset.0,
        );
    }

    // Note: we don't try to handle asset load failures here because a file load failure is assumed to be
    // catastrophic.
}

//-------------------------------------------------------------------------------------------------------------------

fn process_cobweb_asset_files(
    types: Res<AppTypeRegistry>,
    mut caf_cache: ResMut<CobwebAssetCache>,
    mut c: Commands,
    mut commands_buffer: ResMut<CommandsBuffer>,
    mut scene_loader: ResMut<SceneLoader>,
)
{
    let type_registry = types.read();

    if caf_cache.process_cobweb_asset_files(&type_registry, &mut c, &mut commands_buffer, &mut scene_loader) {
        c.react().broadcast(CafCacheUpdated);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_pending_commands(mut c: Commands, mut buffer: ResMut<CommandsBuffer>, loaders: Res<LoaderCallbacks>)
{
    buffer.apply_pending_commands(&mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

/// Only enabled for hot_reload because normally entities are loaded only once, the first time they subscribe
/// to a loadable ref.
#[cfg(feature = "hot_reload")]
fn apply_pending_node_updates(
    mut c: Commands,
    mut caf_cache: ResMut<CobwebAssetCache>,
    loaders: Res<LoaderCallbacks>,
)
{
    caf_cache.apply_pending_node_updates(&mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
fn cleanup_cobweb_asset_cache(
    mut caf_cache: ResMut<CobwebAssetCache>,
    mut scene_loader: ResMut<SceneLoader>,
    mut removed: RemovedComponents<HasLoadables>,
)
{
    for removed in removed.read() {
        caf_cache.remove_entity(&mut scene_loader, removed);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when the [`CobwebAssetCache`] has been updated with CAF asset data.
pub struct CafCacheUpdated;

//-------------------------------------------------------------------------------------------------------------------

/// System set in [`First`] where files are processed.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct FileProcessingSet;

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that enables loading.
pub(crate) struct CobwebAssetCachePlugin;

impl Plugin for CobwebAssetCachePlugin
{
    fn build(&self, app: &mut App)
    {
        let manifest_map = Arc::new(Mutex::new(ManifestMap::default()));
        app.insert_resource(CobwebAssetCache::new(manifest_map.clone()))
            .register_asset_tracker::<CobwebAssetCache>()
            .insert_resource(CommandsBuffer::new())
            .add_systems(
                First,
                (
                    preprocess_cobweb_asset_files,
                    process_cobweb_asset_files.run_if(|s: Res<CobwebAssetCache>| s.num_preprocessed_pending() > 0),
                    apply_pending_commands,
                    #[cfg(feature = "hot_reload")]
                    apply_pending_node_updates,
                )
                    .chain()
                    .in_set(FileProcessingSet),
            );

        #[cfg(not(feature = "hot_reload"))]
        {
            app.configure_sets(First, FileProcessingSet.run_if(in_state(LoadState::Loading)))
                .add_systems(OnExit(LoadState::Loading), |mut c: Commands| {
                    c.remove_resource::<CommandsBuffer>()
                });
        }

        #[cfg(feature = "hot_reload")]
        app.add_systems(Last, cleanup_cobweb_asset_cache);
    }
}

//-------------------------------------------------------------------------------------------------------------------

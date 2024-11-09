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
    mut scene_buffer: ResMut<SceneBuffer>,
    mut scene_loader: ResMut<SceneLoader>,
)
{
    let type_registry = types.read();

    if caf_cache.process_cobweb_asset_files(
        &type_registry,
        &mut c,
        &mut commands_buffer,
        &mut scene_buffer,
        &mut scene_loader,
    ) {
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
fn apply_pending_node_updates_pre(
    mut c: Commands,
    commands_buffer: Res<CommandsBuffer>,
    mut scene_buffer: ResMut<SceneBuffer>,
    loaders: Res<LoaderCallbacks>,
)
{
    // Check if blocked.
    if commands_buffer.is_blocked() {
        return;
    }

    // Apply current pending updates. This handles spawns that occurred while blocked.
    scene_buffer.apply_pending_node_updates(&mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

/// Only enabled for hot_reload because normally entities are loaded only once, the first time they subscribe
/// to a loadable ref.
#[cfg(feature = "hot_reload")]
fn apply_pending_node_updates_extract(
    types: Res<AppTypeRegistry>,
    mut caf_cache: ResMut<CobwebAssetCache>,
    mut c: Commands,
    commands_buffer: Res<CommandsBuffer>,
    mut scene_buffer: ResMut<SceneBuffer>,
    mut scene_loader: ResMut<SceneLoader>,
)
{
    // Check if blocked.
    if commands_buffer.is_blocked() {
        return;
    }

    // Extract scenes from recently loaded files.
    let type_registry = types.read();
    caf_cache.handle_pending_scene_extraction(&type_registry, &mut c, &mut scene_buffer, &mut scene_loader);
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
fn apply_pending_node_updates_post(
    mut c: Commands,
    commands_buffer: Res<CommandsBuffer>,
    mut scene_buffer: ResMut<SceneBuffer>,
    loaders: Res<LoaderCallbacks>,
)
{
    // Check if blocked.
    if commands_buffer.is_blocked() {
        return;
    }

    // Apply current pending updates again. Doing this here ensures updates occur in an order that is valid based
    // on the current structure of all scenes.
    scene_buffer.apply_pending_node_updates(&mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

/// `HasLoadables` is only removed when the entity is despawned.
#[cfg(feature = "hot_reload")]
fn cleanup_despawned_loaded_entities(
    mut scene_buffer: ResMut<SceneBuffer>,
    mut scene_loader: ResMut<SceneLoader>,
    mut removed: RemovedComponents<HasLoadables>,
)
{
    for removed in removed.read() {
        scene_buffer.remove_entity(&mut scene_loader, removed);
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
            .insert_resource(SceneBuffer::new(manifest_map))
            .add_systems(
                First,
                (
                    preprocess_cobweb_asset_files,
                    process_cobweb_asset_files.run_if(|s: Res<CobwebAssetCache>| s.num_preprocessed_pending() > 0),
                    apply_pending_commands,
                    #[cfg(feature = "hot_reload")]
                    apply_pending_node_updates_pre,
                    #[cfg(feature = "hot_reload")]
                    apply_pending_node_updates_extract,
                    #[cfg(feature = "hot_reload")]
                    apply_pending_node_updates_post,
                )
                    .chain()
                    .in_set(FileProcessingSet),
            );

        #[cfg(not(feature = "hot_reload"))]
        {
            app.configure_sets(First, FileProcessingSet.run_if(in_state(LoadState::Loading)))
                .add_systems(OnExit(LoadState::Loading), |mut c: Commands| {
                    c.remove_resource::<CommandsBuffer>();
                });
        }

        #[cfg(feature = "hot_reload")]
        app.add_systems(Last, cleanup_despawned_loaded_entities);
    }
}

//-------------------------------------------------------------------------------------------------------------------

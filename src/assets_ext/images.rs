use std::collections::{HashMap, HashSet};

use bevy::asset::AssetLoadFailedEvent;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_images(In(paths): In<Vec<String>>, asset_server: Res<AssetServer>, mut map: ResMut<ImageMap>)
{
    for path in paths {
        map.insert(path, &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn check_loaded_images(
    mut events: EventReader<AssetEvent<Image>>,
    mut errors: EventReader<AssetLoadFailedEvent<Image>>,
    mut map: ResMut<ImageMap>,
)
{
    for event in events.read() {
        let AssetEvent::Added { id } = event else { continue };
        map.remove_pending(id);
    }

    for error in errors.read() {
        let AssetLoadFailedEvent { id, .. } = error;
        map.remove_pending(id);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to loaded image textures.
#[derive(Resource, Default)]
pub struct ImageMap
{
    pending: HashSet<AssetId<Image>>,
    map: HashMap<String, Handle<Image>>,
}

impl ImageMap
{
    /// Adds an image that should be loaded.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the image to be loaded.
    pub fn insert(&mut self, path: impl AsRef<str> + Into<String>, asset_server: &AssetServer)
    {
        if self.map.contains_key(path.as_ref()) {
            tracing::warn!("ignoring duplicate load for image {}", path.as_ref());
            return;
        }

        let path = path.into();
        let handle = asset_server.load(path.clone());
        self.pending.insert(handle.id());
        self.map.insert(path, handle);
    }

    fn remove_pending(&mut self, id: &AssetId<Image>)
    {
        let _ = self.pending.remove(id);
    }

    /// Gets an image handle for the given path.
    ///
    /// Returns a default handle if the image was not pre-inserted via [`Self::insert`].
    pub fn get(&self, path: impl AsRef<str>) -> Handle<Image>
    {
        let Some(entry) = self.map.get(path.as_ref()) else {
            tracing::error!("failed getting image {} that was not loaded; use LoadImages command", path.as_ref());
            return Default::default();
        };
        entry.clone()
    }
}

impl AssetLoadProgress for ImageMap
{
    fn pending_assets(&self) -> usize
    {
        self.pending.len()
    }

    fn total_assets(&self) -> usize
    {
        self.map.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering image assets that need to be pre-loaded.
///
/// The loaded images can be accessed via [`ImageMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadImages(pub Vec<String>);

impl Command for LoadImages
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_images);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ImageLoadPlugin;

impl Plugin for ImageLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<ImageMap>()
            .register_asset_tracker::<ImageMap>()
            .register_command::<LoadImages>()
            .add_systems(PreUpdate, check_loaded_images.before(LoadProgressSet::AssetProgress));
    }
}

//-------------------------------------------------------------------------------------------------------------------

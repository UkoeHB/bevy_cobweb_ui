use std::collections::{HashMap, HashSet};

use bevy::asset::AssetLoadFailedEvent;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_images(In(images): In<Vec<String>>, asset_server: Res<AssetServer>, mut img_map: ResMut<ImageMap>)
{
    for image in images {
        img_map.insert(image, &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn check_loaded_images(
    mut events: EventReader<AssetEvent<Image>>,
    mut errors: EventReader<AssetLoadFailedEvent<Image>>,
    mut img_map: ResMut<ImageMap>,
)
{
    for event in events.read() {
        let AssetEvent::Added { id } = event else { continue };
        img_map.remove_pending(id);
    }

    for error in errors.read() {
        let AssetLoadFailedEvent { id, .. } = error;
        img_map.remove_pending(id);
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
    /// Add an image that should be loaded.
    ///
    /// Note that if this is called during [`LoadProgressSet::Loading`], then [`LoadProgressSet::Done`] will wait
    /// for the image to be loaded.
    pub fn insert(&mut self, path: String, asset_server: &AssetServer)
    {
        if self.map.contains_key(&path) {
            tracing::warn!("ignoring duplicate load for image {}", path);
            return;
        }

        let handle = asset_server.load(&path);
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
    pub fn get(&self, path: &String) -> Handle<Image>
    {
        let Some(entry) = self.map.get(path) else {
            tracing::error!("failed getting image {} that was not loaded; use LoadImages command", path);
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
/// The loaded images can be access via [`ImageMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadImages(pub Vec<String>);

impl ApplyCommand for LoadImages
{
    fn apply(self, c: &mut Commands)
    {
        c.syscall(self.0, load_images);
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

use std::collections::{HashMap, HashSet};

use bevy::asset::AssetLoadFailedEvent;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_fonts(In(paths): In<Vec<String>>, asset_server: Res<AssetServer>, mut map: ResMut<FontMap>)
{
    for path in paths {
        map.insert(path, &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn check_loaded_fonts(
    mut events: EventReader<AssetEvent<Font>>,
    mut errors: EventReader<AssetLoadFailedEvent<Font>>,
    mut map: ResMut<FontMap>,
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

/// Resource that stores handles to loaded fonts.
//todo: how to distinguish between a request for font for localization vs request for font for manual
// non-localized text?
#[derive(Resource, Default)]
pub struct FontMap
{
    pending: HashSet<AssetId<Font>>,
    map: HashMap<String, Handle<Font>>,
}

impl FontMap
{
    /// Adds a font that should be loaded.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the font to be loaded.
    pub fn insert(&mut self, path: impl AsRef<str> + Into<String>, asset_server: &AssetServer)
    {
        if self.map.contains_key(path.as_ref()) {
            tracing::warn!("ignoring duplicate load for font {}", path.as_ref());
            return;
        }

        let path = path.into();
        let handle = asset_server.load(path.clone());
        self.pending.insert(handle.id());
        self.map.insert(path, handle);
    }

    //todo: insert_localized()

    fn remove_pending(&mut self, id: &AssetId<Font>)
    {
        let _ = self.pending.remove(id);
    }

    /// Gets a font handle for the given path.
    ///
    /// Returns a default handle if the font was not pre-inserted via [`Self::insert`].
    pub fn get(&self, path: impl AsRef<str>) -> Handle<Font>
    {
        let Some(entry) = self.map.get(path.as_ref()) else {
            tracing::error!("failed getting font {} that was not loaded; use LoadFonts command", path.as_ref());
            return Default::default();
        };
        entry.clone()
    }

    //todo: get_or_insert()
}

//todo: delay asset loading until the TextLocalizer knows the language list

impl AssetLoadProgress for FontMap
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

/// Loadable command for registering font assets that need to be pre-loaded.
///
/// The loaded fonts can be accessed via [`FontMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadFonts(pub Vec<String>);

impl ApplyCommand for LoadFonts
{
    fn apply(self, c: &mut Commands)
    {
        c.syscall(self.0, load_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct FontLoadPlugin;

impl Plugin for FontLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<FontMap>()
            .register_asset_tracker::<FontMap>()
            .register_command::<LoadFonts>()
            .add_systems(PreUpdate, check_loaded_fonts.before(LoadProgressSet::AssetProgress));
    }
}

//-------------------------------------------------------------------------------------------------------------------

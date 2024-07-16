use std::collections::HashMap;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_texture_atlas_layouts(
    In(mut layouts): In<Vec<LoadedTextureAtlasLayout>>,
    mut map: ResMut<TextureAtlasLayoutMap>,
    mut layout_assets: ResMut<Assets<TextureAtlasLayout>>,
)
{
    for loaded_layout in layouts.drain(..) {
        let layout = loaded_layout.get_layout();
        map.insert(loaded_layout.texture, loaded_layout.alias, &mut layout_assets, layout);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to [`TextureAtlasLayouts`](TextureAtlasLayout).
///
/// Values can be loaded via [`LoadTextureAtlasLayouts`].
#[derive(Resource, Default)]
pub struct TextureAtlasLayoutMap
{
    map: HashMap<String, HashMap<String, Handle<TextureAtlasLayout>>>,
}

impl TextureAtlasLayoutMap
{
    /// Inserts a layout entry.
    ///
    /// Layouts are indexed by `texture` and also an `alias` in case you need multiple layouts for a given texture.
    pub fn insert(
        &mut self,
        texture: String,
        alias: String,
        assets: &mut Assets<TextureAtlasLayout>,
        layout: TextureAtlasLayout,
    )
    {
        self.map
            .entry(texture)
            .or_default()
            .insert(alias, assets.add(layout));
    }

    /// Gets a handle from the map.
    ///
    /// Returns `Handle::default` if the layout was not found.
    pub fn get(&self, texture: impl AsRef<str>, alias: impl AsRef<str>) -> Handle<TextureAtlasLayout>
    {
        self.map
            .get(texture.as_ref())
            .and_then(|l| l.get(alias.as_ref()))
            .cloned()
            .unwrap_or_default()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering texture altases that need to be pre-loaded.
///
/// The loaded atlases can be accessed via [`TextureAtlasLayoutMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadTextureAtlasLayouts(pub Vec<LoadedTextureAtlasLayout>);

impl Command for LoadTextureAtlasLayouts
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_texture_atlas_layouts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct TextureAtlasLoadPlugin;

impl Plugin for TextureAtlasLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<LoadTextureAtlasLayouts>()
            .init_resource::<TextureAtlasLayoutMap>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

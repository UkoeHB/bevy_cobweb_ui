use std::collections::HashMap;

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
    /// [ texture : [ alias : layout handle ] ]
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

/// Mirrors [`TextureAtlasLayout`] for serialization.
///
/// Used in combination with [`TextureAtlasLayoutMap`] to get atlas layout handles.
///
/// Includes an `alias`, which can be used by [`TextureAtlasReference`] to access the layout.
///
/// See [`LoadTextureAtlasLayouts`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedTextureAtlasLayout
{
    /// The texture this layout is affiliated with.
    pub texture: String,
    /// The alias assigned to this layout, for use in accessing the layout's handle in [`TextureAtlasLayoutMap`].
    pub alias: String,
    pub tile_size: UVec2,
    pub columns: u32,
    pub rows: u32,
    #[reflect(default)]
    pub padding: Option<UVec2>,
    #[reflect(default)]
    pub offset: Option<UVec2>,
}

impl LoadedTextureAtlasLayout
{
    /// Gets a handle to the atlas layout.
    ///
    /// To avoid re-allocating the layout, it is mapped to a string representing the associated image.
    pub fn get_layout(&self) -> TextureAtlasLayout
    {
        TextureAtlasLayout::from_grid(self.tile_size, self.columns, self.rows, self.padding, self.offset)
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
        app.init_resource::<TextureAtlasLayoutMap>()
            .register_command_type::<LoadTextureAtlasLayouts>()
            .register_type::<LoadedTextureAtlasLayout>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

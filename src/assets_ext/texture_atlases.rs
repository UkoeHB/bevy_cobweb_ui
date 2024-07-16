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
    /// Layouts are indexed by `texture` and an `alias` in case you need multiple layouts for a given texture.
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

/// Mirrors [`BorderRect`] for serialization.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliceRect
{
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Into<BorderRect> for SliceRect
{
    fn into(self) -> BorderRect
    {
        BorderRect {
            top: self.top,
            bottom: self.bottom,
            left: self.left,
            right: self.left,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SliceScaleMode`] for serialization.
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadedSliceScaleMode
{
    /// Slices stretch to fill space.
    #[default]
    Stretch,
    /// Slices repeat instead of stretching.
    Tile
    {
        /// Determines after what percent of the slice size the slice will repeat.
        ///
        /// For example, if `0.5` then only up to the first half of the slice will be shown before repeating.
        stretch_value: f32,
    },
}

impl Into<SliceScaleMode> for LoadedSliceScaleMode
{
    fn into(self) -> SliceScaleMode
    {
        match self {
            Self::Stretch => SliceScaleMode::Stretch,
            Self::Tile { stretch_value } => SliceScaleMode::Tile { stretch_value },
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`TextureSlicer`] for serialization.
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedTextureSlicer
{
    /// The sprite borders, defining the 9 sections of the image.
    pub border: SliceRect,
    /// Defines how the center part of the 9 slices will scale.
    #[reflect(default)]
    pub center_scale_mode: LoadedSliceScaleMode,
    /// Defines how the 4 side parts of the 9 slices will scale.
    #[reflect(default)]
    pub sides_scale_mode: LoadedSliceScaleMode,
    /// Defines the maximum scale of the 4 corner slices (defaults to `1.0`).
    #[reflect(default = "LoadedTextureSlicer::default_corner_scale")]
    pub max_corner_scale: f32,
}

impl LoadedTextureSlicer
{
    fn default_corner_scale() -> f32
    {
        1.0
    }
}

impl Default for LoadedTextureSlicer
{
    fn default() -> Self
    {
        Self {
            border: Default::default(),
            center_scale_mode: Default::default(),
            sides_scale_mode: Default::default(),
            max_corner_scale: Self::default_corner_scale(),
        }
    }
}

impl Into<TextureSlicer> for LoadedTextureSlicer
{
    fn into(self) -> TextureSlicer
    {
        TextureSlicer {
            border: self.border.into(),
            center_scale_mode: self.center_scale_mode.into(),
            sides_scale_mode: self.sides_scale_mode.into(),
            max_corner_scale: self.max_corner_scale,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ImageScaleMode`] for serialization.
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadedImageScaleMode
{
    Sliced(LoadedTextureSlicer),
    Tiled
    {
        tile_x: bool,
        tile_y: bool,
        stretch_value: f32,
    },
}

impl Default for LoadedImageScaleMode
{
    fn default() -> Self
    {
        Self::Sliced(LoadedTextureSlicer::default())
    }
}

impl Into<ImageScaleMode> for LoadedImageScaleMode
{
    fn into(self) -> ImageScaleMode
    {
        match self {
            Self::Sliced(slicer) => ImageScaleMode::Sliced(slicer.into()),
            Self::Tiled { tile_x, tile_y, stretch_value } => {
                ImageScaleMode::Tiled { tile_x, tile_y, stretch_value }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`TextureAtlasLayout`] for serialization.
///
/// Used in combination with [`TextureAtlasLayoutMap`] to get atlas layout handles.
///
/// Includes an `alias`, which can be used by [`TextureAtlasReference`] to access the layout.
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

/// Used to create a [`TextureAtlas`] by accessing a [`TextureAtlasLayout`] by reference via
/// [`TextureAtlasLayoutMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureAtlasReference
{
    /// The identifier for this texture atlas map, which can be used to reference this atlas
    /// The index into the atlas for the desired sprite.
    pub index: usize,
    /// The alias of the [`TextureAtlasLayout`] that is referenced.
    ///
    /// Note that to get a layout handle from [`TextureAtlasLayoutMap`] you also need the texture, which we assume
    /// is stored adjacent to this atlas reference.
    pub alias: String,
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct TextureAtlasLoadPlugin;

impl Plugin for TextureAtlasLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<SliceRect>()
            .register_type::<LoadedSliceScaleMode>()
            .register_type::<LoadedTextureSlicer>()
            .register_type::<LoadedImageScaleMode>()
            .register_type::<LoadedTextureAtlasLayout>()
            .register_type::<TextureAtlasReference>()
            .register_command::<LoadTextureAtlasLayouts>()
            .init_resource::<TextureAtlasLayoutMap>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

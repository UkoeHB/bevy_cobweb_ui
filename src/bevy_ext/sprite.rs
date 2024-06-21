use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to [`TextureAtlasLayouts`](TextureAtlasLayout).
//TODO: this assumes each image only uses one TextureAtlasLayout, but it's possible for an image to be divided
// into sections with different layouts.
//TODO: add pre-loading and progress tracking
#[derive(Resource, Default)]
pub struct TextureAtlasMap
{
    map: HashMap<String, Handle<TextureAtlasLayout>>,
}

impl TextureAtlasMap
{
    /// Gets a handle from the map.
    pub fn get(
        &mut self,
        image: impl AsRef<str>,
        assets: &mut Assets<TextureAtlasLayout>,
        layout: TextureAtlasLayout,
    ) -> Handle<TextureAtlasLayout>
    {
        let Some(entry) = self.map.get(image.as_ref()) else {
            let entry = assets.add(layout);
            self.map.insert(String::from(image.as_ref()), entry.clone());
            return entry;
        };
        entry.clone()
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
/// Used in combination with [`TextureAtlasMap`] to get atlas layout handles.
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadedTextureAtlasLayout
{
    Layout
    {
        tile_size: Vec2,
        columns: usize,
        rows: usize,
        #[reflect(default)]
        padding: Option<Vec2>,
        #[reflect(default)]
        offset: Option<Vec2>,
    },
    #[reflect(ignore)]
    #[serde(skip)]
    Handle(Handle<TextureAtlasLayout>),
}

impl LoadedTextureAtlasLayout
{
    /// Gets a handle to the atlas layout.
    ///
    /// To avoid re-allocating the layout, it is mapped to a string representing the associated image.
    pub fn get_handle(
        &mut self,
        image: impl AsRef<str>,
        assets: &mut Assets<TextureAtlasLayout>,
        map: &mut TextureAtlasMap,
    ) -> Handle<TextureAtlasLayout>
    {
        match self.clone() {
            Self::Handle(handle) => handle,
            Self::Layout { tile_size, columns, rows, padding, offset } => {
                let layout = TextureAtlasLayout::from_grid(tile_size, columns, rows, padding, offset);
                let handle = map.get(image, assets, layout);
                *self = Self::Handle(handle.clone());
                handle
            }
        }
    }
}

impl Default for LoadedTextureAtlasLayout
{
    fn default() -> Self
    {
        Self::Layout {
            tile_size: Vec2::default(),
            columns: 0,
            rows: 0,
            padding: None,
            offset: None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`TextureAtlas`] for serialization.
///
/// Note that this must include a [`LoadedTextureAtlasLayout::Layout`] when serialized.
//todo: possibly reduce duplication by referring to layout by image path?
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedTextureAtlas
{
    /// The index into the atlas for the desired sprite.
    pub index: usize,
    /// The layout of the atlas.
    pub layout: LoadedTextureAtlasLayout,
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BevySpriteExtPlugin;

impl Plugin for BevySpriteExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<SliceRect>()
            .register_type::<LoadedSliceScaleMode>()
            .register_type::<LoadedTextureSlicer>()
            .register_type::<LoadedImageScaleMode>()
            .register_type::<LoadedTextureAtlas>()
            .register_type::<LoadedTextureAtlasLayout>()
            .init_resource::<TextureAtlasMap>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

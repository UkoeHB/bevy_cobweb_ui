use bevy::prelude::*;

#[allow(unused_imports)]
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRect`] for serialization.
// TODO: use `BorderRect` when it has Serialize/Deserialize
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SliceRect
{
    #[reflect(default)]
    pub top: f32,
    #[reflect(default)]
    pub bottom: f32,
    #[reflect(default)]
    pub left: f32,
    #[reflect(default)]
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
// TODO: use `SliceScaleMode` when it has Serialize/Deserialize
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
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
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// Mirrors [`NodeImageMode`] and [`SpriteImageMode`] for serialization.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadedImageMode
{
    #[default]
    Auto,
    /// Falls back to [`Self::Auto`] when converting to [`SpriteImageMode`].
    Stretch,
    Sliced(LoadedTextureSlicer),
    Tiled
    {
        tile_x: bool,
        tile_y: bool,
        stretch_value: f32,
    },
}

impl Into<NodeImageMode> for LoadedImageMode
{
    fn into(self) -> NodeImageMode
    {
        match self {
            Self::Auto => NodeImageMode::Auto,
            Self::Stretch => NodeImageMode::Stretch,
            Self::Sliced(slicer) => NodeImageMode::Sliced(slicer.into()),
            Self::Tiled { tile_x, tile_y, stretch_value } => {
                NodeImageMode::Tiled { tile_x, tile_y, stretch_value }
            }
        }
    }
}

impl Into<SpriteImageMode> for LoadedImageMode
{
    fn into(self) -> SpriteImageMode
    {
        match self {
            Self::Auto => SpriteImageMode::Auto,
            Self::Stretch => SpriteImageMode::Auto,
            Self::Sliced(slicer) => SpriteImageMode::Sliced(slicer.into()),
            Self::Tiled { tile_x, tile_y, stretch_value } => {
                SpriteImageMode::Tiled { tile_x, tile_y, stretch_value }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Used to create a [`TextureAtlas`] by accessing a [`TextureAtlasLayout`] by reference via
/// [`TextureAtlasLayoutMap`].
///
/// See [`LoadedTextureAtlasLayout`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct TextureAtlasReference
{
    /// The index into the atlas for the desired sprite.
    pub index: usize,
    /// The alias of the [`TextureAtlasLayout`] that is referenced.
    ///
    /// Note that to get a layout handle from [`TextureAtlasLayoutMap`] you also need the texture, which we assume
    /// is stored adjacent to this atlas reference.
    pub alias: String,
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct TextureAtlasExtPlugin;

impl Plugin for TextureAtlasExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<SliceRect>()
            .register_type::<LoadedSliceScaleMode>()
            .register_type::<LoadedTextureSlicer>()
            .register_type::<LoadedImageMode>()
            .register_type::<TextureAtlasReference>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

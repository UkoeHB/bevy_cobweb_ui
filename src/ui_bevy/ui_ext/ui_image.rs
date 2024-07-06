use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::widget::UiImageSize;
use bevy::ui::ContentSize;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Inserts a UiImage to an entity.
//note: in bevy 0.13 you need a non-transparent BackgroundColor for ui images to display, but in 0.14 it should be
// fixed
fn insert_ui_image(In((entity, img)): In<(Entity, LoadedUiImage)>, mut commands: Commands, img_map: Res<ImageMap>)
{
    // Extract
    let content_size = match img.size {
        Some(size) => ContentSize::fixed_size(size),
        None => ContentSize::default(),
    };
    let maybe_scale_mode = img.scale_mode.clone();
    let ui_image = img.to_ui_image(&img_map);

    // Insert
    let mut ec = commands.entity(entity);
    let bundle = (ui_image, UiImageSize::default(), content_size);

    if let Some(scale_mode) = maybe_scale_mode {
        let scale_mode: ImageScaleMode = scale_mode.into();
        ec.try_insert((scale_mode, bundle));
    } else {
        ec.try_insert(bundle);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn update_ui_image_color(In((entity, color)): In<(Entity, Color)>, mut q: Query<&mut UiImage>)
{
    let Ok(mut img) = q.get_mut(entity) else { return };
    img.color = color;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiImage`] for serialization.
///
/// Must be inserted to an entity with [`NodeBundle`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedUiImage
{
    /// The location of the UiImage.
    pub texture: String,
    // The [`TextureAtlas`] to pull this image from. (TODO: broken w/ nine-slicing until Bevy v0.14)
    // atlas: Option<LoadedTextureAtlas>
    /// The scale mode for this image.
    ///
    /// [`LoadedImageScaleMode::Sliced`] can be used for nine-slicing.
    #[reflect(default)]
    pub scale_mode: Option<LoadedImageScaleMode>,
    /// The color of the image.
    #[reflect(default = "LoadedUiImage::default_color")]
    pub color: Color,
    /// The size of the image.
    ///
    /// Set this if you want to force the node to match the image size.
    #[reflect(default)]
    pub size: Option<Vec2>,
    /// Whether to flip the image on its x axis.
    #[reflect(default)]
    pub flip_x: bool,
    /// Whether to flip the image on its y axis.
    #[reflect(default)]
    pub flip_y: bool,
}

impl LoadedUiImage
{
    /// Converts to a [`UiImage`].
    pub fn to_ui_image(self, map: &ImageMap) -> UiImage
    {
        UiImage {
            color: self.color,
            texture: map.get(&self.texture),
            flip_x: self.flip_x,
            flip_y: self.flip_y,
        }
    }

    /// Gets the default color, which is white.
    pub fn default_color() -> Color
    {
        Color::WHITE
    }
}

impl ApplyLoadable for LoadedUiImage
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let entity = ec.id();
        ec.commands().syscall((entity, self), insert_ui_image);
    }
}
impl ThemedAttribute for LoadedUiImage
{
    type Value = Self;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        value.apply(ec);
    }
}
//todo: animate ui images by lerping between indices into a texture map? and panic if not pointing to the same
//      spritesheet
// - might need custom AnimatedUiImage, especially for the case with more than 2 end states

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiImage::color`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiImageColor(pub Color);

impl ApplyLoadable for UiImageColor
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.syscall((id, self.0), update_ui_image_color);
    }
}

impl ThemedAttribute for UiImageColor
{
    type Value = Color;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for UiImageColor {}
impl AnimatableAttribute for UiImageColor {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiImageExtPlugin;

impl Plugin for UiImageExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Option<ImageScaleMode>>()
            .register_themed::<LoadedUiImage>()
            .register_animatable::<UiImageColor>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

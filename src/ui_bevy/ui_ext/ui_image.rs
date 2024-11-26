use bevy::prelude::*;
use bevy::ui::widget::UiImageSize;
use bevy::ui::ContentSize;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Inserts a UiImage to an entity.
fn insert_ui_image(
    In((entity, img)): In<(Entity, LoadedUiImage)>,
    mut commands: Commands,
    img_map: Res<ImageMap>,
    layout_map: Res<TextureAtlasLayoutMap>,
)
{
    let Some(mut ec) = commands.get_entity(entity) else { return };

    // Extract
    let content_size = match img.size {
        Some(size) => ContentSize::fixed_size(size),
        None => ContentSize::default(),
    };
    let ui_image = img.to_ui_image(&img_map, &layout_map);

    // Insert
    // - Note this is a bit messy to avoid archetype moves on insert.
    ec.try_insert((ui_image, content_size));
}

//-------------------------------------------------------------------------------------------------------------------

fn update_ui_image_color(In((entity, color)): In<(Entity, Color)>, mut q: Query<&mut UiImage>)
{
    let Ok(mut img) = q.get_mut(entity) else { return };
    img.color = color;
}

//-------------------------------------------------------------------------------------------------------------------

fn update_ui_image_index(In((entity, index)): In<(Entity, usize)>, mut q: Query<&mut UiImage>)
{
    let Ok(mut img) = q.get_mut(entity) else { return };
    img.texture_atlas.as_mut().map(|a| a.index = index);
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiImage`] for serialization.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoadedUiImage
{
    /// The location of the UiImage.
    ///
    /// If no image is specified, then a default handle will be inserted to the `UiImage`. This is useful if
    /// you want to manually set the image in rust code.
    #[reflect(default)]
    pub image: Option<String>,
    /// A reference to the [`TextureAtlas`] to process this image with.
    ///
    /// The image can be animated using the referenced texture atlas with [`Animated<UiImageIndex>`].
    ///
    /// The atlas's layout should be loaded into [`TextureAtlasLayoutMap`].
    #[reflect(default)]
    pub atlas: Option<TextureAtlasReference>,
    /// The scale mode for this image.
    ///
    /// [`LoadedImageMode::Sliced`] can be used for nine-slicing.
    #[reflect(default)]
    pub mode: Option<LoadedImageMode>,
    /// The color of the image.
    #[reflect(default = "LoadedUiImage::default_color")]
    pub color: Color,
    /// The size of the image.
    ///
    /// Set this if you want to force the node to stretch to a specific size.
    ///
    /// When [`LoadedImageMode::Auto`] is used, the node will automatically size itself to fit the image.
    // TODO: is this ^ a false statement? need to test it
    #[reflect(default)]
    pub size: Option<Vec2>,
    /// Allows specifying a rectangle on the image to render. A cheap alternative to [`Self::atlas`].
    #[reflect(default)]
    pub rect: Option<Rect>,
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
    pub fn to_ui_image(self, map: &ImageMap, layout_map: &TextureAtlasLayoutMap) -> UiImage
    {
        let texture_atlas = self.atlas.and_then(|a| {
            let Some(img) = self.image.as_ref() else {
                tracing::warn!("failed setting TextureAtlas in UiImage when converting LoadedUiImage; the atlas is set but \
                    the image texture is None");
                return None;
            };
            Some(TextureAtlas {
                layout: layout_map.get(img, &a.alias),
                index: a.index,
            })
        });
        UiImage {
            color: self.color,
            image: self.image.map(|i| map.get(&i)).unwrap_or_default(),
            texture_atlas,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            rect: self.rect,
            image_mode: self.mode.unwrap_or_default().into(),
        }
    }

    /// Gets the default color, which is white.
    pub fn default_color() -> Color
    {
        Color::WHITE
    }
}

impl Instruction for LoadedUiImage
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self), insert_ui_image);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            // TODO: requires https://github.com/bevyengine/bevy/pull/16288
            //e.remove_with_requires::<UiImage>();
            e.remove::<(UiImage, ContentSize, UiImageSize)>();
        });
    }
}

impl StaticAttribute for LoadedUiImage
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiImage::color`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct UiImageColor(pub Color);

impl Instruction for UiImageColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self.0), update_ui_image_color);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        Instruction::apply(Self(LoadedUiImage::default_color()), entity, world);
    }
}

impl StaticAttribute for UiImageColor
{
    type Value = Color;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for UiImageColor {}
impl AnimatableAttribute for UiImageColor {}

//-------------------------------------------------------------------------------------------------------------------

/// Allows setting the [`TextureAtlas`] index of a UI image.
///
/// Primarily useful for animating UI textures.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct UiImageIndex(pub usize);

impl Instruction for UiImageIndex
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self.0), update_ui_image_index);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        Instruction::apply(Self(0), entity, world);
    }
}

impl StaticAttribute for UiImageIndex
{
    type Value = usize;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for UiImageIndex {}
impl AnimatableAttribute for UiImageIndex {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiImageExtPlugin;

impl Plugin for UiImageExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_themed::<LoadedUiImage>()
            .register_animatable::<UiImageColor>()
            .register_animatable::<UiImageIndex>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;
use bevy::ui::widget::UiImageSize;
use bevy::ui::ContentSize;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Inserts a UiImage to an entity.
fn insert_ui_image(
    In((entity, mut img)): In<(Entity, LoadedUiImage)>,
    mut commands: Commands,
    img_map: Res<ImageMap>,
    layout_map: Res<TextureAtlasLayoutMap>,
)
{
    // Extract
    let content_size = match img.size {
        Some(size) => ContentSize::fixed_size(size),
        None => ContentSize::default(),
    };
    let maybe_atlas = img.atlas.take().map(|a| TextureAtlas {
        layout: layout_map.get(&img.texture, &a.alias),
        index: a.index,
    });
    let maybe_scale_mode = img.scale_mode.take();
    let ui_image = img.to_ui_image(&img_map);

    // Insert
    // - Note this is a bit messy to avoid archetype moves on insert.
    //todo: simplify when Bevy has batched ECS commands
    let mut ec = commands.entity(entity);
    let bundle = (ui_image, UiImageSize::default(), content_size);

    match (maybe_atlas, maybe_scale_mode) {
        (Some(atlas), Some(scale_mode)) => {
            let scale_mode: ImageScaleMode = scale_mode.into();
            ec.try_insert((atlas, scale_mode, bundle));
        }
        (Some(atlas), None) => {
            ec.try_insert((atlas, bundle));
        }
        (None, Some(scale_mode)) => {
            let scale_mode: ImageScaleMode = scale_mode.into();
            ec.try_insert((scale_mode, bundle));
        }
        _ => {
            ec.try_insert(bundle);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn update_ui_image_color(In((entity, color)): In<(Entity, Color)>, mut q: Query<&mut UiImage>)
{
    let Ok(mut img) = q.get_mut(entity) else { return };
    img.color = color;
}

//-------------------------------------------------------------------------------------------------------------------

fn update_ui_image_index(In((entity, index)): In<(Entity, usize)>, mut q: Query<&mut TextureAtlas, With<UiImage>>)
{
    let Ok(mut atlas) = q.get_mut(entity) else { return };
    atlas.index = index;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiImage`] for serialization.
///
/// Must be inserted to an entity with [`NodeBundle`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct LoadedUiImage
{
    /// The location of the UiImage.
    pub texture: String,
    /// A reference to the [`TextureAtlas`] to process this image with.
    ///
    /// The image can be animated using the referenced texture atlas with [`Animated<UiImageIndex>`].
    ///
    /// The atlas's layout should be loaded into [`TextureAtlasLayoutMap`].
    #[reflect(default)]
    pub atlas: Option<TextureAtlasReference>,
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

impl Instruction for LoadedUiImage
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self), insert_ui_image);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<(UiImage, UiImageSize, ContentSize, ImageScaleMode, TextureAtlas)>();
        });
    }
}

impl ThemedAttribute for LoadedUiImage
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

impl ThemedAttribute for UiImageColor
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
/// Primarily useful for animating UI textures using `sickle_ui`.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
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

impl ThemedAttribute for UiImageIndex
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

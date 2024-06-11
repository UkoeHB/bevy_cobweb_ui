use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::widget::UiImageSize;
use bevy::ui::ContentSize;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn insert_ui_image(
    In((entity, img)): In<(Entity, LoadedUiImage)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut img_map: ResMut<UiImageMap>,
)
{
    // Extract
    let content_size = match img.size {
        Some(size) => ContentSize::fixed_size(size),
        None => ContentSize::default(),
    };
    let maybe_scale_mode = img.scale_mode.clone();
    let ui_image = img.to_ui_image(&asset_server, &mut img_map);

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

/// Resource that stores handles to loaded UI image textures.
//TODO: add pre-loading and progress tracking
#[derive(Resource, Default)]
pub struct UiImageMap
{
    map: HashMap<String, Handle<Image>>,
}

impl UiImageMap
{
    fn get(&mut self, path: Option<String>, asset_server: &AssetServer) -> Handle<Image>
    {
        let Some(path) = path else { return Default::default() };
        let Some(entry) = self.map.get(&path) else {
            let entry = asset_server.load(&path);
            self.map.insert(path, entry.clone());
            return entry;
        };
        entry.clone()
    }
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
    pub fn to_ui_image(self, asset_server: &AssetServer, map: &mut UiImageMap) -> UiImage
    {
        UiImage {
            texture: map.get(self.texture.into(), asset_server),
            flip_x: self.flip_x,
            flip_y: self.flip_y,
        }
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

pub(crate) struct UiImageExtPlugin;

impl Plugin for UiImageExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<UiImageMap>()
            .register_type::<Option<ImageScaleMode>>()
            .register_themed::<LoadedUiImage>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

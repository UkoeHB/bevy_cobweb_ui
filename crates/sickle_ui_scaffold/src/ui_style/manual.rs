use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use bevy::text::TextLayoutInfo;

use super::generated::*;
use super::{LockableStyleAttribute, LockedStyleAttributes, UiStyle, UiStyleUnchecked};
use crate::flux_interaction::FluxInteraction;

// Special style-related components needing manual implementation
macro_rules! check_lock {
    ($world:expr, $entity:expr, $prop:literal, $lock_attr:path) => {
        if let Some(locked_attrs) = $world.get::<LockedStyleAttributes>($entity) {
            if locked_attrs.contains($lock_attr) {
                warn!(
                    "Failed to style {} property on entity {:?}: Attribute locked!",
                    $prop, $entity
                );
                return;
            }
        }
    };
}

impl EntityCommand for SetZIndex
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(world, entity, "z index", LockableStyleAttribute::ZIndex);
        }

        let Some(mut z_index) = world.get_mut::<ZIndex>(entity) else {
            warn!("Failed to set z index on entity {}: No ZIndex component found!", entity);
            return;
        };

        // Best effort avoid change triggering
        if let (ZIndex(level), ZIndex(target)) = (*z_index, self.z_index) {
            if level != target {
                *z_index = self.z_index;
            }
        } else {
            *z_index = self.z_index;
        }
    }
}

#[derive(Clone, Debug)]
pub enum ImageSource
{
    Path(String),
    Lookup(String, fn(String, Entity, &mut World) -> Handle<Image>),
    Handle(Handle<Image>),
    Atlas(String, TextureAtlasLayout),
}

impl Default for ImageSource
{
    fn default() -> Self
    {
        Self::Handle(Handle::default())
    }
}

impl From<&str> for ImageSource
{
    fn from(path: &str) -> Self
    {
        Self::Path(path.to_string())
    }
}

impl From<String> for ImageSource
{
    fn from(path: String) -> Self
    {
        Self::Path(path)
    }
}

pub struct SetImage
{
    source: ImageSource,
    check_lock: bool,
}

impl EntityCommand for SetImage
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(world, entity, "image", LockableStyleAttribute::Image);
        }

        let handle = match self.source.clone() {
            ImageSource::Path(path) => {
                if path == "" {
                    Handle::default()
                } else {
                    world.resource::<AssetServer>().load(path)
                }
            }
            ImageSource::Lookup(path, callback) => callback(path, entity, world),
            ImageSource::Handle(handle) => handle,
            ImageSource::Atlas(path, _) => {
                if path == "" {
                    Handle::default()
                } else {
                    world.resource::<AssetServer>().load(path)
                }
            }
        };

        let Some(mut image) = world.get_mut::<UiImage>(entity) else {
            warn!("Failed to set image on entity {}: No UiImage component found!", entity);
            return;
        };

        if image.image != handle {
            image.image = handle;
        }

        if let ImageSource::Atlas(_, layout) = self.source {
            let layout_handle = world
                .resource_mut::<Assets<TextureAtlasLayout>>()
                .add(layout.clone())
                .clone();

            let Some(mut image) = world.get_mut::<UiImage>(entity) else {
                warn!("Failed to set image on entity {}: No UiImage component found!", entity);
                return;
            };

            if let Some(atlas) = image.texture_atlas.as_mut() {
                if atlas.layout != layout_handle {
                    atlas.layout = layout_handle;
                    atlas.index = 0;
                }
            } else {
                image.texture_atlas = Some(TextureAtlas::from(layout_handle));
            }
        }
    }
}

pub trait SetImageExt
{
    fn image(&mut self, source: ImageSource) -> &mut Self;
}

impl SetImageExt for UiStyle<'_>
{
    fn image(&mut self, source: ImageSource) -> &mut Self
    {
        self.commands.queue(SetImage { source, check_lock: true });
        self
    }
}

pub trait SetImageUncheckedExt
{
    fn image(&mut self, source: ImageSource) -> &mut Self;
}

impl SetImageUncheckedExt for UiStyleUnchecked<'_>
{
    fn image(&mut self, source: ImageSource) -> &mut Self
    {
        self.commands.queue(SetImage { source, check_lock: false });
        self
    }
}

impl EntityCommand for SetImageTint
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(world, entity, "image tint", LockableStyleAttribute::ImageTint);
        }

        let Some(mut image) = world.get_mut::<UiImage>(entity) else {
            warn!(
                "Failed to set image tint on entity {}: No UiImage component found!",
                entity
            );
            return;
        };

        if image.color != self.image_tint {
            image.color = self.image_tint;
        }
    }
}

impl EntityCommand for SetImageFlip
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(world, entity, "image flip", LockableStyleAttribute::ImageFlip);
        }

        let Some(mut image) = world.get_mut::<UiImage>(entity) else {
            warn!(
                "Failed to set image flip on entity {}: No UiImage component found!",
                entity
            );
            return;
        };

        if image.flip_x != self.image_flip.x {
            image.flip_x = self.image_flip.x;
        }

        if image.flip_y != self.image_flip.y {
            image.flip_y = self.image_flip.y;
        }
    }
}

impl EntityCommand for SetImageScaleMode
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(
                world,
                entity,
                "image scale mode",
                LockableStyleAttribute::ImageScaleMode
            );
        }

        if let Some(image_scale_mode) = self.image_scale_mode {
            if let Some(mut scale_mode) = world.get_mut::<ImageScaleMode>(entity) {
                *scale_mode = image_scale_mode;
            } else {
                world.entity_mut(entity).insert(image_scale_mode);
            }
        } else if let Some(_) = world.get::<ImageScaleMode>(entity) {
            world.entity_mut(entity).remove::<ImageScaleMode>();
        }
    }
}

pub struct SetFluxInteractionEnabled
{
    enabled: bool,
    check_lock: bool,
}

impl EntityCommand for SetFluxInteractionEnabled
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(
                world,
                entity,
                "flux interaction",
                LockableStyleAttribute::FluxInteraction
            );
        }

        let Some(mut flux_interaction) = world.get_mut::<FluxInteraction>(entity) else {
            warn!(
                "Failed to set flux interaction on entity {}: No FluxInteraction component found!",
                entity
            );
            return;
        };

        if self.enabled {
            if *flux_interaction == FluxInteraction::Disabled {
                *flux_interaction = FluxInteraction::None;
            }
        } else {
            if *flux_interaction != FluxInteraction::Disabled {
                *flux_interaction = FluxInteraction::Disabled;
            }
        }
    }
}

pub trait SetFluxInteractionExt
{
    fn disable_flux_interaction(&mut self) -> &mut Self;
    fn enable_flux_interaction(&mut self) -> &mut Self;
    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self;
}

impl SetFluxInteractionExt for UiStyle<'_>
{
    fn disable_flux_interaction(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetFluxInteractionEnabled { enabled: false, check_lock: true });
        self
    }

    fn enable_flux_interaction(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetFluxInteractionEnabled { enabled: true, check_lock: true });
        self
    }

    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self
    {
        self.commands
            .queue(SetFluxInteractionEnabled { enabled, check_lock: true });
        self
    }
}

pub trait SetFluxInteractionUncheckedExt
{
    fn disable_flux_interaction(&mut self) -> &mut Self;
    fn enable_flux_interaction(&mut self) -> &mut Self;
    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self;
}

impl SetFluxInteractionUncheckedExt for UiStyleUnchecked<'_>
{
    fn disable_flux_interaction(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetFluxInteractionEnabled { enabled: false, check_lock: false });
        self
    }

    fn enable_flux_interaction(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetFluxInteractionEnabled { enabled: true, check_lock: false });
        self
    }

    fn flux_interaction_enabled(&mut self, enabled: bool) -> &mut Self
    {
        self.commands
            .queue(SetFluxInteractionEnabled { enabled, check_lock: false });
        self
    }
}

pub trait SetNodeShowHideExt
{
    fn show(&mut self) -> &mut Self;
    fn hide(&mut self) -> &mut Self;
    fn render(&mut self, render: bool) -> &mut Self;
}

impl SetNodeShowHideExt for UiStyle<'_>
{
    fn show(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetVisibility { visibility: Visibility::Inherited, check_lock: true })
            .queue(SetDisplay { display: Display::Flex, check_lock: true });
        self
    }

    fn hide(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetVisibility { visibility: Visibility::Hidden, check_lock: true })
            .queue(SetDisplay { display: Display::None, check_lock: true });
        self
    }

    fn render(&mut self, render: bool) -> &mut Self
    {
        if render {
            self.commands
                .queue(SetVisibility { visibility: Visibility::Inherited, check_lock: true })
                .queue(SetDisplay { display: Display::Flex, check_lock: true });
        } else {
            self.commands
                .queue(SetVisibility { visibility: Visibility::Hidden, check_lock: true })
                .queue(SetDisplay { display: Display::None, check_lock: true });
        }

        self
    }
}

pub trait SetNodeShowHideUncheckedExt
{
    fn show(&mut self) -> &mut Self;
    fn hide(&mut self) -> &mut Self;
    fn render(&mut self, render: bool) -> &mut Self;
}

impl SetNodeShowHideUncheckedExt for UiStyleUnchecked<'_>
{
    fn show(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetVisibility { visibility: Visibility::Inherited, check_lock: false })
            .queue(SetDisplay { display: Display::Flex, check_lock: false });
        self
    }

    fn hide(&mut self) -> &mut Self
    {
        self.commands
            .queue(SetVisibility { visibility: Visibility::Hidden, check_lock: false })
            .queue(SetDisplay { display: Display::None, check_lock: false });
        self
    }

    fn render(&mut self, render: bool) -> &mut Self
    {
        if render {
            self.commands
                .queue(SetVisibility { visibility: Visibility::Inherited, check_lock: false })
                .queue(SetDisplay { display: Display::Flex, check_lock: false });
        } else {
            self.commands
                .queue(SetVisibility { visibility: Visibility::Hidden, check_lock: false })
                .queue(SetDisplay { display: Display::None, check_lock: false });
        }

        self
    }
}

pub struct SetAbsolutePosition
{
    absolute_position: Vec2,
    check_lock: bool,
}

impl EntityCommand for SetAbsolutePosition
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(world, entity, "position: top", LockableStyleAttribute::Top);
            check_lock!(world, entity, "position: left", LockableStyleAttribute::Left);
        }

        let offset = if let Some(parent) = world.get::<Parent>(entity) {
            let Some(parent_node) = world.get::<ComputedNode>(parent.get()) else {
                warn!(
                    "Failed to set position on entity {}: Parent has no Node component!",
                    entity
                );
                return;
            };

            let size = parent_node.unrounded_size();
            let Some(parent_transform) = world.get::<GlobalTransform>(parent.get()) else {
                warn!(
                    "Failed to set position on entity {}: Parent has no GlobalTransform component!",
                    entity
                );
                return;
            };

            parent_transform.translation().truncate() - (size / 2.)
        } else {
            Vec2::ZERO
        };

        let Some(mut style) = world.get_mut::<Node>(entity) else {
            warn!("Failed to set position on entity {}: No Node component found!", entity);
            return;
        };

        style.top = Val::Px(self.absolute_position.y - offset.y);
        style.left = Val::Px(self.absolute_position.x - offset.x);
    }
}

pub trait SetAbsolutePositionExt
{
    fn absolute_position(&mut self, position: Vec2) -> &mut Self;
}

impl SetAbsolutePositionExt for UiStyle<'_>
{
    fn absolute_position(&mut self, position: Vec2) -> &mut Self
    {
        self.commands
            .queue(SetAbsolutePosition { absolute_position: position, check_lock: true });
        self
    }
}

pub trait SetAbsolutePositionUncheckedExt
{
    fn absolute_position(&mut self, position: Vec2) -> &mut Self;
}

impl SetAbsolutePositionUncheckedExt for UiStyleUnchecked<'_>
{
    fn absolute_position(&mut self, position: Vec2) -> &mut Self
    {
        self.commands
            .queue(SetAbsolutePosition { absolute_position: position, check_lock: false });
        self
    }
}

#[derive(Clone, Debug)]
pub enum FontSource
{
    Path(String),
    Handle(Handle<Font>),
}

impl Default for FontSource
{
    fn default() -> Self
    {
        Self::Handle(Handle::default())
    }
}

impl From<&str> for FontSource
{
    fn from(path: &str) -> Self
    {
        Self::Path(path.to_string())
    }
}

impl From<String> for FontSource
{
    fn from(path: String) -> Self
    {
        Self::Path(path)
    }
}

// TODO: Update these once font / text handling improves
impl EntityCommand for SetFont
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let font = match self.font {
            FontSource::Path(path) => world.resource::<AssetServer>().load(path),
            FontSource::Handle(handle) => handle,
        };

        let Some(mut text_font) = world.get_mut::<TextFont>(entity) else {
            warn!("Failed to set font on entity {}: No Text component found!", entity);
            return;
        };

        text_font.font = font.clone();
    }
}

impl EntityCommand for SetFontSize
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut text_font) = world.get_mut::<TextFont>(entity) else {
            warn!("Failed to set font on entity {}: No Text component found!", entity);
            return;
        };

        text_font.font_size = self.font_size;
    }
}

struct SetLockedAttribute
{
    attribute: LockableStyleAttribute,
    locked: bool,
}

impl EntityCommand for SetLockedAttribute
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if let Some(mut locked_attributes) = world.get_mut::<LockedStyleAttributes>(entity) {
            if self.locked {
                if !locked_attributes.contains(self.attribute) {
                    locked_attributes.0.insert(self.attribute);
                }
            } else {
                if locked_attributes.contains(self.attribute) {
                    locked_attributes.0.remove(&self.attribute);
                }
            }
        } else if self.locked {
            let mut locked_attributes = LockedStyleAttributes::default();
            locked_attributes.0.insert(self.attribute);
            world.entity_mut(entity).insert(locked_attributes);
        }
    }
}

pub trait SetLockedAttributeExt
{
    fn lock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self;
}

impl SetLockedAttributeExt for UiStyle<'_>
{
    fn lock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self
    {
        self.commands
            .queue(SetLockedAttribute { attribute, locked: true });
        self
    }
}

pub trait SetLockedAttributeUncheckedExt
{
    fn unlock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self;
}

impl SetLockedAttributeUncheckedExt for UiStyleUnchecked<'_>
{
    fn unlock_attribute(&mut self, attribute: LockableStyleAttribute) -> &mut Self
    {
        self.commands
            .queue(SetLockedAttribute { attribute, locked: false });
        self
    }
}

impl EntityCommand for SetScale
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(world, entity, "scale", LockableStyleAttribute::Scale);
        }

        let Some(mut transform) = world.get_mut::<Transform>(entity) else {
            warn!(
                "Failed to set scale on entity {}: No Transform component found!",
                entity
            );
            return;
        };

        let new_scale = Vec3::ONE * self.scale;
        if transform.scale != new_scale {
            transform.scale = new_scale;
        }
    }
}

impl EntityCommand for SetSize
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        if self.check_lock {
            check_lock!(world, entity, "size: width", LockableStyleAttribute::Width);
            check_lock!(world, entity, "size: height", LockableStyleAttribute::Height);
        }

        let Some(mut style) = world.get_mut::<Node>(entity) else {
            warn!("Failed to set size on entity {}: No Node component found!", entity);
            return;
        };

        if style.width != self.size {
            style.width = self.size;
        }

        if style.height != self.size {
            style.height = self.size;
        }
    }
}

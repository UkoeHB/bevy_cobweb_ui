use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_cobweb::prelude::*;

use crate::prelude::*;
use crate::sickle::Lerp;

//-------------------------------------------------------------------------------------------------------------------

fn set_border_radius_top_left(
    In((entity, radius)): In<(Entity, Val)>,
    mut c: Commands,
    mut q: Query<Option<&mut BorderRadius>>,
)
{
    let Ok(maybe_border_radius) = q.get_mut(entity) else { return };
    let Some(mut border_radius) = maybe_border_radius else {
        c.entity(entity).try_insert(BorderRadius::top_left(radius));
        return;
    };
    border_radius.top_left = radius;
}

//-------------------------------------------------------------------------------------------------------------------

fn set_border_radius_top_right(
    In((entity, radius)): In<(Entity, Val)>,
    mut c: Commands,
    mut q: Query<Option<&mut BorderRadius>>,
)
{
    let Ok(maybe_border_radius) = q.get_mut(entity) else { return };
    let Some(mut border_radius) = maybe_border_radius else {
        c.entity(entity).try_insert(BorderRadius::top_right(radius));
        return;
    };
    border_radius.top_right = radius;
}

//-------------------------------------------------------------------------------------------------------------------

fn set_border_radius_bottom_left(
    In((entity, radius)): In<(Entity, Val)>,
    mut c: Commands,
    mut q: Query<Option<&mut BorderRadius>>,
)
{
    let Ok(maybe_border_radius) = q.get_mut(entity) else { return };
    let Some(mut border_radius) = maybe_border_radius else {
        c.entity(entity)
            .try_insert(BorderRadius::bottom_left(radius));
        return;
    };
    border_radius.bottom_left = radius;
}

//-------------------------------------------------------------------------------------------------------------------

fn set_border_radius_bottom_right(
    In((entity, radius)): In<(Entity, Val)>,
    mut c: Commands,
    mut q: Query<Option<&mut BorderRadius>>,
)
{
    let Ok(maybe_border_radius) = q.get_mut(entity) else { return };
    let Some(mut border_radius) = maybe_border_radius else {
        c.entity(entity)
            .try_insert(BorderRadius::bottom_right(radius));
        return;
    };
    border_radius.bottom_right = radius;
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for BackgroundColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

impl StaticAttribute for BackgroundColor
{
    type Value = Color;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BackgroundColor {}
impl AnimatedAttribute for BackgroundColor
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let bg = world.get::<BackgroundColor>(entity).copied()?;
        Some(bg.0)
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for BorderColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

impl StaticAttribute for BorderColor
{
    type Value = Color;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BorderColor {}
impl AnimatedAttribute for BorderColor
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let br = world.get::<BorderColor>(entity).copied()?;
        Some(br.0)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set all corner radii to the same value, can be loaded as an instruction.
///
/// See [`BrRadiusTopLeft`], [`BrRadiusTopRight`], [`BrRadiusBottomLeft`], [`BrRadiusBottomRight`] to set
/// individual corners.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct BrRadius(pub Val);

impl Instruction for BrRadius
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(BorderRadius::all(self.0));
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl StaticAttribute for BrRadius
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadius {}
impl AnimatedAttribute for BrRadius
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let br = world.get::<BorderRadius>(entity).copied()?;
        Some(br.top_left)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the top left corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct BrRadiusTopLeft(pub Val);

impl Instruction for BrRadiusTopLeft
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self.0), set_border_radius_top_left);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl StaticAttribute for BrRadiusTopLeft
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusTopLeft {}
impl AnimatedAttribute for BrRadiusTopLeft
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let br = world.get::<BorderRadius>(entity).copied()?;
        Some(br.top_left)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the top right corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct BrRadiusTopRight(pub Val);

impl Instruction for BrRadiusTopRight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self.0), set_border_radius_top_right);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl StaticAttribute for BrRadiusTopRight
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusTopRight {}
impl AnimatedAttribute for BrRadiusTopRight
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let br = world.get::<BorderRadius>(entity).copied()?;
        Some(br.top_right)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the bottom left corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct BrRadiusBottomLeft(pub Val);

impl Instruction for BrRadiusBottomLeft
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self.0), set_border_radius_bottom_left);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl StaticAttribute for BrRadiusBottomLeft
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusBottomLeft {}
impl AnimatedAttribute for BrRadiusBottomLeft
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let br = world.get::<BorderRadius>(entity).copied()?;
        Some(br.bottom_left)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the bottom right corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct BrRadiusBottomRight(pub Val);

impl Instruction for BrRadiusBottomRight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self.0), set_border_radius_bottom_right);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl StaticAttribute for BrRadiusBottomRight
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusBottomRight {}
impl AnimatedAttribute for BrRadiusBottomRight
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let br = world.get::<BorderRadius>(entity).copied()?;
        Some(br.bottom_right)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Outline`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeOutline
{
    pub width: Val,
    /// Space added between the outline and the node's border edge.
    #[reflect(default)]
    pub offset: Val,
    pub color: Color,
}

impl Into<Outline> for NodeOutline
{
    fn into(self) -> Outline
    {
        Outline { width: self.width, offset: self.offset, color: self.color }
    }
}

impl From<Outline> for NodeOutline
{
    fn from(outline: Outline) -> Self
    {
        Self {
            width: outline.width,
            offset: outline.offset,
            color: outline.color,
        }
    }
}

//todo: consider separate lerps for each of the outline fields
impl Lerp for NodeOutline
{
    fn lerp(&self, to: Self, t: f32) -> Self
    {
        Self {
            width: self.width.lerp(to.width, t),
            offset: self.offset.lerp(to.offset, t),
            color: self.color.lerp(to.color, t),
        }
    }
}

impl Instruction for NodeOutline
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let outline: Outline = self.into();
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(outline);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Outline>();
        });
    }
}

impl StaticAttribute for NodeOutline
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

impl ResponsiveAttribute for NodeOutline {}
impl AnimatedAttribute for NodeOutline
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let outline = world.get::<Outline>(entity).copied()?;
        Some(outline.into())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BoxShadow`], can be loaded as an instruction.
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeShadow
{
    /// The shadow's color.
    ///
    /// Defaults to black.
    #[reflect(default = "NodeShadow::default_color")]
    pub color: Color,
    /// Horizontal offset.
    #[reflect(default)]
    pub x_offset: Val,
    /// Vertical offset
    #[reflect(default)]
    pub y_offset: Val,
    /// How much the shadow should spread outward.
    ///
    /// Negative values will make the shadow shrink inwards.
    /// Percentage values are based on the width of the UI node.
    #[reflect(default)]
    pub spread_radius: Val,
    /// Blurriness of the shadow
    #[reflect(default)]
    pub blur_radius: Val,
}

impl NodeShadow
{
    fn default_color() -> Color
    {
        Color::BLACK
    }
}

impl Default for NodeShadow
{
    fn default() -> Self
    {
        Self {
            color: NodeShadow::default_color(),
            x_offset: Default::default(),
            y_offset: Default::default(),
            spread_radius: Default::default(),
            blur_radius: Default::default(),
        }
    }
}

impl Into<BoxShadow> for NodeShadow
{
    fn into(self) -> BoxShadow
    {
        BoxShadow {
            color: self.color,
            x_offset: self.x_offset,
            y_offset: self.y_offset,
            spread_radius: self.spread_radius,
            blur_radius: self.blur_radius,
        }
    }
}

impl From<BoxShadow> for NodeShadow
{
    fn from(shadow: BoxShadow) -> Self
    {
        Self {
            color: shadow.color,
            x_offset: shadow.x_offset,
            y_offset: shadow.y_offset,
            spread_radius: shadow.spread_radius,
            blur_radius: shadow.blur_radius,
        }
    }
}

impl Lerp for NodeShadow
{
    fn lerp(&self, to: Self, t: f32) -> Self
    {
        Self {
            color: self.color.lerp(to.color, t),
            x_offset: self.x_offset.lerp(to.x_offset, t),
            y_offset: self.y_offset.lerp(to.y_offset, t),
            spread_radius: self.spread_radius.lerp(to.spread_radius, t),
            blur_radius: self.blur_radius.lerp(to.blur_radius, t),
        }
    }
}

impl Instruction for NodeShadow
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let shadow: BoxShadow = self.into();
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(shadow);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BoxShadow>();
        });
    }
}

impl StaticAttribute for NodeShadow
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

impl ResponsiveAttribute for NodeShadow {}
impl AnimatedAttribute for NodeShadow
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let shadow = world.get::<BoxShadow>(entity).copied()?;
        Some(shadow.into())
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for FocusPolicy
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let policy: FocusPolicy = self.into();
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(policy);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<FocusPolicy>();
        });
    }
}

impl StaticAttribute for FocusPolicy
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}
impl ResponsiveAttribute for FocusPolicy {}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for ZIndex
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<ZIndex>();
        });
    }
}

impl StaticAttribute for ZIndex
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}
impl ResponsiveAttribute for ZIndex {}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for GlobalZIndex
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<GlobalZIndex>();
        });
    }
}

impl StaticAttribute for GlobalZIndex
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}
impl ResponsiveAttribute for GlobalZIndex {}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for Visibility
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let visibility: Visibility = self.into();
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(visibility);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Visibility>();
        });
    }
}

impl StaticAttribute for Visibility
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}
impl ResponsiveAttribute for Visibility {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiComponentWrappersPlugin;

impl Plugin for UiComponentWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_animatable::<BackgroundColor>()
            .register_animatable::<BorderColor>()
            .register_animatable::<BrRadius>()
            .register_animatable::<BrRadiusTopLeft>()
            .register_animatable::<BrRadiusTopRight>()
            .register_animatable::<BrRadiusBottomLeft>()
            .register_animatable::<BrRadiusBottomRight>()
            .register_animatable::<NodeOutline>()
            .register_animatable::<NodeShadow>()
            .register_responsive::<FocusPolicy>()
            .register_responsive::<ZIndex>()
            .register_responsive::<GlobalZIndex>()
            .register_responsive::<Visibility>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

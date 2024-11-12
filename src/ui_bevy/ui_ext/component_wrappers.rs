use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_cobweb::prelude::*;

use crate::prelude::*;
use crate::sickle_ext::lerp::Lerp;

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
impl AnimatableAttribute for BackgroundColor {}

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
impl AnimatableAttribute for BorderColor {}

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
impl AnimatableAttribute for BrRadius {}

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
impl AnimatableAttribute for BrRadiusTopLeft {}

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
impl AnimatableAttribute for BrRadiusTopRight {}

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
impl AnimatableAttribute for BrRadiusBottomLeft {}

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
impl AnimatableAttribute for BrRadiusBottomRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Outline`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
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
impl AnimatableAttribute for NodeOutline {}

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
            .register_responsive::<FocusPolicy>()
            .register_responsive::<ZIndex>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

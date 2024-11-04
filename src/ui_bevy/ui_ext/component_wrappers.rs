use crate::sickle::lerp::Lerp;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn set_border_radius_top_left(
    In((entity, radius)): In<(Entity, Val)>,
    mut c: Commands,
    mut q: Query<Option<&mut BorderRadius>>,
) {
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
) {
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
) {
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
) {
    let Ok(maybe_border_radius) = q.get_mut(entity) else { return };
    let Some(mut border_radius) = maybe_border_radius else {
        c.entity(entity)
            .try_insert(BorderRadius::bottom_right(radius));
        return;
    };
    border_radius.bottom_right = radius;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BackgroundColor`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgColor(pub Color);

impl Instruction for BgColor {
    fn apply(self, entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(BackgroundColor(self.0));
        });
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BackgroundColor>();
        });
    }
}

impl ThemedAttribute for BgColor {
    type Value = Color;
    fn construct(value: Self::Value) -> Self {
        Self(value)
    }
}

impl ResponsiveAttribute for BgColor {}
impl AnimatableAttribute for BgColor {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderColor`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrColor(pub Color);

impl Instruction for BrColor {
    fn apply(self, entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(BorderColor(self.0));
        });
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderColor>();
        });
    }
}

impl ThemedAttribute for BrColor {
    type Value = Color;
    fn construct(value: Self::Value) -> Self {
        Self(value)
    }
}

impl ResponsiveAttribute for BrColor {}
impl AnimatableAttribute for BrColor {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set all corner radii to the same value, can be loaded as an instruction.
///
/// See [`BrRadiusTopLeft`], [`BrRadiusTopRight`], [`BrRadiusBottomLeft`], [`BrRadiusBottomRight`] to set
/// individual corners.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadius(pub Val);

impl Instruction for BrRadius {
    fn apply(self, entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(BorderRadius::all(self.0));
        });
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl ThemedAttribute for BrRadius {
    type Value = Val;
    fn construct(value: Self::Value) -> Self {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadius {}
impl AnimatableAttribute for BrRadius {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the top left corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusTopLeft(pub Val);

impl Instruction for BrRadiusTopLeft {
    fn apply(self, entity: Entity, world: &mut World) {
        world.syscall((entity, self.0), set_border_radius_top_left);
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl ThemedAttribute for BrRadiusTopLeft {
    type Value = Val;
    fn construct(value: Self::Value) -> Self {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusTopLeft {}
impl AnimatableAttribute for BrRadiusTopLeft {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the top right corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusTopRight(pub Val);

impl Instruction for BrRadiusTopRight {
    fn apply(self, entity: Entity, world: &mut World) {
        world.syscall((entity, self.0), set_border_radius_top_right);
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl ThemedAttribute for BrRadiusTopRight {
    type Value = Val;
    fn construct(value: Self::Value) -> Self {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusTopRight {}
impl AnimatableAttribute for BrRadiusTopRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the bottom left corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusBottomLeft(pub Val);

impl Instruction for BrRadiusBottomLeft {
    fn apply(self, entity: Entity, world: &mut World) {
        world.syscall((entity, self.0), set_border_radius_bottom_left);
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl ThemedAttribute for BrRadiusBottomLeft {
    type Value = Val;
    fn construct(value: Self::Value) -> Self {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusBottomLeft {}
impl AnimatableAttribute for BrRadiusBottomLeft {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the bottom right corner radius, can be loaded as an instruction.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusBottomRight(pub Val);

impl Instruction for BrRadiusBottomRight {
    fn apply(self, entity: Entity, world: &mut World) {
        world.syscall((entity, self.0), set_border_radius_bottom_right);
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BorderRadius>();
        });
    }
}

impl ThemedAttribute for BrRadiusBottomRight {
    type Value = Val;
    fn construct(value: Self::Value) -> Self {
        Self(value)
    }
}

impl ResponsiveAttribute for BrRadiusBottomRight {}
impl AnimatableAttribute for BrRadiusBottomRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Outline`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeOutline {
    pub width: Val,
    /// Space added between the outline and the node's border edge.
    #[reflect(default)]
    pub offset: Val,
    pub color: Color,
}

impl Into<Outline> for NodeOutline {
    fn into(self) -> Outline {
        Outline { width: self.width, offset: self.offset, color: self.color }
    }
}

//todo: consider separate lerps for each of the outline fields
impl Lerp for NodeOutline {
    fn lerp(&self, to: Self, t: f32) -> Self {
        Self {
            width: self.width.lerp(to.width, t),
            offset: self.offset.lerp(to.offset, t),
            color: self.color.lerp(to.color, t),
        }
    }
}

impl Instruction for NodeOutline {
    fn apply(self, entity: Entity, world: &mut World) {
        let outline: Outline = self.into();
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(outline);
        });
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Outline>();
        });
    }
}

impl ThemedAttribute for NodeOutline {
    type Value = Self;
    fn construct(value: Self::Value) -> Self {
        value
    }
}

impl ResponsiveAttribute for NodeOutline {}
impl AnimatableAttribute for NodeOutline {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`FocusPolicy`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SetFocusPolicy {
    Block,
    #[default]
    Pass,
}

impl Into<FocusPolicy> for SetFocusPolicy {
    fn into(self) -> FocusPolicy {
        match self {
            Self::Block => FocusPolicy::Block,
            Self::Pass => FocusPolicy::Pass,
        }
    }
}

impl Instruction for SetFocusPolicy {
    fn apply(self, entity: Entity, world: &mut World) {
        let policy: FocusPolicy = self.into();
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(policy);
        });
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<FocusPolicy>();
        });
    }
}

impl ThemedAttribute for SetFocusPolicy {
    type Value = Self;
    fn construct(value: Self::Value) -> Self {
        value
    }
}
impl ResponsiveAttribute for SetFocusPolicy {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ZIndex`], can be loaded as an instruction.
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SetZIndex {
    Local(i32),
    Global(i32),
}

impl Default for SetZIndex {
    fn default() -> Self {
        Self::Local(0)
    }
}

impl Into<ZIndex> for SetZIndex {
    fn into(self) -> ZIndex {
        match self {
            Self::Local(i) => ZIndex::Local(i),
            Self::Global(i) => ZIndex::Global(i),
        }
    }
}

impl Instruction for SetZIndex {
    fn apply(self, entity: Entity, world: &mut World) {
        let z: ZIndex = self.into();
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(z);
        });
    }

    fn revert(entity: Entity, world: &mut World) {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<ZIndex>();
        });
    }
}

impl ThemedAttribute for SetZIndex {
    type Value = Self;
    fn construct(value: Self::Value) -> Self {
        value
    }
}
impl ResponsiveAttribute for SetZIndex {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiComponentWrappersPlugin;

impl Plugin for UiComponentWrappersPlugin {
    fn build(&self, app: &mut App) {
        app.register_animatable::<BgColor>()
            .register_animatable::<BrColor>()
            .register_animatable::<BrRadius>()
            .register_animatable::<BrRadiusTopLeft>()
            .register_animatable::<BrRadiusTopRight>()
            .register_animatable::<BrRadiusBottomLeft>()
            .register_animatable::<BrRadiusBottomRight>()
            .register_animatable::<NodeOutline>()
            .register_responsive::<SetFocusPolicy>()
            .register_responsive::<SetZIndex>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

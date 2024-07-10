use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::lerp::Lerp;

use crate::prelude::*;

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

/// Mirrors [`BackgroundColor`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgColor(pub Color);

impl ApplyLoadable for BgColor
{
    fn apply(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BackgroundColor(self.0));
    }
}

impl ThemedAttribute for BgColor
{
    type Value = Color;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BgColor {}
impl AnimatableAttribute for BgColor {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderColor`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrColor(pub Color);

impl ApplyLoadable for BrColor
{
    fn apply(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BorderColor(self.0));
    }
}

impl ThemedAttribute for BrColor
{
    type Value = Color;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BrColor {}
impl AnimatableAttribute for BrColor {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set all corner radii to the same value, can be loaded as a style.
///
/// See [`BrRadiusTopLeft`], [`BrRadiusTopRight`], [`BrRadiusBottomLeft`], [`BrRadiusBottomRight`] to set
/// individual corners.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadius(pub Val);

impl ApplyLoadable for BrRadius
{
    fn apply(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BorderRadius::all(self.0));
    }
}

impl ThemedAttribute for BrRadius
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BrRadius {}
impl AnimatableAttribute for BrRadius {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the top left corner radius, can be loaded as a style.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusTopLeft(pub Val);

impl ApplyLoadable for BrRadiusTopLeft
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.syscall((id, self.0), set_border_radius_top_left);
    }
}

impl ThemedAttribute for BrRadiusTopLeft
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BrRadiusTopLeft {}
impl AnimatableAttribute for BrRadiusTopLeft {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the top right corner radius, can be loaded as a style.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusTopRight(pub Val);

impl ApplyLoadable for BrRadiusTopRight
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.syscall((id, self.0), set_border_radius_top_right);
    }
}

impl ThemedAttribute for BrRadiusTopRight
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BrRadiusTopRight {}
impl AnimatableAttribute for BrRadiusTopRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the bottom left corner radius, can be loaded as a style.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusBottomLeft(pub Val);

impl ApplyLoadable for BrRadiusBottomLeft
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.syscall((id, self.0), set_border_radius_bottom_left);
    }
}

impl ThemedAttribute for BrRadiusBottomLeft
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BrRadiusBottomLeft {}
impl AnimatableAttribute for BrRadiusBottomLeft {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderRadius`] to set the bottom right corner radius, can be loaded as a style.
///
/// See [`BrRadius`] to set all corners at once.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrRadiusBottomRight(pub Val);

impl ApplyLoadable for BrRadiusBottomRight
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.syscall((id, self.0), set_border_radius_bottom_right);
    }
}

impl ThemedAttribute for BrRadiusBottomRight
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BrRadiusBottomRight {}
impl AnimatableAttribute for BrRadiusBottomRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Outline`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl ApplyLoadable for NodeOutline
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let outline: Outline = self.into();
        ec.try_insert(outline);
    }
}

impl ThemedAttribute for NodeOutline
{
    type Value = Self;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        value.apply(ec);
    }
}

impl ResponsiveAttribute for NodeOutline {}
impl AnimatableAttribute for NodeOutline {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`FocusPolicy`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SetFocusPolicy
{
    Block,
    #[default]
    Pass,
}

impl Into<FocusPolicy> for SetFocusPolicy
{
    fn into(self) -> FocusPolicy
    {
        match self {
            Self::Block => FocusPolicy::Block,
            Self::Pass => FocusPolicy::Pass,
        }
    }
}

impl ApplyLoadable for SetFocusPolicy
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let policy: FocusPolicy = self.into();
        ec.try_insert(policy);
    }
}

impl ThemedAttribute for SetFocusPolicy
{
    type Value = Self;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        value.apply(ec);
    }
}
impl ResponsiveAttribute for SetFocusPolicy {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ZIndex`], can be loaded as a style.
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SetZIndex
{
    Local(i32),
    Global(i32),
}

impl Default for SetZIndex
{
    fn default() -> Self
    {
        Self::Local(0)
    }
}

impl Into<ZIndex> for SetZIndex
{
    fn into(self) -> ZIndex
    {
        match self {
            Self::Local(i) => ZIndex::Local(i),
            Self::Global(i) => ZIndex::Global(i),
        }
    }
}

impl ApplyLoadable for SetZIndex
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let z: ZIndex = self.into();
        ec.try_insert(z);
    }
}

impl ThemedAttribute for SetZIndex
{
    type Value = Self;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        value.apply(ec);
    }
}
impl ResponsiveAttribute for SetZIndex {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiComponentWrappersPlugin;

impl Plugin for UiComponentWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
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

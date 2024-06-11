use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use serde::{Deserialize, Serialize};
use sickle_ui::lerp::Lerp;

use crate::*;

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
        BgColor(value).apply(ec);
    }
}

impl ResponsiveAttribute for BgColor
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for BgColor
{
    type Interactive = Interactive;
}

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
        BrColor(value).apply(ec);
    }
}

impl ResponsiveAttribute for BrColor
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for BrColor
{
    type Interactive = Interactive;
}

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

impl ResponsiveAttribute for NodeOutline
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for NodeOutline
{
    type Interactive = Interactive;
}

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
impl ResponsiveAttribute for SetFocusPolicy
{
    type Interactive = Interactive;
}

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
impl ResponsiveAttribute for SetZIndex
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiComponentWrappersPlugin;

impl Plugin for UiComponentWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_animatable::<BgColor>()
            .register_animatable::<BrColor>()
            .register_animatable::<NodeOutline>()
            .register_responsive::<SetFocusPolicy>()
            .register_responsive::<SetZIndex>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

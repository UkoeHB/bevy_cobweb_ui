use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BackgroundColor`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgColor(pub Color);

impl ApplyLoadable for BgColor
{
    /// Converts to a [`BackgroundColor`].
    fn apply(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BackgroundColor(self.0.clone()));
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
    /// Converts to a [`BorderColor`].
    fn apply(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BorderColor(self.0.clone()));
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

//TODO: FocusPolicy

//-------------------------------------------------------------------------------------------------------------------

//TODO: ZIndex

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiComponentWrappersPlugin;

impl Plugin for UiComponentWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_animatable::<BgColor>().register_animatable::<BrColor>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

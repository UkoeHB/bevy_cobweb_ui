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

/*
impl Animatable for BgColor
{
    type Value = Color;
    type Interaction = Interaction;

    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        BgColor(value).apply(ec);
    }
}
*/

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

//-------------------------------------------------------------------------------------------------------------------

//TODO: FocusPolicy

//-------------------------------------------------------------------------------------------------------------------

//TODO: ZIndex

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiComponentsExtPlugin;

impl Plugin for UiComponentsExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<BgColor>()
            .register_type::<BrColor>()
            .register_derived_loadable::<BgColor>()
            .register_derived_loadable::<BrColor>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

use crate::*;

use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Serialize, Deserialize};

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BackgroundColor`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgColor(pub Color);

impl StyleToBevy for BgColor
{
    /// Converts to a [`BackgroundColor`].
    fn to_bevy(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BackgroundColor(self.0.clone()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BorderColor`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrColor(pub Color);

impl StyleToBevy for BrColor
{
    /// Converts to a [`BorderColor`].
    fn to_bevy(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BorderColor(self.0.clone()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiComponentsExtPlugin;

impl Plugin for UiComponentsExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<BgColor>()
            .register_type::<BrColor>()
            .register_derived_style::<BgColor>()
            .register_derived_style::<BrColor>()
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

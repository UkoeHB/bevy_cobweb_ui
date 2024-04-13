use crate::*;

use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Serialize, Deserialize};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn background_color_converter(color: BgColor, cmds: &mut EntityCommands)
{
    cmds.insert(color.to_bevy());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`BackgroundColor`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgColor(Color);

impl BgColor
{
    /// Converts to a [`BackgroundColor`].
    pub fn to_bevy(&self) -> BackgroundColor
    {
        BackgroundColor(self.0.clone())
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
            .register_derived_style::<BgColor>(background_color_converter)
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

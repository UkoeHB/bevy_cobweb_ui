use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct StyleExtPlugin;

impl Plugin for StyleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(StyleWrappersPlugin)
            .add_plugins(UiComponentWrappersPlugin)
            .add_plugins(UiStyleFieldWrappersPlugin)
            .add_plugins(UiTextExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

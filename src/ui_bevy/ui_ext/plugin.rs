use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct StyleExtPlugin;

impl Plugin for StyleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(StyleWrappersPlugin)
            // IMPORTANT: These plugins must be added after StyleWrappersPlugin so the loadables defined here will
            // overwrite style fields correctly.
            .add_plugins(UiComponentWrappersPlugin)
            // IMPORTANT: These plugins must be added after StyleWrappersPlugin so the loadables defined here will
            // overwrite style fields correctly.
            .add_plugins(UiStyleFieldWrappersPlugin)
            .add_plugins(UiTextExtPlugin)
            .add_plugins(UiImageExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

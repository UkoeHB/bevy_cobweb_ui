use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(UiInteractionExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

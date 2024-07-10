use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebBevyUiPlugin;

impl Plugin for CobwebBevyUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(StyleExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

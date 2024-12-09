use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ToolsPlugin;

impl Plugin for ToolsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<IterChildren>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

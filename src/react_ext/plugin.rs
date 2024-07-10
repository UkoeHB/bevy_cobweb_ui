use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ReactExtPlugin;

impl Plugin for ReactExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(ReactorExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use crate::*;

use bevy::prelude::*;


//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ParentsPlugin;

impl Plugin for ParentsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(Camera2DPlugin)
            .add_plugins(ParentPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

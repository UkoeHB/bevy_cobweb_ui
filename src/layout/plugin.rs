//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LayoutPlugin;

impl Plugin for LayoutPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(DimsPlugin)
            .add_plugins(PositionPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

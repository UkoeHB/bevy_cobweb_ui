use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct Cobweb2dUiPlugin;

impl Plugin for Cobweb2dUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .add_plugins(PrimitivesPlugin)
            .add_plugins(ParentsPlugin)
            .add_plugins(LayoutPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

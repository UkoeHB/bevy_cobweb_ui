use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct Cobweb2DUiPlugin;

impl Plugin for Cobweb2DUiPlugin
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

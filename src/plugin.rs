//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(ReactPlugin)
            .add_plugins(AppEventsPlugin)
            .add_plugins(StylePlugin)
            .add_plugins(StyleSheetPlugin)
            .add_plugins(ParentsPlugin)
            .add_plugins(LayoutPlugin)
            .add_plugins(PrimitivesPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

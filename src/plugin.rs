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
        app.add_plugins(ReactPlugin);
        app.add_plugins(AppEventsPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

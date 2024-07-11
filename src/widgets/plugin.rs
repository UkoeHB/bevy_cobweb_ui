use bevy::prelude::*;

use crate::widgets::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebWidgetsPlugin;

impl Plugin for CobwebWidgetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(radio_button::CobwebRadioButtonPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

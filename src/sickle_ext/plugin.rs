use crate::*;

use bevy::prelude::*;
use sickle_ui::FluxInteractionUpdate;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, flux_ui_events.after(FluxInteractionUpdate));
    }
}

//-------------------------------------------------------------------------------------------------------------------

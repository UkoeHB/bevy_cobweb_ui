use bevy::asset::embedded_asset;
use bevy::prelude::*;

use crate::prelude::*;
use crate::widgets::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebWidgetsPlugin;

impl Plugin for CobwebWidgetsPlugin
{
    fn build(&self, app: &mut App)
    {
        embedded_asset!(app, "src/widgets/", "manifest.caf.json");

        app.add_plugins(radio_buttons::CobwebRadioButtonsPlugin)
            .load("embedded://bevy_cobweb_ui/manifest.caf.json");
    }
}

//-------------------------------------------------------------------------------------------------------------------

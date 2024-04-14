use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(ReactPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(StyleExtPlugin)
            .add_plugins(SickleExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BevyExtPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(ReactPlugin)
            .add_plugins(SickleExtPlugin)
            .add_plugins(CobwebBevyUiPlugin)
            //.add_plugins(Cobweb2DUiPlugin)
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

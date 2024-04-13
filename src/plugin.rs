use crate::*;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;


//-------------------------------------------------------------------------------------------------------------------

pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(ReactPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(StyleExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Depends on [`bevy_cobweb::prelude::ReactPlugin`] and [`sickle_ui::prelude::SickleUiPlugin`].
pub struct CobwebUiPlugin;

impl Plugin for CobwebUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(BevyExtPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(LocalizationPlugin)
            .add_plugins(SickleExtPlugin)
            .add_plugins(AssetsExtPlugin)
            .add_plugins(CobwebBevyUiPlugin)
            //.add_plugins(Cobweb2DUiPlugin)
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

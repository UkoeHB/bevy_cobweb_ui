use bevy::prelude::*;

use super::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobEditorPlugin;

impl Plugin for CobEditorPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(CobWidgetRegistryPlugin)
            .add_plugins(CobHashRegistryPlugin)
            .add_plugins(CobEditorImplPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

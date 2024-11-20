use bevy::prelude::*;

use crate::prelude::*;
use crate::sickle::{Ease, *};

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins((FluxInteractionPlugin, PseudoStatePlugin, DynamicStylePlugin));
        app.register_type::<Ease>()
            .register_type::<PseudoState>()
            .register_type::<AnimationSettings>()
            .register_type::<AnimationConfig>()
            .register_type::<AnimationLoop>()
            .add_plugins(ControlPlugin)
            .add_plugins(ControlMapPlugin)
            .add_plugins(UiInteractionExtPlugin)
            .add_plugins(PseudoStatesExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

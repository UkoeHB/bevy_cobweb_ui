use attributes::dynamic_style::DynamicStylePlugin;
use attributes::pseudo_state::PseudoStatePlugin;
use bevy::prelude::*;
use flux_interaction::FluxInteractionPlugin;

use crate::prelude::*;
use crate::sickle_ext::attributes::pseudo_state::PseudoState;
use crate::sickle_ext::attributes::style_animation::{AnimationConfig, AnimationLoop, AnimationSettings};
use crate::sickle_ext::ease::Ease;

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

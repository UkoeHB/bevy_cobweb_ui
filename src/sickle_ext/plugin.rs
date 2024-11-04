use bevy::prelude::*;

use crate::prelude::*;
use crate::sickle_ext::ease::Ease;
use crate::sickle_ext::theme::pseudo_state::PseudoState;
use crate::sickle_ext::theme::style_animation::{AnimationConfig, AnimationLoop, AnimationSettings};
use bevy::prelude::*;
use flux_interaction::FluxInteractionPlugin;
use theme::ThemePlugin;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins((
            FluxInteractionPlugin,
            ThemePlugin,
        ));
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

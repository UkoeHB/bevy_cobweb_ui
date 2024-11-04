use crate::sickle::ease::Ease;
use crate::sickle::theme::pseudo_state::PseudoState;
use crate::sickle::theme::style_animation::{AnimationConfig, AnimationLoop, AnimationSettings};
use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Ease>()
            .register_type::<PseudoState>()
            .register_type::<AnimationSettings>()
            .register_type::<AnimationConfig>()
            .register_type::<AnimationLoop>()
            .add_plugins(SickleUiDefaultAssetsPlugin)
            .add_plugins(ControlPlugin)
            .add_plugins(ControlMapPlugin)
            .add_plugins(UiInteractionExtPlugin)
            .add_plugins(PseudoStatesExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

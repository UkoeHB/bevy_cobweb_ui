use bevy::prelude::*;
use sickle_ui::ease::Ease;
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::style_animation::{AnimationConfig, AnimationLoop, AnimationSettings};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Ease>()
            .register_type::<PseudoState>()
            .register_type::<AnimationSettings>()
            .register_type::<AnimationConfig>()
            .register_type::<AnimationLoop>()
            .add_plugins(SickleUiDefaultAssetsPlugin)
            .add_plugins(LoadedThemesPlugin)
            .add_plugins(UiInteractionExtPlugin)
            .add_plugins(PseudoStatesExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

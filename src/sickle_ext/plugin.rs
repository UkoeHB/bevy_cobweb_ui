use bevy::prelude::*;
use sickle_ui::ease::Ease;
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::style_animation::{AnimationConfig, AnimationLoop, AnimationSettings};
use sickle_ui::ui_style::AnimatedVals;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Option<Val>>()
            .register_type::<Ease>()
            .register_type::<Option<Ease>>()
            .register_type::<PseudoState>()
            .register_type::<AnimationSettings>()
            .register_type::<AnimationConfig>()
            .register_type::<Option<AnimationConfig>>()
            .register_type::<AnimationLoop>()
            .register_type::<Option<AnimationLoop>>()
            .register_type::<AnimatedVals<f32>>()
            .register_type::<AnimatedVals<Color>>()
            .register_type::<AnimatedVals<Val>>()
            .register_type::<AnimatedVals<UiRect>>()
            .add_plugins(UiInteractionExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

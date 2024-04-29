use bevy::prelude::*;
use sickle_ui::{ease::Ease, theme::{pseudo_state::PseudoState, style_animation::AnimationSettings}, ui_style::AnimatedVals};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<Ease>()
            .register_type::<PseudoState>()
            .register_type::<AnimationSettings>()
            .register_type::<AnimatedVals<f32>>()
            .register_type::<AnimatedVals<Color>>()
            .register_type::<AnimatedVals<Val>>()
            .register_type::<AnimatedVals<UiRect>>()
            .add_plugins(UiInteractionExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

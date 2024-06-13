use bevy::prelude::*;
use sickle_ui::ease::Ease;
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::style_animation::{AnimationConfig, AnimationLoop, AnimationSettings};
use sickle_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SickleExtPlugin;

impl Plugin for SickleExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Option<Val>>()
            .register_type::<Option<Color>>()
            .register_type::<Ease>()
            .register_type::<Option<Ease>>()
            .register_type::<PseudoState>()
            .register_type::<Vec<PseudoState>>()
            .register_type::<Option<Vec<PseudoState>>>()
            .register_type::<AnimationSettings>()
            .register_type::<AnimationConfig>()
            .register_type::<Option<AnimationConfig>>()
            .register_type::<AnimationLoop>()
            .register_type::<Option<AnimationLoop>>()
            .register_type::<InteractiveVals<f32>>()
            .register_type::<InteractiveVals<Color>>()
            .register_type::<InteractiveVals<Val>>()
            .register_type::<InteractiveVals<UiRect>>()
            .register_type::<InteractiveVals<StyleRect>>()
            .register_type::<AnimatedVals<f32>>()
            .register_type::<AnimatedVals<Color>>()
            .register_type::<AnimatedVals<Val>>()
            .register_type::<AnimatedVals<UiRect>>()
            .register_type::<AnimatedVals<StyleRect>>()
            .add_plugins(UiInteractionExtPlugin)
            .add_plugins(PseudoStatesExtPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

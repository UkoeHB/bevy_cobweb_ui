use bevy::prelude::*;
use sickle_math::ease::Ease;

use crate::ui_style::builder::StyleBuilder;

use super::{
    icons::Icons,
    style_animation::AnimationSettings,
    theme_colors::{SchemeColors, ThemeColors},
    theme_spacing::ThemeSpacing,
    typography::ThemeTypography,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum Contrast {
    #[default]
    Standard,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum Scheme {
    Light(Contrast),
    Dark(Contrast),
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Dark(Default::default())
    }
}

impl Scheme {
    pub fn is_light(&self) -> bool {
        matches!(self, Scheme::Light(_))
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Scheme::Dark(_))
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
pub struct ThemeData {
    pub active_scheme: Scheme,
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub text: ThemeTypography,
    pub icons: Icons,
    pub interaction_animation: AnimationSettings,
    pub delayed_interaction_animation: AnimationSettings,
    pub enter_animation: AnimationSettings,
}

impl Default for ThemeData {
    fn default() -> Self {
        let mut interaction_animation = AnimationSettings::new();
        interaction_animation
            .pointer_enter(0.1, Ease::OutExpo, None)
            .pointer_leave(0.1, Ease::OutExpo, None)
            .press(0.1, Ease::OutExpo, None);

        let mut delayed_interaction_animation = AnimationSettings::new();
        delayed_interaction_animation
            .pointer_enter(0.1, Ease::OutExpo, 0.1)
            .pointer_leave(0.1, Ease::OutExpo, 0.1)
            .press(0.1, Ease::OutExpo, None);

        let mut enter_animation = AnimationSettings::new();
        enter_animation
            .enter(0.1, Ease::OutExpo, None)
            .delete_on_entered(true);

        Self {
            active_scheme: Default::default(),
            colors: Default::default(),
            spacing: Default::default(),
            text: Default::default(),
            icons: Default::default(),
            interaction_animation,
            delayed_interaction_animation,
            enter_animation,
        }
    }
}

impl ThemeData {
    pub fn with_default(builder: fn(&mut StyleBuilder, &ThemeData)) -> StyleBuilder {
        let theme_data = ThemeData::default();
        let mut style_builder = StyleBuilder::new();
        builder(&mut style_builder, &theme_data);

        style_builder
    }

    /// Returns the scheme colors of the current active scheme / contrast
    pub fn colors(&self) -> SchemeColors {
        match self.active_scheme {
            Scheme::Light(contrast) => self.colors.schemes.light_contrast(contrast),
            Scheme::Dark(contrast) => self.colors.schemes.dark_contrast(contrast),
        }
    }
}

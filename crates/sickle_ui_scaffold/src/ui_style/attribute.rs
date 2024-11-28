use std::fmt::Debug;

use bevy::prelude::*;
use cob_sickle_math::Lerp;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct InteractiveVals<T: Clone + Default>
{
    pub idle: T,
    #[reflect(default)]
    pub hover: Option<T>,
    #[reflect(default)]
    pub press: Option<T>,
    #[reflect(default)]
    pub cancel: Option<T>,
}

impl<T: Default + Clone> From<T> for InteractiveVals<T>
{
    fn from(value: T) -> Self
    {
        InteractiveVals::new(value)
    }
}

impl<T: Default + Clone + PartialEq> PartialEq for InteractiveVals<T>
{
    fn eq(&self, other: &Self) -> bool
    {
        self.idle == other.idle
            && self.hover == other.hover
            && self.press == other.press
            && self.cancel == other.cancel
    }
}

impl<T: Clone + Default> InteractiveVals<T>
{
    pub fn new(value: T) -> Self
    {
        InteractiveVals { idle: value, ..default() }
    }

    pub fn hover(self, value: T) -> Self
    {
        Self { hover: value.into(), ..self }
    }

    pub fn press(self, value: T) -> Self
    {
        Self { press: value.into(), ..self }
    }

    pub fn cancel(self, value: T) -> Self
    {
        Self { cancel: value.into(), ..self }
    }

    pub fn to_value(&self, flux_interaction: FluxInteraction) -> T
    {
        match flux_interaction {
            FluxInteraction::None => self.idle.clone(),
            FluxInteraction::PointerEnter => self.hover.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::PointerLeave => self.idle.clone(),
            FluxInteraction::Pressed => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            FluxInteraction::Released => self.hover.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::PressCanceled => self.cancel.clone().unwrap_or(self.idle.clone()),
            FluxInteraction::Disabled => self.idle.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnimatedVals<T: Lerp + Default + Clone + PartialEq>
{
    #[reflect(default)]
    pub enter_ref: Option<T>,
    /// Required.
    pub idle: T,
    #[reflect(default)]
    pub hover: Option<T>,
    #[reflect(default)]
    pub press: Option<T>,
    #[reflect(default)]
    pub cancel: Option<T>,
    #[reflect(default)]
    pub idle_secondary: Option<T>,
    #[reflect(default)]
    pub hover_secondary: Option<T>,
    #[reflect(default)]
    pub press_secondary: Option<T>,
}

impl<T: Lerp + Default + Clone + PartialEq> From<T> for AnimatedVals<T>
{
    fn from(value: T) -> Self
    {
        AnimatedVals { idle: value, ..default() }
    }
}

impl<T: Lerp + Default + Clone + PartialEq> From<InteractiveVals<T>> for AnimatedVals<T>
{
    fn from(value: InteractiveVals<T>) -> Self
    {
        Self {
            idle: value.idle,
            hover: value.hover,
            press: value.press,
            cancel: value.cancel,
            ..default()
        }
    }
}

impl<T: Lerp + Default + Clone + PartialEq> AnimatedVals<T>
{
    pub fn interaction_style(&self, interaction: InteractionStyle) -> T
    {
        match interaction {
            InteractionStyle::Idle => self.idle.clone(),
            InteractionStyle::Hover => self.hover.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::Press => self
                .press
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::Cancel => self.cancel.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::IdleAlt => self
                .idle_secondary
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::HoverAlt => self.hover_secondary.clone().unwrap_or(self.idle.clone()),
            InteractionStyle::PressAlt => self
                .press_secondary
                .clone()
                .unwrap_or(self.hover.clone().unwrap_or(self.idle.clone())),
            InteractionStyle::Enter => self.enter_ref.clone().unwrap_or(self.idle.clone()),
        }
    }

    pub fn to_value(&self, current_state: &AnimationState) -> T
    {
        current_state.extract(&self)
    }
}

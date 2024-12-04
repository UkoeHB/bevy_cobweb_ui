use bevy::utils::default;

use crate::*;

#[derive(Clone, Debug)]
pub enum DynamicStyleAttribute
{
    // Remove on apply
    Static(StaticStyleAttribute),

    // Needs flux
    Responsive(ResponsiveStyleAttribute),

    // Needs stopwatch
    // None animations are effectively Pop
    // Only Lerp properties
    Animated
    {
        attribute: AnimatedStyleAttribute,
        controller: DynamicStyleController,
    },
}

impl LogicalEq for DynamicStyleAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        match (self, other) {
            (Self::Static(l0), Self::Static(r0)) => l0.logical_eq(r0),
            (Self::Static(_), Self::Responsive(_)) => false,
            (Self::Static(_), Self::Animated { .. }) => false,
            (Self::Responsive(l0), Self::Responsive(r0)) => l0.logical_eq(r0),
            (Self::Responsive(_), Self::Static(_)) => false,
            (Self::Responsive(_), Self::Animated { .. }) => false,
            (Self::Animated { attribute: l_attribute, .. }, Self::Animated { attribute: r_attribute, .. }) => {
                l_attribute.logical_eq(r_attribute)
            }
            (Self::Animated { .. }, Self::Static(_)) => false,
            (Self::Animated { .. }, Self::Responsive(_)) => false,
        }
    }
}

impl DynamicStyleAttribute
{
    pub fn is_static(&self) -> bool
    {
        match self {
            DynamicStyleAttribute::Static(_) => true,
            _ => false,
        }
    }

    pub fn is_responsive(&self) -> bool
    {
        match self {
            DynamicStyleAttribute::Responsive(_) => true,
            _ => false,
        }
    }

    pub fn is_animated(&self) -> bool
    {
        matches!(self, DynamicStyleAttribute::Animated { .. })
    }

    pub fn controller(&self) -> Result<&DynamicStyleController, &'static str>
    {
        let DynamicStyleAttribute::Animated { ref controller, .. } = self else {
            return Err("DynamicStyleAttribute isn't animated!");
        };

        Ok(controller)
    }

    pub fn controller_mut(&mut self) -> Result<&mut DynamicStyleController, &'static str>
    {
        let DynamicStyleAttribute::Animated { ref mut controller, .. } = self else {
            return Err("DynamicStyleAttribute isn't animated!");
        };

        Ok(controller)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EnterState
{
    Unspecified,
    Entering,
    Entered,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicStyleController
{
    pub animation: AnimationSettings,
    current_state: AnimationState,
    dirty: bool,
    enter_state: EnterState,
    /// True the first tick entering state changed from Unspecified to Entering.
    just_started_entering: bool,
}

impl Default for DynamicStyleController
{
    fn default() -> Self
    {
        Self {
            animation: Default::default(),
            current_state: Default::default(),
            dirty: Default::default(),
            enter_state: EnterState::Unspecified,
            just_started_entering: false,
        }
    }
}

impl DynamicStyleController
{
    pub fn new(animation: AnimationSettings, starting_state: AnimationState) -> Self
    {
        Self { animation, current_state: starting_state, ..default() }
    }

    pub fn update(&mut self, flux_interaction: &FluxInteraction, mut elapsed: f32)
    {
        // TODO: `enter` animation is currently played when a style animation different from
        // the previous one is requested. This means that playing the enter animation is *contextual*
        // and cannot be directly controlled by the developer. Figure out a way to factor out these
        // for explicit control.

        // Pre-check if we should tick an enter frame
        let entering = match self.animation.enter_idle_with.is_some() {
            true => self.enter_state,
            false => EnterState::Entered,
        };

        let mut target_style: InteractionStyle = match entering {
            EnterState::Unspecified | EnterState::Entering => InteractionStyle::Idle,
            EnterState::Entered => flux_interaction.into(),
        };
        // In case a button switches it's own dynamic style and the new style has an enter animation
        // The animation will be played, HOWEVER it's flux will still be "Released". This means that the
        // Idle -> Hover animation would be skipped if there is no Release tween set (otherwise the Release
        // tween will be used instead of the PointerEnter).
        let mut tween = match entering {
            EnterState::Unspecified | EnterState::Entering => self.animation.enter_idle_with.clone(),
            EnterState::Entered => self.animation.to_tween(&flux_interaction),
        };
        let loop_tween = match entering {
            EnterState::Unspecified | EnterState::Entering => None,
            EnterState::Entered => self.animation.to_loop_tween(&flux_interaction),
        };

        // This only activates post-entering.
        if target_style == InteractionStyle::Cancel {
            if let Some(cancel_tween) = tween.clone() {
                let cancel_tween_length = cancel_tween.duration + cancel_tween.delay;

                if elapsed >= cancel_tween_length {
                    target_style = InteractionStyle::Idle;
                    tween = self.animation.cancel_end_with.clone();
                    elapsed -= cancel_tween_length;
                }
            } else {
                target_style = InteractionStyle::Idle;
                tween = self.animation.cancel_end_with.clone();
            }
        }

        let new_state = self
            .current_state
            .tick(target_style, tween.clone(), loop_tween, elapsed);

        // Update entering state post tick, to allow Hold to occur
        self.just_started_entering = false;
        self.enter_state = match self.animation.enter_idle_with.clone() {
            Some(tween) => {
                if elapsed < (tween.duration + tween.delay) {
                    if matches!(self.enter_state, EnterState::Unspecified) {
                        self.just_started_entering = true;
                    }
                    EnterState::Entering
                } else {
                    EnterState::Entered
                }
            }
            None => EnterState::Entered,
        };

        self.dirty = new_state != self.current_state;
        if self.dirty {
            self.current_state = new_state;
        }
    }

    pub fn current_state(&self) -> &AnimationState
    {
        &self.current_state
    }

    pub fn dirty(&self) -> bool
    {
        self.dirty
    }

    pub fn enter_state(&self) -> EnterState
    {
        self.enter_state
    }

    pub fn is_entered(&self) -> bool
    {
        self.enter_state == EnterState::Entered
    }

    /// Returns `true` after the first time the controller is updated if the controller has not yet fully entered.
    pub fn just_started_entering(&self) -> bool
    {
        self.just_started_entering
    }

    pub fn copy_state_from(&mut self, other: &DynamicStyleController)
    {
        self.current_state = other.current_state().clone();
        self.enter_state = other.enter_state;
        self.dirty = other.dirty;
    }
}

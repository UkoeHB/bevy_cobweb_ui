use std::time::Duration;

use bevy::prelude::*;
use cob_sickle_math::{Ease, Lerp, ValueEasing};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InteractionStyle
{
    #[default]
    Enter,
    Idle,
    Hover,
    Press,
    Cancel,
    IdleAlt,
    HoverAlt,
    PressAlt,
}

impl From<FluxInteraction> for InteractionStyle
{
    fn from(value: FluxInteraction) -> Self
    {
        match value {
            FluxInteraction::None => Self::Idle,
            FluxInteraction::PointerEnter => Self::Hover,
            FluxInteraction::PointerLeave => Self::Idle,
            FluxInteraction::Pressed => Self::Press,
            FluxInteraction::Released => Self::Hover,
            FluxInteraction::PressCanceled => Self::Cancel,
            FluxInteraction::Disabled => Self::Idle,
        }
    }
}

impl From<&FluxInteraction> for InteractionStyle
{
    fn from(value: &FluxInteraction) -> Self
    {
        Self::from(*value)
    }
}

impl InteractionStyle
{
    fn alt(&self) -> Option<InteractionStyle>
    {
        match self {
            InteractionStyle::Idle => InteractionStyle::IdleAlt.into(),
            InteractionStyle::Hover => InteractionStyle::HoverAlt.into(),
            InteractionStyle::Press => InteractionStyle::PressAlt.into(),
            InteractionStyle::Cancel => None,
            InteractionStyle::IdleAlt => InteractionStyle::Idle.into(),
            InteractionStyle::HoverAlt => InteractionStyle::Hover.into(),
            InteractionStyle::PressAlt => InteractionStyle::Press.into(),
            InteractionStyle::Enter => None,
        }
    }
}

pub const TRANSITION_BETWEEN_POINTS: usize = 5;

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationResult
{
    Hold(InteractionStyle),
    Interpolate
    {
        from: InteractionStyle,
        to: InteractionStyle,
        t: f32,
        offset: f32,
    },
    TransitionBetween
    {
        origin: InteractionStyle,
        points: SmallVec<[(InteractionStyle, f32); TRANSITION_BETWEEN_POINTS]>,
    },
}

impl Default for AnimationResult
{
    fn default() -> Self
    {
        Self::Hold(Default::default())
    }
}

impl AnimationResult
{
    pub fn extract<T: Lerp + Default + Clone + PartialEq>(&self, bundle: &AnimatedVals<T>) -> T
    {
        match self {
            AnimationResult::Hold(style) => bundle.interaction_style(*style),
            AnimationResult::Interpolate { from, to, t, .. } => bundle
                .interaction_style(*from)
                .lerp(bundle.interaction_style(*to), *t),
            AnimationResult::TransitionBetween { origin, points } => {
                let start_value = bundle.interaction_style(*origin);
                points
                    .iter()
                    .fold(start_value, |current_value, (style, t)| {
                        current_value.lerp(bundle.interaction_style(*style), *t)
                    })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum AnimationLoop
{
    #[default]
    None,
    Continous,
    /// Repeat animation u8 number of times. Once the animation is completed
    /// it will reset to the start value if the second argument is set to true.
    Times(u8, bool),
    PingPongContinous,
    PingPong(u8),
}

#[derive(Clone, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnimationConfig
{
    /// Defaults to zero seconds.
    #[reflect(default)]
    pub duration: f32,
    /// Defaults to linear easing.
    #[reflect(default)]
    pub ease: Ease,
    /// Defaults to no delay.
    #[reflect(default)]
    pub delay: f32,
}

impl AnimationConfig
{
    pub fn new(duration: f32, ease: Ease, delay: f32) -> AnimationConfig
    {
        AnimationConfig { duration, ease, delay }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct LoopedAnimationConfig
{
    /// Defaults to zero duration.
    #[reflect(default)]
    pub duration: f32,
    /// Defaults to linear easing.
    #[reflect(default)]
    pub ease: Ease,
    /// Defaults to no delay.
    #[reflect(default)]
    pub start_delay: f32,
    /// Defaults to no gap.
    #[reflect(default)]
    pub loop_gap: f32,
    /// Defaults to `AnimationLoop::None`.
    #[reflect(default)]
    pub loop_type: AnimationLoop,
}

impl LoopedAnimationConfig
{
    pub fn new(
        duration: f32,
        ease: Ease,
        start_delay: f32,
        loop_gap: f32,
        loop_type: AnimationLoop,
    ) -> LoopedAnimationConfig
    {
        LoopedAnimationConfig { duration, ease, start_delay, loop_gap, loop_type }
    }

    fn is_pingpong(&self) -> bool
    {
        match self.loop_type {
            AnimationLoop::PingPong(_) | AnimationLoop::PingPongContinous => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Reflect, Serialize, Deserialize)]
pub struct AnimationSettings
{
    #[reflect(default)]
    pub enter_idle_with: Option<AnimationConfig>,
    // TODO: this does nothing
    // #[reflect(default)]
    // pub non_interacted: Option<AnimationConfig>,
    #[reflect(default)]
    pub hover_with: Option<AnimationConfig>,
    #[reflect(default)]
    pub unhover_with: Option<AnimationConfig>,
    #[reflect(default)]
    pub press_with: Option<AnimationConfig>,
    #[reflect(default)]
    pub release_with: Option<AnimationConfig>,
    #[reflect(default)]
    pub cancel_with: Option<AnimationConfig>,
    #[reflect(default)]
    pub cancel_end_with: Option<AnimationConfig>,
    #[reflect(default)]
    pub disable_with: Option<AnimationConfig>,
    #[reflect(default)]
    pub idle_loop: Option<LoopedAnimationConfig>,
    #[reflect(default)]
    pub hover_loop: Option<LoopedAnimationConfig>,
    #[reflect(default)]
    pub press_loop: Option<LoopedAnimationConfig>,
    #[reflect(default)]
    pub delete_on_entered: bool,
}

macro_rules! transition_animation_setter {
    ($setter:ident, $setter_edit:ident) => {
        pub fn $setter(&mut self, duration: f32, ease: Ease, delay: f32) -> &mut Self {
            let config = AnimationConfig { duration, ease, delay };
            self.$setter = Some(config);

            self
        }

        /// If the value is `None`, a default value will be pre-inserted.
        pub fn $setter_edit(
            &mut self,
            callback: impl FnOnce(&mut AnimationConfig),
        ) -> &mut Self {
            if self.$setter.is_none() {
                self.$setter = Some(AnimationConfig::default());
            }
            (callback)(self.$setter.as_mut().unwrap());

            self
        }
    };
}

macro_rules! transition_from_animation_setter {
    ($setter:ident, $setter_from:ident) => {
        pub fn $setter_from(&mut self, config: impl Into<Option<AnimationConfig>>) -> &mut Self {
            self.$setter = config.into();

            self
        }
    };
}

macro_rules! state_animation_setter {
    ($setter:ident, $setter_edit:ident) => {
        pub fn $setter(
            &mut self,
            duration: f32,
            ease: Ease,
            start_delay: f32,
            loop_gap: f32,
            loop_type: AnimationLoop,
        ) -> &mut Self {
            if duration <= 0. {
                warn!("Invalid animation duration used: {}", duration);
            }

            let config = LoopedAnimationConfig {
                duration,
                ease,
                start_delay,
                loop_gap,
                loop_type,
            };
            self.$setter = Some(config);

            self
        }

        /// If the value is `None`, a default value will be pre-inserted.
        pub fn $setter_edit(
            &mut self,
            callback: impl FnOnce(&mut LoopedAnimationConfig),
        ) -> &mut Self {
            if self.$setter.is_none() {
                self.$setter = Some(LoopedAnimationConfig::default());
            }
            (callback)(self.$setter.as_mut().unwrap());

            self
        }
    };
}

macro_rules! state_from_animation_setter {
    ($setter:ident, $setter_from:ident) => {
        pub fn $setter_from(
            &mut self,
            config: impl Into<Option<LoopedAnimationConfig>>,
        ) -> &mut Self {
            if let Some(config) = config.into() {
                if config.duration <= 0. {
                    warn!("Invalid animation duration used: {}", config.duration);
                }
                self.$setter = Some(config);
            } else {
                self.$setter = None;
            }

            self
        }
    };
}

impl AnimationSettings
{
    pub fn new() -> Self
    {
        Self { ..default() }
    }

    pub fn copy_from(&mut self, other: Self) -> &mut Self
    {
        self.enter_idle_with = other.enter_idle_with;
        //self.non_interacted = other.non_interacted;
        self.hover_with = other.hover_with;
        self.unhover_with = other.unhover_with;
        self.press_with = other.press_with;
        self.release_with = other.release_with;
        self.cancel_with = other.cancel_with;
        self.cancel_end_with = other.cancel_end_with;
        self.disable_with = other.disable_with;
        self.idle_loop = other.idle_loop;
        self.hover_loop = other.hover_loop;
        self.press_loop = other.press_loop;
        self.delete_on_entered = other.delete_on_entered;

        self
    }

    transition_animation_setter!(enter_idle_with, edit_enter_idle_with);
    // transition_animation_setter!(non_interacted, edit_non_interacted);
    // transition_from_animation_setter!(non_interacted, non_interacted_from);
    transition_animation_setter!(hover_with, edit_hover_with);
    transition_from_animation_setter!(hover_with, hover_with_from);
    transition_animation_setter!(unhover_with, edit_unhover_with);
    transition_from_animation_setter!(unhover_with, unhover_with_from);
    transition_animation_setter!(press_with, edit_press_with);
    transition_from_animation_setter!(press_with, press_with_from);
    transition_animation_setter!(release_with, edit_release_with);
    transition_from_animation_setter!(release_with, release_with_from);
    transition_animation_setter!(cancel_with, edit_cancel_with);
    transition_from_animation_setter!(cancel_with, cancel_with_from);
    transition_animation_setter!(cancel_end_with, edit_cancel_end_with);
    transition_from_animation_setter!(cancel_end_with, cancel_end_with_from);
    transition_animation_setter!(disable_with, edit_disable_with);
    transition_from_animation_setter!(disable_with, disable_with_from);
    state_animation_setter!(idle_loop, edit_idle_loop);
    state_from_animation_setter!(idle_loop, idle_loop_from);
    state_animation_setter!(hover_loop, edit_hover_loop);
    state_from_animation_setter!(hover_loop, hover_loop_from);
    state_animation_setter!(press_loop, edit_press_loop);
    state_from_animation_setter!(press_loop, press_loop_from);

    pub fn delete_on_entered(&mut self, do_delete: bool) -> &mut Self
    {
        self.delete_on_entered = do_delete;

        self
    }

    pub fn to_tween(&self, flux_interaction: &FluxInteraction) -> Option<AnimationConfig>
    {
        match flux_interaction {
            FluxInteraction::None => self.enter_idle_with.clone(),
            FluxInteraction::PointerEnter => self.hover_with.clone(),
            FluxInteraction::PointerLeave => self.unhover_with.clone(),
            FluxInteraction::Pressed => self.press_with.clone(),
            FluxInteraction::Released => self.release_with.clone(),
            FluxInteraction::PressCanceled => self.cancel_with.clone(),
            FluxInteraction::Disabled => self.disable_with.clone(),
        }
    }

    pub fn to_loop_tween(&self, flux_interaction: &FluxInteraction) -> Option<LoopedAnimationConfig>
    {
        match flux_interaction {
            FluxInteraction::None => self.idle_loop.clone(),
            FluxInteraction::PointerEnter => self.hover_loop.clone(),
            FluxInteraction::PointerLeave => self.idle_loop.clone(),
            FluxInteraction::Pressed => self.press_loop.clone(),
            FluxInteraction::Released => self.idle_loop.clone(),
            FluxInteraction::PressCanceled => self.idle_loop.clone(),
            FluxInteraction::Disabled => None,
        }
    }

    pub fn enter_duration(&self) -> StopwatchLock
    {
        AnimationSettings::transition_lock_duration(self.enter_idle_with.clone())
    }

    pub fn lock_duration(&self, flux_interaction: &FluxInteraction) -> StopwatchLock
    {
        let transition = match flux_interaction {
            FluxInteraction::PressCanceled => {
                let cancel_lock = AnimationSettings::transition_lock_duration(self.cancel_with.clone());
                let reset_lock = AnimationSettings::transition_lock_duration(self.cancel_end_with.clone());
                cancel_lock + reset_lock
            }
            _ => AnimationSettings::transition_lock_duration(self.to_tween(flux_interaction)),
        };

        let state_animation = match flux_interaction {
            FluxInteraction::None => AnimationSettings::state_lock_duration(self.idle_loop.clone()),
            FluxInteraction::PointerEnter => AnimationSettings::state_lock_duration(self.hover_loop.clone()),
            FluxInteraction::PointerLeave => AnimationSettings::state_lock_duration(self.idle_loop.clone()),
            FluxInteraction::Pressed => AnimationSettings::state_lock_duration(self.press_loop.clone()),
            FluxInteraction::Released => AnimationSettings::state_lock_duration(self.idle_loop.clone()),
            FluxInteraction::PressCanceled => AnimationSettings::state_lock_duration(self.idle_loop.clone()),
            FluxInteraction::Disabled => StopwatchLock::None,
        };

        transition + state_animation
    }

    pub fn transition_lock_duration(tween: Option<AnimationConfig>) -> StopwatchLock
    {
        let Some(tween) = tween else {
            return StopwatchLock::None;
        };

        StopwatchLock::Duration(Duration::from_secs_f32(tween.delay + tween.duration))
    }

    pub fn state_lock_duration(tween: Option<LoopedAnimationConfig>) -> StopwatchLock
    {
        let Some(tween) = tween else {
            return StopwatchLock::None;
        };

        match tween.loop_type {
            AnimationLoop::None => {
                StopwatchLock::Duration(Duration::from_secs_f32(tween.start_delay + tween.duration))
            }
            AnimationLoop::Continous => StopwatchLock::Infinite,
            AnimationLoop::Times(n, _) => StopwatchLock::Duration(Duration::from_secs_f32(
                tween.start_delay + (tween.duration * n as f32) + (tween.loop_gap * n as f32),
            )),
            AnimationLoop::PingPongContinous => StopwatchLock::Infinite,
            AnimationLoop::PingPong(n) => StopwatchLock::Duration(Duration::from_secs_f32(
                tween.start_delay + (tween.duration * n as f32) + (tween.loop_gap * n as f32),
            )),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationState
{
    result: AnimationResult,
    iteration: u8,
}

impl AnimationState
{
    pub fn extract<T: Lerp + Default + Clone + PartialEq>(&self, bundle: &AnimatedVals<T>) -> T
    {
        self.result.extract(bundle)
    }

    pub fn is_entering(&self) -> bool
    {
        match self.result {
            AnimationResult::Hold(style) => style == InteractionStyle::Enter,
            AnimationResult::Interpolate { from, .. } => from == InteractionStyle::Enter,
            _ => false,
        }
    }

    pub fn result(&self) -> &AnimationResult
    {
        &self.result
    }

    pub fn tick(
        &self,
        target_style: InteractionStyle,
        tween: Option<AnimationConfig>,
        loop_tween: Option<LoopedAnimationConfig>,
        elapsed: f32,
    ) -> Self
    {
        // No animation applied for the current interaction
        let Some(tween) = tween else {
            let (Some(alt_tween), Some(alt_target)) = (loop_tween, target_style.alt()) else {
                return AnimationState { result: AnimationResult::Hold(target_style), iteration: 0 };
            };

            return AnimationState::process_animation_loops(target_style, alt_target, elapsed, alt_tween);
        };

        let delay = tween.delay;
        let tween_time = tween.duration.max(0.);
        let ease = tween.ease;

        // Includes elapsed == 0.
        if elapsed <= delay {
            let AnimationResult::Interpolate { from, to, t, .. } = &self.result else {
                return self.clone();
            };

            // Special case of canceling a delay.
            if *from == target_style {
                return AnimationState {
                    result: AnimationResult::Interpolate { from: *to, to: *from, t: 1. - *t, offset: 1. - *t },
                    iteration: 0,
                };
            }

            return self.clone();
        }

        if elapsed >= (tween_time + delay) {
            // Do loop or hold
            let (Some(alt_tween), Some(alt_target)) = (loop_tween, target_style.alt()) else {
                return AnimationState { result: AnimationResult::Hold(target_style), iteration: 0 };
            };
            AnimationState::process_animation_loops(
                target_style,
                alt_target,
                elapsed - (tween_time + delay),
                alt_tween,
            )
        } else {
            AnimationState::process_transition_animations(
                target_style,
                elapsed,
                delay,
                tween_time,
                ease,
                &self.result,
            )
        }
    }

    fn process_animation_loops(
        target_style: InteractionStyle,
        alt_target: InteractionStyle,
        mut elapsed: f32,
        tween: LoopedAnimationConfig,
    ) -> AnimationState
    {
        let start_delay = tween.start_delay;
        if tween.loop_type == AnimationLoop::None || elapsed < start_delay || tween.duration <= 0. {
            return AnimationState { result: AnimationResult::Hold(target_style), iteration: 0 };
        }
        elapsed -= start_delay;

        let loop_gap = tween.loop_gap;
        let iteration = (elapsed / (tween.duration + loop_gap)).floor() as usize;
        let even = iteration % 2 == 0;

        match tween.loop_type {
            AnimationLoop::Times(times, reset) => {
                if iteration >= times as usize {
                    return AnimationState {
                        result: AnimationResult::Hold(match reset {
                            true => target_style,
                            false => alt_target,
                        }),
                        iteration: (iteration % 255) as u8,
                    };
                }
            }
            AnimationLoop::PingPong(times) => {
                if iteration >= times as usize {
                    return AnimationState {
                        result: AnimationResult::Hold(match even {
                            true => target_style,
                            false => alt_target,
                        }),
                        iteration: (iteration % 255) as u8,
                    };
                }
            }
            _ => (),
        }

        let offset = elapsed % (tween.duration + loop_gap);
        if loop_gap > 0. && offset > tween.duration {
            // We are in the pause-gap
            let hold_style = match tween.is_pingpong() {
                true => match even {
                    true => alt_target,
                    false => target_style,
                },
                false => alt_target,
            };

            AnimationState {
                result: AnimationResult::Hold(hold_style),
                iteration: (iteration % 255) as u8,
            }
        } else {
            let tween_ratio = (offset / tween.duration).clamp(0., 1.).ease(tween.ease);
            let from = match tween.is_pingpong() {
                true => match even {
                    true => target_style,
                    false => alt_target,
                },
                false => target_style,
            };
            let to = match tween.is_pingpong() {
                true => match even {
                    true => alt_target,
                    false => target_style,
                },
                false => alt_target,
            };

            AnimationState {
                result: AnimationResult::Interpolate { from, to, t: tween_ratio, offset: 0. },
                iteration: (iteration % 255) as u8,
            }
        }
    }

    fn process_transition_animations(
        target_style: InteractionStyle,
        elapsed: f32,
        delay: f32,
        tween_time: f32,
        ease: Ease,
        previous_result: &AnimationResult,
    ) -> AnimationState
    {
        let tween_ratio = ((elapsed - delay) / tween_time).clamp(0., 1.).ease(ease);
        match previous_result {
            AnimationResult::Hold(prev_style) => {
                AnimationState::process_hold(target_style, prev_style, tween_ratio)
            }
            AnimationResult::Interpolate { from, to, t, offset } => AnimationState::process_interpolate(
                target_style,
                elapsed,
                delay,
                tween_time,
                ease,
                from,
                to,
                t,
                offset,
            ),
            AnimationResult::TransitionBetween { origin, points } => {
                AnimationState::process_transition_between(target_style, tween_ratio, origin, points.clone())
            }
        }
    }

    fn process_hold(
        target_style: InteractionStyle,
        prev_style: &InteractionStyle,
        tween_ratio: f32,
    ) -> AnimationState
    {
        if *prev_style != target_style {
            AnimationState {
                result: AnimationResult::Interpolate {
                    from: *prev_style,
                    to: target_style,
                    t: tween_ratio,
                    offset: 0.,
                },
                iteration: 0,
            }
        } else {
            AnimationState { result: AnimationResult::Hold(target_style), iteration: 0 }
        }
    }

    fn process_interpolate(
        target_style: InteractionStyle,
        elapsed: f32,
        delay: f32,
        tween_time: f32,
        ease: Ease,
        from: &InteractionStyle,
        to: &InteractionStyle,
        t: &f32,
        offset: &f32,
    ) -> AnimationState
    {
        // Best effort complete the animation by tweening for only the remaining distance.
        // We could store `elapsed` and the `ease` type and try to continue animations,
        // but there is no guarantee we continue the interrupted one.
        let base_ratio = (((elapsed - delay) / tween_time) * (1. - offset)).clamp(0., 1.);
        let tween_ratio = (offset + base_ratio.ease(ease)).clamp(0., 1.);

        if *from == target_style {
            AnimationState {
                result: AnimationResult::Interpolate { from: *to, to: *from, t: 1. - *t, offset: 1. - *t },
                iteration: 0,
            }
        } else if *to == target_style {
            AnimationState {
                result: AnimationResult::Interpolate { from: *from, to: *to, t: tween_ratio, offset: *offset },
                iteration: 0,
            }
        } else {
            AnimationState {
                result: AnimationResult::TransitionBetween {
                    origin: *from,
                    points: SmallVec::from_slice(&[(*to, *t), (target_style, tween_ratio)]),
                },
                iteration: 0,
            }
        }
    }

    fn process_transition_between(
        target_style: InteractionStyle,
        tween_ratio: f32,
        origin: &InteractionStyle,
        mut points: SmallVec<[(InteractionStyle, f32); TRANSITION_BETWEEN_POINTS]>,
    ) -> AnimationState
    {
        let point_count = points.len();

        // Safe unwrap: We never remove points, only add, and we start with two points
        let last_point = points.last_mut().unwrap();
        let last_style = last_point.0;
        if last_style == target_style {
            last_point.1 = tween_ratio;
        } else if point_count < TRANSITION_BETWEEN_POINTS {
            points.push((target_style, tween_ratio));
        } else {
            // At this point, this is from a weird jiggle, escape leak!
            // Reset to the last two step's interpolation
            return AnimationState {
                result: AnimationResult::Interpolate {
                    from: last_style,
                    to: target_style,
                    t: tween_ratio,
                    offset: 0.,
                },
                iteration: 0,
            };
        }

        AnimationState {
            result: AnimationResult::TransitionBetween { origin: *origin, points },
            iteration: 0,
        }
    }
}

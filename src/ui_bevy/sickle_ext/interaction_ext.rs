use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::animated_interaction::{AnimatedInteraction, AnimationConfig};
use sickle_ui::interactions::InteractiveBackground;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::{FluxInteraction, FluxInteractionUpdate, TrackedInteraction};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Entity event emitted when [`FluxInteraction::PointerEnter`] is set on an entity.
pub struct PointerEnter;
/// Entity event emitted when [`FluxInteraction::PointerLeave`] is set on an entity.
pub struct PointerLeave;
/// Entity event emitted when [`FluxInteraction::Pressed`] is set on an entity.
pub struct Pressed;
/// Entity event emitted when [`FluxInteraction::Released`] is set on an entity.
pub struct Released;
/// Entity event emitted when [`FluxInteraction::PressCanceled`] is set on an entity.
pub struct PressCanceled;
/// Entity event emitted when [`FluxInteraction::Disabled`] is set on an entity.
pub struct Disabled;

//-------------------------------------------------------------------------------------------------------------------

/// Converts `sickle_ui` flux events to reactive entity events (see [`ReactCommand::entity_event`]).
pub(crate) fn flux_ui_events(mut c: Commands, fluxes: Query<(Entity, &FluxInteraction), Changed<FluxInteraction>>)
{
    for (entity, flux) in fluxes.iter() {
        match *flux {
            FluxInteraction::None => (),
            FluxInteraction::PointerEnter => {
                c.react().entity_event(entity, PointerEnter);
            }
            FluxInteraction::PointerLeave => {
                c.react().entity_event(entity, PointerLeave);
            }
            FluxInteraction::Pressed => {
                c.react().entity_event(entity, Pressed);
            }
            FluxInteraction::Released => {
                c.react().entity_event(entity, Released);
            }
            FluxInteraction::PressCanceled => {
                c.react().entity_event(entity, PressCanceled);
            }
            FluxInteraction::Disabled => {
                c.react().entity_event(entity, Disabled);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering interaction reactors for node entities.
pub trait UiInteractionExt
{
    /// Adds a reactor to a [`PointerEnter`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<PointerEnter>().r(callback)`.
    fn on_pointer_enter<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>;

    /// Adds a reactor to a [`PointerLeave`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<PointerLeave>().r(callback)`.
    fn on_pointer_leave<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>;

    /// Adds a reactor to a [`Pressed`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Pressed>().r(callback)`.
    fn on_pressed<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>;

    /// Adds a reactor to a [`Released`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Released>().r(callback)`.
    fn on_released<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>;

    /// Adds a reactor to a [`PressCanceled`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<PressCanceled>().r(callback)`.
    fn on_press_canceled<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>;

    /// Adds a reactor to a [`Disabled`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Disabled>().r(callback)`.
    fn on_disabled<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>;
}

impl UiInteractionExt for UiBuilder<'_, '_, '_, Entity>
{
    fn on_pointer_enter<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>
    {
        self.on_event::<Pressed>().r(callback)
    }

    fn on_pointer_leave<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>
    {
        self.on_event::<PointerLeave>().r(callback)
    }

    fn on_pressed<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static)
        -> EntityCommands<'_>
    {
        self.on_event::<Pressed>().r(callback)
    }

    fn on_released<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>
    {
        self.on_event::<Released>().r(callback)
    }

    fn on_press_canceled<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>
    {
        self.on_event::<PressCanceled>().r(callback)
    }

    fn on_disabled<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>
    {
        self.on_event::<Disabled>().r(callback)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable that indicates a node is interactable.
///
/// Causes [`Interaction`] and [`TrackedInteraction`] to be inserted on a node.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interactive;

impl ApplyLoadable for Interactive
{
    fn apply(self, ec: &mut EntityCommands)
    {
        ec.try_insert((Interaction::default(), TrackedInteraction::default()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Configuration for a specific interactive animation.
///
/// Mirrors [`AnimationConfig`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimateConfig
{
    /// Duration of the animation when moving into the 'activated' state.
    #[reflect(default)]
    pub duration: f32,
    // Easing when moving into the 'activated' state.
    //#[reflect(default)]
    //pub easing: Ease,
    /// Duration of the animation when moving out of the 'activated' state.
    ///
    /// If `None`, equals [`Self::duration`].
    #[reflect(default)]
    pub out_duration: Option<f32>,
    // Easing when moving out of the 'activated' state.
    //
    // If `None`, equals [`Self::easing`].
    //pub out_easing: Option<Ease>,
}

impl Into<AnimationConfig> for AnimateConfig
{
    fn into(self) -> AnimationConfig
    {
        AnimationConfig {
            duration: self.duration,
            easing: Default::default(),
            out_duration: self.out_duration,
            out_easing: None,
        }
    }
}

impl From<AnimationConfig> for AnimateConfig
{
    fn from(config: AnimationConfig) -> AnimateConfig
    {
        AnimateConfig {
            duration: config.duration,
            //easing: Default::default(),
            out_duration: config.out_duration,
            //out_easing: None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Settings for an animatable attribute on a node.
///
/// Mirrors [`AnimatedInteraction`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationSettings
{
    /// Default [`AnimationConfig`] for the attribute.
    ///
    /// Can be overridden by the `hover`/`press`/`cancel` fields respectively.
    #[reflect(default = "AnimationSettings::default_default_field")]
    pub default: AnimateConfig,
    #[reflect(default = "AnimationSettings::default_hover_field")]
    pub hover: Option<AnimateConfig>,
    #[reflect(default = "AnimationSettings::default_press_field")]
    pub press: Option<AnimateConfig>,
    #[reflect(default = "AnimationSettings::default_cancel_field")]
    pub cancel: Option<AnimateConfig>,
    #[reflect(default = "AnimationSettings::default_reset_field")]
    pub reset_delay: Option<f32>,
}

impl AnimationSettings
{
    fn default_default_field() -> AnimateConfig
    {
        AnimatedInteraction::<Node>::default().tween.into()
    }

    fn default_hover_field() -> Option<AnimateConfig>
    {
        AnimatedInteraction::<Node>::default().hover.map(AnimateConfig::from)
    }

    fn default_press_field() -> Option<AnimateConfig>
    {
        AnimatedInteraction::<Node>::default().press.map(AnimateConfig::from)
    }

    fn default_cancel_field() -> Option<AnimateConfig>
    {
        AnimatedInteraction::<Node>::default().cancel.map(AnimateConfig::from)
    }

    fn default_reset_field() -> Option<f32>
    {
        AnimatedInteraction::<Node>::default().reset_delay
    }

    /// Convers the settings to an [`AnimatedInteraction`].
    pub fn to_sickle<T: Component>(self) -> AnimatedInteraction<T>
    {
        AnimatedInteraction::<T> {
            tween: self.default.into(),
            hover: self.hover.map(AnimateConfig::into),
            press: self.press.map(AnimateConfig::into),
            cancel: self.cancel.map(AnimateConfig::into),
            reset_delay: self.reset_delay,
            ..default()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable version of [`InteractiveBackground`].
///
/// Applies the [`BgColor`] and [`Interactive`] loadables automatically.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimatedBgColor
{
    pub base: Color,
    #[reflect(default)]
    pub highlight: Option<Color>,
    #[reflect(default)]
    pub pressed: Option<Color>,
    #[reflect(default)]
    pub cancel: Option<Color>,
    #[reflect(default)]
    pub animate: AnimationSettings,
}

impl ApplyLoadable for AnimatedBgColor
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let interactive_bg = InteractiveBackground {
            highlight: self.highlight,
            pressed: self.pressed,
            cancel: self.cancel,
        };
        let animated = self.animate.to_sickle::<InteractiveBackground>();

        Interactive.apply(ec);
        BgColor(self.base).apply(ec);
        ec.try_insert((interactive_bg, animated));
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiInteractionExtPlugin;

impl Plugin for UiInteractionExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Interactive>()
            .register_type::<Option<Color>>()
            .register_type::<AnimateConfig>()
            .register_type::<AnimationSettings>()
            .register_type::<AnimatedBgColor>()
            .register_derived_loadable::<Interactive>()
            .register_derived_loadable::<AnimatedBgColor>()
            .add_systems(Update, flux_ui_events.after(FluxInteractionUpdate));
    }
}

//-------------------------------------------------------------------------------------------------------------------

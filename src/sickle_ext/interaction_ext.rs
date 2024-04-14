use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::{ui_builder::UiBuilder, FluxInteraction, FluxInteractionUpdate, TrackedInteraction};

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

    fn on_pressed<M>(
        &mut self,
        callback: impl IntoSystem<(), (), M> + Send + Sync + 'static,
    ) -> EntityCommands<'_>
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

impl StyleToBevy for Interactive
{
    fn to_bevy(self, ec: &mut EntityCommands)
    {
        ec.try_insert((Interaction::default(), TrackedInteraction::default()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiInteractionExtPlugin;

impl Plugin for UiInteractionExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<Interactive>()
            .register_derived_style::<Interactive>()
            .add_systems(Update, flux_ui_events.after(FluxInteractionUpdate))
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

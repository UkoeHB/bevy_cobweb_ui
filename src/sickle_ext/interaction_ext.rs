use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::theme::pseudo_state::{PseudoState, PseudoStates};
use sickle_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Converts `sickle_ui` flux events to reactive entity events (see [`ReactCommand::entity_event`]).
fn flux_ui_events(
    mut c: Commands,
    fluxes: Query<(Entity, &FluxInteraction, Option<&PseudoStates>), Changed<FluxInteraction>>,
)
{
    for (entity, flux, maybe_pseudo_states) in fluxes.iter() {
        // Ignore disabled entities.
        if let Some(pseudo_states) = maybe_pseudo_states {
            if pseudo_states.has(&PseudoState::Disabled) {
                continue;
            }
        }

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
                // No flux interaction event for disabled. See the `Disable` entity event.
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Entity event emitted when [`FluxInteraction::PointerEnter`] is set on an entity.
///
/// Not emitted if the entity has [`PseudoState::Disabled`].
pub struct PointerEnter;
/// Entity event emitted when [`FluxInteraction::PointerLeave`] is set on an entity.
///
/// Not emitted if the entity has [`PseudoState::Disabled`].
pub struct PointerLeave;
/// Entity event emitted when [`FluxInteraction::Pressed`] is set on an entity.
///
/// Not emitted if the entity has [`PseudoState::Disabled`].
pub struct Pressed;
/// Entity event emitted when [`FluxInteraction::Released`] is set on an entity.
///
/// Not emitted if the entity has [`PseudoState::Disabled`].
pub struct Released;
/// Entity event emitted when [`FluxInteraction::PressCanceled`] is set on an entity.
///
/// Not emitted if the entity has [`PseudoState::Disabled`].
pub struct PressCanceled;

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
}

impl UiInteractionExt for EntityCommands<'_>
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
}

// TODO: same extensions for UiBuilder2d

//-------------------------------------------------------------------------------------------------------------------

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

// TODO: Interactive2d

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiInteractionExtPlugin;

impl Plugin for UiInteractionExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_derived::<Interactive>()
            .add_systems(Update, flux_ui_events.after(FluxInteractionUpdate));
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::FluxInteraction;

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
pub(crate) fn flux_ui_events(mut rc: ReactCommands, fluxes: Query<(Entity, &FluxInteraction), Changed<FluxInteraction>>)
{
    for (entity, flux) in fluxes.iter() {
        match *flux {
            FluxInteraction::None => (),
            FluxInteraction::PointerEnter => {
                rc.entity_event(entity, PointerEnter);
            }
            FluxInteraction::PointerLeave => {
                rc.entity_event(entity, PointerLeave);
            }
            FluxInteraction::Pressed => {
                rc.entity_event(entity, Pressed);
            }
            FluxInteraction::Released => {
                rc.entity_event(entity, Released);
            }
            FluxInteraction::PressCanceled => {
                rc.entity_event(entity, PressCanceled);
            }
            FluxInteraction::Disabled => {
                rc.entity_event(entity, Disabled);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

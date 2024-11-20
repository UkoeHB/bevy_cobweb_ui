use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

/// Converts `sickle_ui` flux events to reactive entity events (see [`ReactCommand::entity_event`]).
///
/// Is situated between `FluxInteractionUpdate` and `ApplyFluxChanges` sets so the effects of reactions here
/// can be immediately handled.
//todo: better to have these in PreUpdate - note that state transitions occur between PreUpdate and Update, so
// any states set in reaction to these events will be applied 1 frame late
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
    fn on_pointer_enter<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`PointerLeave`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<PointerLeave>().r(callback)`.
    fn on_pointer_leave<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`Pressed`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Pressed>().r(callback)`.
    fn on_pressed<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`Released`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Released>().r(callback)`.
    fn on_released<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`PressCanceled`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<PressCanceled>().r(callback)`.
    fn on_press_canceled<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;
}

impl UiInteractionExt for UiBuilder<'_, Entity>
{
    fn on_pointer_enter<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Pressed>().r(callback);
        self
    }

    fn on_pointer_leave<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<PointerLeave>().r(callback);
        self
    }

    fn on_pressed<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Pressed>().r(callback);
        self
    }

    fn on_released<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Released>().r(callback);
        self
    }

    fn on_press_canceled<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<PressCanceled>().r(callback);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Causes [`Interaction`] and [`TrackedInteraction`] to be inserted on a node.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Interactive;

impl Instruction for Interactive
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert((Interaction::default(), TrackedInteraction::default()));
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<(Interaction, TrackedInteraction)>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiInteractionExtPlugin;

impl Plugin for UiInteractionExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<Interactive>().add_systems(
            Update,
            flux_ui_events
                .after(FluxInteractionUpdate)
                .before(ApplyFluxChanges),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

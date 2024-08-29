use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_commands::ManagePseudoStateExt;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn detect_enable_reactor(event: EntityEvent<Enable>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).add_pseudo_state(PseudoState::Enabled);
    c.entity(entity).remove_pseudo_state(PseudoState::Disabled);
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_disable_reactor(event: EntityEvent<Disable>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).add_pseudo_state(PseudoState::Disabled);
    c.entity(entity).remove_pseudo_state(PseudoState::Enabled);
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_select_reactor(event: EntityEvent<Select>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).add_pseudo_state(PseudoState::Selected);
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_deselect_reactor(event: EntityEvent<Deselect>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).remove_pseudo_state(PseudoState::Selected);
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_check_reactor(event: EntityEvent<Check>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).add_pseudo_state(PseudoState::Checked);
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_uncheck_reactor(event: EntityEvent<Uncheck>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).remove_pseudo_state(PseudoState::Checked);
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_open_reactor(event: EntityEvent<Open>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).add_pseudo_state(PseudoState::Open);
    c.entity(entity).remove_pseudo_state(PseudoState::Closed);
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_close_reactor(event: EntityEvent<Close>, mut c: Commands)
{
    let entity = event.entity();
    c.entity(entity).add_pseudo_state(PseudoState::Closed);
    c.entity(entity).remove_pseudo_state(PseudoState::Open);
}

//-------------------------------------------------------------------------------------------------------------------

/// Entity event that can be sent to set [`PseudoState::Enabled`] on an entity (and remove
/// [`PseudoState::Disabled`]).
pub struct Enable;
/// Entity event that can be sent to set [`PseudoState::Disabled`] on an entity (and remove
/// [`PseudoState::Enabled`]).
pub struct Disable;
/// Entity event that can be sent to set [`PseudoState::Selected`] on an entity.
pub struct Select;
/// Entity event that can be sent to remove [`PseudoState::Selected`] from an entity.
pub struct Deselect;
/// Entity event that can be sent to set [`PseudoState::Checked`] on an entity.
pub struct Check;
/// Entity event that can be sent to remove [`PseudoState::Checked`] from an entity.
pub struct Uncheck;
/// Entity event that can be sent to set [`PseudoState::Open`] on an entity (and remove
/// [`PseudoState::Closed`]).
pub struct Open;
/// Entity event that can be sent to set [`PseudoState::Closed`] on an entity (and remove
/// [`PseudoState::Open`]).
pub struct Close;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering interaction reactors for node entities.
pub trait PseudoStateExt
{
    /// Adds a reactor to an [`Enable`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Enable>().r(callback)`.
    fn on_enable<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`Disable`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Disable>().r(callback)`.
    fn on_disable<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`Select`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Select>().r(callback)`.
    fn on_select<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`Deselect`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Deselect>().r(callback)`.
    fn on_deselect<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`Check`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Check>().r(callback)`.
    fn on_check<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to an [`Uncheck`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Uncheck>().r(callback)`.
    fn on_uncheck<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to an [`Open`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Open>().r(callback)`.
    fn on_open<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    /// Adds a reactor to a [`Close`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Close>().r(callback)`.
    fn on_close<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;
}

impl PseudoStateExt for UiBuilder<'_, Entity>
{
    fn on_enable<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Enable>().r(callback);
        self
    }

    /// Adds a reactor to a [`Disable`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Disable>().r(callback)`.
    fn on_disable<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Disable>().r(callback);
        self
    }

    /// Adds a reactor to a [`Select`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Select>().r(callback)`.
    fn on_select<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Select>().r(callback);
        self
    }

    /// Adds a reactor to a [`Deselect`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Deselect>().r(callback)`.
    fn on_deselect<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Deselect>().r(callback);
        self
    }

    /// Adds a reactor to a [`Check`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Check>().r(callback)`.
    fn on_check<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Check>().r(callback);
        self
    }

    /// Adds a reactor to an [`Uncheck`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Uncheck>().r(callback)`.
    fn on_uncheck<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Uncheck>().r(callback);
        self
    }

    /// Adds a reactor to an [`Open`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Open>().r(callback)`.
    fn on_open<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Open>().r(callback);
        self
    }

    /// Adds a reactor to a [`Close`] entity event.
    ///
    /// Equivalent to `entity_builder.on_event::<Close>().r(callback)`.
    fn on_close<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<Close>().r(callback);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct PseudoStatesExtPlugin;

impl Plugin for PseudoStatesExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_simple_reactor(any_entity_event::<Enable>(), detect_enable_reactor);
        app.add_simple_reactor(any_entity_event::<Disable>(), detect_disable_reactor);
        app.add_simple_reactor(any_entity_event::<Select>(), detect_select_reactor);
        app.add_simple_reactor(any_entity_event::<Deselect>(), detect_deselect_reactor);
        app.add_simple_reactor(any_entity_event::<Check>(), detect_check_reactor);
        app.add_simple_reactor(any_entity_event::<Uncheck>(), detect_uncheck_reactor);
        app.add_simple_reactor(any_entity_event::<Open>(), detect_open_reactor);
        app.add_simple_reactor(any_entity_event::<Close>(), detect_close_reactor);
    }
}

//-------------------------------------------------------------------------------------------------------------------

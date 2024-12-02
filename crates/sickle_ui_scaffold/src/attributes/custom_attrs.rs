use std::any::TypeId;
use std::fmt::{Debug, Formatter, Result};
use std::sync::Arc;

use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;

use crate::*;

#[derive(Clone)]
pub struct CustomStaticStyleAttribute
{
    type_id: TypeId,
    reference: Arc<dyn AnyClone + Send + Sync + 'static>,
    callback: fn(Entity, &mut World, &dyn AnyClone),
}

impl CustomStaticStyleAttribute
{
    pub fn new(
        type_id: TypeId,
        reference: Arc<dyn AnyClone + Send + Sync + 'static>,
        callback: fn(Entity, &mut World, &dyn AnyClone),
    ) -> Self
    {
        Self { type_id, reference, callback }
    }
}

impl LogicalEq for CustomStaticStyleAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self.type_id == other.type_id
    }
}

impl Debug for CustomStaticStyleAttribute
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        f.debug_struct("CustomStaticStyleAttribute").finish()
    }
}

impl PartialEq for CustomStaticStyleAttribute
{
    fn eq(&self, other: &Self) -> bool
    {
        Arc::ptr_eq(&self.reference, &other.reference)
    }
}

#[derive(Clone)]
pub struct ResponsiveStyleAttribute
{
    type_id: TypeId,
    reference: Arc<dyn AnyClone + Send + Sync + 'static>,
    callback: fn(Entity, FluxInteraction, &mut World, &dyn AnyClone),
}

impl ResponsiveStyleAttribute
{
    pub fn new(
        type_id: TypeId,
        reference: Arc<dyn AnyClone + Send + Sync + 'static>,
        callback: fn(Entity, FluxInteraction, &mut World, &dyn AnyClone),
    ) -> Self
    {
        Self { type_id, reference, callback }
    }

    pub fn apply(&self, flux_interaction: FluxInteraction, ui_style: &mut UiStyle)
    {
        ui_style
            .entity_commands()
            .queue(ApplyResponsiveStyleAttribute { attr: self.clone(), flux_interaction });
    }
}

impl LogicalEq for ResponsiveStyleAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self.type_id == other.type_id
    }
}

impl Debug for ResponsiveStyleAttribute
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        f.debug_struct("ResponsiveStyleAttribute").finish()
    }
}

impl PartialEq for ResponsiveStyleAttribute
{
    fn eq(&self, other: &Self) -> bool
    {
        Arc::ptr_eq(&self.reference, &other.reference)
    }
}

#[derive(Clone)]
pub struct AnimatedStyleAttribute
{
    type_id: TypeId,
    reference: Arc<dyn AnyClone + Send + Sync + 'static>,
    callback: fn(Entity, AnimationState, &mut World, &dyn AnyClone),
}

impl AnimatedStyleAttribute
{
    pub fn new(
        type_id: TypeId,
        reference: Arc<dyn AnyClone + Send + Sync + 'static>,
        callback: fn(Entity, AnimationState, &mut World, &dyn AnyClone),
    ) -> Self
    {
        Self { type_id, reference, callback }
    }

    pub fn apply(&self, current_state: &AnimationState, ui_style: &mut UiStyle)
    {
        ui_style
            .entity_commands()
            .queue(ApplyAnimatadStyleAttribute { attr: self.clone(), current_state: current_state.clone() });
    }
}

impl LogicalEq for AnimatedStyleAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self.type_id == other.type_id
    }
}

impl Debug for AnimatedStyleAttribute
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        f.debug_struct("AnimatedStyleAttribute").finish()
    }
}

impl PartialEq for AnimatedStyleAttribute
{
    fn eq(&self, other: &Self) -> bool
    {
        Arc::ptr_eq(&self.reference, &other.reference)
    }
}

pub struct ApplyCustomStaticStyleAttribute
{
    pub attr: CustomStaticStyleAttribute,
}

impl EntityCommand for ApplyCustomStaticStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        (self.attr.callback)(id, world, self.attr.reference.as_ref());
    }
}

pub struct ApplyResponsiveStyleAttribute
{
    pub attr: ResponsiveStyleAttribute,
    pub flux_interaction: FluxInteraction,
}

impl EntityCommand for ApplyResponsiveStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        (self.attr.callback)(id, self.flux_interaction, world, self.attr.reference.as_ref());
    }
}

pub struct ApplyAnimatadStyleAttribute
{
    pub attr: AnimatedStyleAttribute,
    pub current_state: AnimationState,
}

impl EntityCommand for ApplyAnimatadStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        (self.attr.callback)(id, self.current_state, world, self.attr.reference.as_ref());
    }
}

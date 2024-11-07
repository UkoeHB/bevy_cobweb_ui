use std::fmt::{Debug, Formatter, Result};
use std::sync::Arc;

use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;

use crate::attributes::prelude::*;
use crate::flux_interaction::FluxInteraction;
use crate::prelude::{LogicalEq, UiStyle};

#[derive(Clone)]
pub struct CustomStaticStyleAttribute
{
    pub callback: Arc<dyn Fn(Entity, &mut World) + Send + Sync + 'static>,
}

impl CustomStaticStyleAttribute
{
    pub fn new(callback: impl Fn(Entity, &mut World) + Send + Sync + 'static) -> Self
    {
        Self { callback: Arc::new(callback) }
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
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

#[derive(Clone)]
pub struct InteractiveStyleAttribute
{
    pub callback: Arc<dyn Fn(Entity, FluxInteraction, &mut World) + Send + Sync + 'static>,
}

impl InteractiveStyleAttribute
{
    pub fn new(callback: impl Fn(Entity, FluxInteraction, &mut World) + Send + Sync + 'static) -> Self
    {
        Self { callback: Arc::new(callback) }
    }

    pub fn apply(&self, flux_interaction: FluxInteraction, ui_style: &mut UiStyle)
    {
        ui_style
            .entity_commands()
            .queue(ApplyInteractiveStyleAttribute { callback: self.clone(), flux_interaction });
    }
}

impl LogicalEq for InteractiveStyleAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self == other
    }
}

impl Debug for InteractiveStyleAttribute
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        f.debug_struct("InteractiveStyleAttribute").finish()
    }
}

impl PartialEq for InteractiveStyleAttribute
{
    fn eq(&self, other: &Self) -> bool
    {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

#[derive(Clone)]
pub struct AnimatedStyleAttribute
{
    pub callback: Arc<dyn Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static>,
}

impl AnimatedStyleAttribute
{
    pub fn new(callback: impl Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static) -> Self
    {
        Self { callback: Arc::new(callback) }
    }

    pub fn apply(&self, current_state: &AnimationState, ui_style: &mut UiStyle)
    {
        ui_style
            .entity_commands()
            .queue(ApplyAnimatadStyleAttribute { callback: self.clone(), current_state: current_state.clone() });
    }
}

impl LogicalEq for AnimatedStyleAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self == other
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
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

pub struct ApplyCustomStaticStyleAttribute
{
    pub callback: CustomStaticStyleAttribute,
}

impl EntityCommand for ApplyCustomStaticStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        (self.callback.callback)(id, world);
    }
}

pub struct ApplyInteractiveStyleAttribute
{
    pub callback: InteractiveStyleAttribute,
    pub flux_interaction: FluxInteraction,
}

impl EntityCommand for ApplyInteractiveStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        (self.callback.callback)(id, self.flux_interaction, world);
    }
}

pub struct ApplyAnimatadStyleAttribute
{
    pub callback: AnimatedStyleAttribute,
    pub current_state: AnimationState,
}

impl EntityCommand for ApplyAnimatadStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        (self.callback.callback)(id, self.current_state, world);
    }
}

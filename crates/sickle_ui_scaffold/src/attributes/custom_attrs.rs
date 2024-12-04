use std::any::TypeId;
use std::fmt::{Debug, Formatter, Result};
use std::sync::Arc;

use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct CustomStaticStyleAttribute
{
    type_id: TypeId,
    attr: Arc<dyn StaticAttributeObject>,
}

impl CustomStaticStyleAttribute
{
    pub fn new(type_id: TypeId, attr: Arc<dyn StaticAttributeObject>) -> Self
    {
        Self { type_id, attr }
    }

    pub fn apply(&self, ui_style: &mut UiStyle)
    {
        ui_style
            .entity_commands()
            .queue(ApplyCustomStaticStyleAttribute { attr: self.attr.clone() });
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
        Arc::ptr_eq(&self.attr, &other.attr)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct ResponsiveStyleAttribute
{
    type_id: TypeId,
    attr: Arc<dyn ResponsiveAttributeObject>,
}

impl ResponsiveStyleAttribute
{
    pub fn new(type_id: TypeId, attr: Arc<dyn ResponsiveAttributeObject>) -> Self
    {
        Self { type_id, attr }
    }

    pub fn apply(&self, flux_interaction: FluxInteraction, ui_style: &mut UiStyle)
    {
        ui_style
            .entity_commands()
            .queue(ApplyResponsiveStyleAttribute { attr: self.attr.clone(), flux_interaction });
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
        Arc::ptr_eq(&self.attr, &other.attr)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct AnimatedStyleAttribute
{
    type_id: TypeId,
    attr: Arc<dyn AnimatedAttributeObject>,
}

impl AnimatedStyleAttribute
{
    pub fn new(type_id: TypeId, attr: Arc<dyn AnimatedAttributeObject>) -> Self
    {
        Self { type_id, attr }
    }

    pub fn initialize_enter(&mut self, entity: Entity, world: &World)
    {
        let attr = dyn_clone::arc_make_mut(&mut self.attr);
        attr.initialize_enter(entity, world);
    }

    pub fn apply(&self, current_state: &AnimationState, ui_style: &mut UiStyle)
    {
        ui_style
            .entity_commands()
            .queue(ApplyAnimatedStyleAttribute {
                attr: self.attr.clone(),
                current_state: current_state.clone(),
            });
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
        Arc::ptr_eq(&self.attr, &other.attr)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct ApplyCustomStaticStyleAttribute
{
    pub attr: Arc<dyn StaticAttributeObject>,
}

impl EntityCommand for ApplyCustomStaticStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        self.attr.apply(id, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct ApplyResponsiveStyleAttribute
{
    pub attr: Arc<dyn ResponsiveAttributeObject>,
    pub flux_interaction: FluxInteraction,
}

impl EntityCommand for ApplyResponsiveStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        self.attr.apply(id, world, self.flux_interaction);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct ApplyAnimatedStyleAttribute
{
    pub attr: Arc<dyn AnimatedAttributeObject>,
    pub current_state: AnimationState,
}

impl EntityCommand for ApplyAnimatedStyleAttribute
{
    fn apply(self, id: Entity, world: &mut World)
    {
        self.attr.apply(id, world, self.current_state);
    }
}

//-------------------------------------------------------------------------------------------------------------------

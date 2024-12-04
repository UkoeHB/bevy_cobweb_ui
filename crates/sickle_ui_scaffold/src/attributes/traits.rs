use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for storing custom static attributes.
pub trait StaticAttributeObject: AnyClone
{
    /// Convert self to an `AnyClone` reference.
    fn as_anyclone(&self) -> &dyn AnyClone;
    /// Convert self to a mutable `AnyClone` reference.
    fn as_anyclone_mut(&mut self) -> &mut dyn AnyClone;

    /// Applies the attribute to the target entity.
    fn apply(&self, entity: Entity, world: &mut World);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for storing custom responsive attributes.
pub trait ResponsiveAttributeObject: AnyClone
{
    /// Convert self to an `AnyClone` reference.
    fn as_anyclone(&self) -> &dyn AnyClone;
    /// Convert self to a mutable `AnyClone` reference.
    fn as_anyclone_mut(&mut self) -> &mut dyn AnyClone;

    /// Applies the attribute to the target entity.
    fn apply(&self, entity: Entity, world: &mut World, state: FluxInteraction);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for storing custom animated attributes.
pub trait AnimatedAttributeObject: AnyClone
{
    /// Convert self to an `AnyClone` reference.
    fn as_anyclone(&self) -> &dyn AnyClone;
    /// Convert self to a mutable `AnyClone` reference.
    fn as_anyclone_mut(&mut self) -> &mut dyn AnyClone;

    /// Initializes self when begining to enter the idle state.
    fn initialize_enter(&mut self, entity: Entity, world: &World);

    /// Applies the attribute to the target entity.
    fn apply(&self, entity: Entity, world: &mut World, state: AnimationState);
}

//-------------------------------------------------------------------------------------------------------------------

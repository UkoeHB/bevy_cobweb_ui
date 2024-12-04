use std::any::Any;
use std::fmt::Debug;

use bevy::prelude::*;
use dyn_clone::DynClone;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for storing custom static attributes.
pub trait StaticAttributeObject: Any + DynClone + Debug + Send + Sync + 'static
{
    /// Convert self to an `Any` reference.
    fn as_any(&self) -> &dyn Any;
    /// Convert self to a mutable `Any` reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Applies the attribute to the target entity.
    fn apply(&self, entity: Entity, world: &mut World);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for storing custom responsive attributes.
pub trait ResponsiveAttributeObject: Any + DynClone + Debug + Send + Sync + 'static
{
    /// Convert self to an `Any` reference.
    fn as_any(&self) -> &dyn Any;
    /// Convert self to a mutable `Any` reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Applies the attribute to the target entity.
    fn apply(&self, entity: Entity, world: &mut World, state: FluxInteraction);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for storing custom animated attributes.
pub trait AnimatedAttributeObject: Any + DynClone + Debug + Send + Sync + 'static
{
    /// Convert self to an `Any` reference.
    fn as_any(&self) -> &dyn Any;
    /// Convert self to a mutable `Any` reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Initializes self when begining to enter the idle state.
    fn initialize_enter(&mut self, entity: Entity, world: &World);

    /// Applies the attribute to the target entity.
    fn apply(&self, entity: Entity, world: &mut World, state: AnimationState);
}

//-------------------------------------------------------------------------------------------------------------------

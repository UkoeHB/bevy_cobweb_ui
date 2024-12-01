use bevy::prelude::*;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that specify a static value tied to pseudo states.
///
/// See [`Static`].
pub trait StaticAttribute: Instruction
{
    /// Specifies the value-type of the attribute.
    type Value: Loadable + Clone;

    /// Converts [`Self::Value`] into `Self`.
    fn construct(value: Self::Value) -> Self;

    /// Updates an entity with a value.
    #[inline(always)]
    fn update(entity: Entity, world: &mut World, new_value: Self::Value)
    {
        Self::construct(new_value).apply(entity, world);
        world.flush();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that respond to interactions.
///
/// Use [`Interactive`] to make an entity interactable.
///
/// See [`Responsive`].
pub trait ResponsiveAttribute: StaticAttribute
{
    /// Extracts a value from an interaction state.
    ///
    /// The `reference_vals` are set in [`Responsive`].
    ///
    /// The value will be applied with [`StaticAttribute::update`].
    #[inline(always)]
    fn extract(
        _: Entity,
        _: &mut World,
        reference_vals: &InteractiveVals<Self::Value>,
        state: FluxInteraction,
    ) -> Self::Value
    {
        reference_vals.to_value(state)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
///
/// Use [`Interactive`] to make an entity interactable.
///
/// See [`Animated`].
pub trait AnimatedAttribute: StaticAttribute<Value: Lerp>
{
    /// Extracts a value that should be applied to the entity.
    ///
    /// The `reference_vals` are set in [`Animated`].
    ///
    /// The value will be applied with [`StaticAttribute::update`].
    #[inline(always)]
    fn extract(
        _: Entity,
        _: &mut World,
        reference_vals: &AnimatedVals<Self::Value>,
        state: &AnimationState,
    ) -> Self::Value
    {
        reference_vals.to_value(state)
    }
}

//-------------------------------------------------------------------------------------------------------------------

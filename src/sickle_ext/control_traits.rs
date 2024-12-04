use std::fmt::Debug;

use bevy::prelude::*;
use dyn_clone::DynClone;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that specify a static value tied to pseudo states.
///
/// See [`Static`].
pub trait StaticAttribute: Instruction
{
    /// Specifies the value-type of the attribute.
    type Value: Loadable + Clone + DynClone + Debug;

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
        reference_vals: &ResponsiveVals<Self::Value>,
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
    /// Tries to get the current value of the attribute on the entity.
    ///
    /// Used to set the `AnimatedVals::enter_ref` field when entering a new state if the attribute includes an
    /// on-enter animation. Note there is [`Animated::enter_ref_override`] if you want to manually specify an
    /// enter value.
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>;

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

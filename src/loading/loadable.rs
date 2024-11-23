use std::fmt::Debug;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::reflect::{GetTypeRegistration, Typed};

//-------------------------------------------------------------------------------------------------------------------

/// Trait representing types that can be loaded from cobweb asset files.
pub trait Loadable: Reflect + FromReflect + PartialEq + Default {}

impl<T> Loadable for T where T: Reflect + FromReflect + PartialEq + Default {}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for converting `Self` into entity modifications.
///
/// An instruction can be written in a COB file, or applied directly with
/// [`apply`](InstructionExt::apply).
///
/// See [`register_instruction`](crate::prelude::CobLoadableRegistrationAppExt::register_instruction).
pub trait Instruction: Loadable
{
    /// Applies the instruction to the entity.
    ///
    /// Assume the entity might not exist. This should not panic unless necessary.
    fn apply(self, entity: Entity, world: &mut World);

    /// Reverts the instruction on the entity.
    ///
    /// This should clean up as many of the instruction's side effects as possible.
    ///
    /// Assume the entity might not exist. This should not panic unless necessary.
    fn revert(entity: Entity, world: &mut World);
}

//-------------------------------------------------------------------------------------------------------------------

/// Extension trait for applying instructions to entities.
pub trait InstructionExt
{
    /// Applies an instruction to the entity.
    fn apply(&mut self, instruction: impl Instruction) -> &mut Self;

    /// Reverts an instruction on the entity.
    fn revert<T: Instruction>(&mut self) -> &mut Self;
}

impl InstructionExt for EntityCommands<'_>
{
    fn apply(&mut self, instruction: impl Instruction + Send + Sync + 'static) -> &mut Self
    {
        self.queue(move |e: Entity, w: &mut World| instruction.apply(e, w));
        self
    }

    fn revert<T: Instruction>(&mut self) -> &mut Self
    {
        self.queue(|e: Entity, w: &mut World| T::revert(e, w));
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper loadable for cases where multiple values of the same type can be loaded.
///
/// Note that `Multi<T>` must be manually registered with `register_instruction_type` or
/// `register_command_type` for all `T` that want to use it.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Multi<T>(Vec<T>);

impl<T: Instruction + Typed + FromReflect + GetTypeRegistration> Instruction for Multi<T>
{
    fn apply(mut self, entity: Entity, world: &mut World)
    {
        for item in self.0.drain(..) {
            item.apply(entity, world);
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        T::revert(entity, world);
    }
}

impl<T: Command + TypePath + FromReflect + GetTypeRegistration> Command for Multi<T>
{
    fn apply(mut self, world: &mut World)
    {
        for item in self.0.drain(..) {
            item.apply(world);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait that enables loadables to use the [`Splat`] wrapper loadable.
///
/// For example, a UI `Border` could be splatted with `Splat<Border>(Val::Px(2.0))`.
pub trait Splattable
{
    /// The inner value used to splat-construct `Self`.
    type Splat: Typed + FromReflect + GetTypeRegistration + Default + Debug + Clone + PartialEq;

    /// Constructs a full `Self` from a single inner `splat` value.
    fn splat(splat: Self::Splat) -> Self;
}

/// Helper loadable for cases where a loadable can be 'splat-constructed' from a single inner value.
///
/// Note that `Splat<T>` must be manually registered with `register_instruction_type` or
/// `register_command_type` for all `T` that want to use it.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Splat<T: Splattable>(pub T::Splat);

impl<T> Instruction for Splat<T>
where
    T: Splattable + Instruction + Typed + FromReflect + GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        T::splat(self.0).apply(entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        T::revert(entity, world);
    }
}

impl<T> Command for Splat<T>
where
    T: Splattable + Command + TypePath + FromReflect + GetTypeRegistration,
{
    fn apply(self, world: &mut World)
    {
        T::splat(self.0).apply(world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component added to nodes that load scene nodes from cobweb asset files (see [`SceneLoader`]).
#[derive(Component)]
pub(crate) struct HasLoadables;

//-------------------------------------------------------------------------------------------------------------------

/// Entity event emitted when loadables have been updated on an entity.
#[cfg(feature = "hot_reload")]
#[derive(Debug, Default, Copy, Clone, Hash)]
pub struct Loaded;

//-------------------------------------------------------------------------------------------------------------------

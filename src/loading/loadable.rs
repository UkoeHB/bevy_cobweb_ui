use std::fmt::Debug;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

/// Trait representing types that can be loaded from cobweb asset files.
pub trait Loadable:
    Reflect + FromReflect + PartialEq + Clone + Default + Serialize + for<'de> Deserialize<'de>
{
}

impl<T> Loadable for T where
    T: Reflect + FromReflect + PartialEq + Clone + Default + Serialize + for<'de> Deserialize<'de>
{
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for converting `Self` into entity modifications.
///
/// Used by [`register_derived_loadable`](crate::prelude::CobwebAssetRegistrationAppExt::register_derived_loadable).
pub trait ApplyLoadable: Loadable
{
    fn apply(self, entity: Entity, world: &mut World);
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ApplyLoadableExt
{
    /// Calls [`ApplyLoadable::apply`].
    fn apply(&mut self, loadable: impl ApplyLoadable) -> &mut Self;
}

impl ApplyLoadableExt for EntityCommands<'_>
{
    fn apply(&mut self, loadable: impl ApplyLoadable + Send + Sync + 'static) -> &mut Self
    {
        self.add(move |e: Entity, w: &mut World| loadable.apply(e, w));
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper loadable for cases where multiple values of the same type can be loaded.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Multi<T>(Vec<T>);

impl<T: ApplyLoadable + TypePath + FromReflect + GetTypeRegistration> ApplyLoadable for Multi<T>
{
    fn apply(mut self, entity: Entity, world: &mut World)
    {
        for item in self.0.drain(..) {
            item.apply(entity, world);
        }
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

/// Trait that enables derived loadables to use the [`Splat`] wrapper loadable.
///
/// For example, a UI `Border` could be splatted with `Splat<Border>(Val::Px(2.0))`.
pub trait Splattable
{
    /// The inner value used to splat-construct `Self`.
    type Splat: TypePath
        + FromReflect
        + GetTypeRegistration
        + Default
        + Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>;

    /// Constructs a full `Self` from a single inner `splat` value.
    fn splat(splat: Self::Splat) -> Self;
}

/// Helper loadable for cases where a loadable can be 'splat-constructed' from a single inner value.
///
/// Note that `Splat<T>` must be manually registered with `register_derived` or `register_command` for all `T` that
/// want to use it.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Splat<T: Splattable>(T::Splat);

impl<T> ApplyLoadable for Splat<T>
where
    T: Splattable + ApplyLoadable + TypePath + FromReflect + GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        T::splat(self.0).apply(entity, world);
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

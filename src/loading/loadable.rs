use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

/// Trait representing types that can be loaded by [`LoadableSheet`].
pub trait Loadable:
    Reflect + FromReflect + PartialEq + Clone + Default + Serialize + for<'de> Deserialize<'de>
{
}

impl<T> Loadable for T where
    T: Reflect + FromReflect + PartialEq + Clone + Default + Serialize + for<'de> Deserialize<'de>
{
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for converting [`Self`] into entity modifications.
///
/// Used by [`register_derived_loadable`].
pub trait ApplyLoadable: Loadable
{
    fn apply(self, ec: &mut EntityCommands);
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ApplyLoadableExt
{
    /// Calls [`ApplyLoadable::apply`].
    fn apply_loadable(&mut self, loadable: impl ApplyLoadable) -> &mut Self;
}

impl ApplyLoadableExt for EntityCommands<'_>
{
    fn apply_loadable(&mut self, loadable: impl ApplyLoadable) -> &mut Self
    {
        loadable.apply(self);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component added to nodes that load loadables from the loadablesheet.
#[derive(Component)]
pub(crate) struct HasLoadables;

//-------------------------------------------------------------------------------------------------------------------

/// Entity event emitted when loadables have been updated on an entity.
#[derive(Debug, Default, Copy, Clone, Hash)]
pub struct Loaded;

//-------------------------------------------------------------------------------------------------------------------

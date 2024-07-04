use bevy::ecs::system::EntityCommands;
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

/// Trait for applying [`Self`] to a Bevy world.
///
/// Used by [`register_command_loadable`].
pub trait ApplyCommand: Loadable
{
    fn apply(self, c: &mut Commands);
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ApplyCommandExt
{
    /// Calls [`ApplyCommand::apply`].
    fn apply_command(&mut self, loadable: impl ApplyCommand) -> &mut Self;
}

impl ApplyCommandExt for Commands<'_, '_>
{
    fn apply_command(&mut self, loadable: impl ApplyCommand) -> &mut Self
    {
        loadable.apply(self);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper loadable for cases where multiple values of the same type can be loaded.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Multi<T>(Vec<T>);

impl<T: ApplyLoadable + TypePath + FromReflect + GetTypeRegistration> ApplyLoadable for Multi<T>
{
    fn apply(mut self, ec: &mut EntityCommands)
    {
        for item in self.0.drain(..) {
            item.apply(ec);
        }
    }
}

impl<T: ApplyCommand + TypePath + FromReflect + GetTypeRegistration> ApplyCommand for Multi<T>
{
    fn apply(mut self, c: &mut Commands)
    {
        for item in self.0.drain(..) {
            item.apply(c);
        }
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

use crate::*;

use bevy::{ecs::system::EntityCommands, prelude::*};

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering node root entities for style loading.
pub trait NodeLoadingCommandsExt
{
    /// Spawns a new root node registers it to load styles from `style_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn root_node(
        &mut self,
        style_ref: StyleRef,
        callback: impl FnOnce(&mut EntityCommands<'_>, StyleRef)
    ) -> &mut Self;
}

impl NodeLoadingCommandsExt for Commands<'_, '_>
{
    fn root_node(
        &mut self,
        style_ref: StyleRef,
        callback: impl FnOnce(&mut EntityCommands<'_>, StyleRef)
    ) -> &mut Self
    {
        let mut node = self.spawn(NodeBundle::default());
        node.load(style_ref.clone());
        (callback)(&mut node, style_ref);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering child node entities for style loading.
pub trait NodeLoadingEntityCommandsExt
{
    /// Spawns a child node of the current node and registers it to load styles from `style_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the child entity.
    ///
    /// Includes a `child_callback` for interacting with the child entity.
    fn child_node(
        &mut self,
        style_ref: StyleRef,
        child_callback: impl FnOnce(&mut EntityCommands<'_>, StyleRef)
    ) -> EntityCommands<'_>;
}

impl NodeLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn child_node(
        &mut self,
        style_ref: StyleRef,
        child_callback: impl FnOnce(&mut EntityCommands<'_>, StyleRef)
    ) -> EntityCommands<'_>
    {
        let id = self.id();
        let mut commands = self.commands();
        let mut child = commands.spawn(NodeBundle::default());
        child.set_parent(id);
        child.load(style_ref.clone());
        (child_callback)(&mut child, style_ref);
        self.reborrow()
    }
}

//-------------------------------------------------------------------------------------------------------------------

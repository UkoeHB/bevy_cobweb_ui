use crate::*;

use bevy::prelude::*;
use sickle_ui::ui_builder::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering node entities for style loading.
pub trait NodeLoadingExt<'w, 's>
{
    /// Spawns a new node registered to load styles from `style_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn load<'a>(
        &'a mut self,
        style_ref: StyleRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, StyleRef)
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> NodeLoadingExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot>
{
    fn load(
        &mut self,
        style_ref: StyleRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, StyleRef)
    ) -> UiBuilder<'w, 's, '_, Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        node.entity_commands().load_style(style_ref.clone());
        (callback)(&mut node, style_ref);
        node
    }
}

impl<'w, 's> NodeLoadingExt<'w, 's> for UiBuilder<'w, 's, '_, Entity>
{
    fn load(
        &mut self,
        style_ref: StyleRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, StyleRef)
    ) -> UiBuilder<'w, 's, '_, Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        child.entity_commands().load_style(style_ref.clone());
        (callback)(&mut child, style_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

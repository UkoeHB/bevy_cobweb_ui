use bevy::prelude::*;
use sickle_ui::ui_builder::*;

use crate::*;

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
        callback: impl FnOnce(&mut UiBuilder<Entity>, StyleRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        self.load_with(style_ref, NodeBundle::default(), callback)
    }

    /// Spawns a new node registered to load styles from `style_ref`.
    ///
    /// Inserts `bundle` to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn load_with<'a>(
        &'a mut self,
        style_ref: StyleRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, StyleRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> NodeLoadingExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot>
{
    fn load_with<'a>(
        &'a mut self,
        style_ref: StyleRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, StyleRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        let mut node = self.spawn(bundle);
        node.entity_commands().load_style(style_ref.clone());
        (callback)(&mut node, style_ref);
        node
    }
}

impl<'w, 's> NodeLoadingExt<'w, 's> for UiBuilder<'w, 's, '_, Entity>
{
    fn load_with<'a>(
        &'a mut self,
        style_ref: StyleRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, StyleRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        let mut child = self.spawn(bundle);
        child.entity_commands().load_style(style_ref.clone());
        (callback)(&mut child, style_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

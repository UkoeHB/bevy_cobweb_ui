use bevy::prelude::*;
use sickle_ui::theme::DefaultTheme;
use sickle_ui::ui_builder::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering node entities for style loading.
pub trait NodeLoadingExt
{
    /// Spawns a new node registered to load styles from `loadable_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn load(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        self.load_with(loadable_ref, NodeBundle::default(), callback)
    }

    /// Spawns a new node registered to load a theme from `loadable_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback(node_builder, structure_ref, theme_ref)` for interacting with the entity.
    fn load_theme<C: DefaultTheme>(
        &mut self,
        structure_ref: LoadableRef,
        theme_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef, LoadableRef),
    ) -> UiBuilder<Entity>;

    /// Spawns a new node registered to load styles from `loadable_ref`.
    ///
    /// Inserts `bundle` to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn load_with(
        &mut self,
        loadable_ref: LoadableRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>;
}

impl NodeLoadingExt for UiBuilder<'_, '_, UiRoot>
{
    fn load_theme<C: DefaultTheme>(
        &mut self,
        structure_ref: LoadableRef,
        theme_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        node.entity_commands().load_theme::<C>(theme_ref.clone());
        if theme_ref != structure_ref {
            node.entity_commands().load(structure_ref.clone());
        }
        (callback)(&mut node, structure_ref, theme_ref);
        node
    }

    fn load_with(
        &mut self,
        loadable_ref: LoadableRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(bundle);
        node.entity_commands().load(loadable_ref.clone());
        (callback)(&mut node, loadable_ref);
        node
    }
}

impl NodeLoadingExt for UiBuilder<'_, '_, Entity>
{
    fn load_theme<C: DefaultTheme>(
        &mut self,
        structure_ref: LoadableRef,
        theme_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        child.entity_commands().load_theme::<C>(theme_ref.clone());
        if theme_ref != structure_ref {
            child.entity_commands().load(structure_ref.clone());
        }
        (callback)(&mut child, structure_ref, theme_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }

    fn load_with(
        &mut self,
        loadable_ref: LoadableRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(bundle);
        child.entity_commands().load(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

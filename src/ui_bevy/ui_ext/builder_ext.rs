use bevy::prelude::*;
use sickle_ui::theme::DefaultTheme;
use sickle_ui::ui_builder::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering node entities for style loading.
pub trait NodeLoadingExt<'w, 's>
{
    /// Spawns a new node registered to load styles from `loadable_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn load<'a>(
        &'a mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        self.load_with(loadable_ref, NodeBundle::default(), callback)
    }

    /// Spawns a new node registered to load a theme from `loadable_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn load_theme<'a, C: DefaultTheme>(
        &'a mut self,
        theme_ref: LoadableRef,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>;

    /// Spawns a new node registered to load styles from `loadable_ref`.
    ///
    /// Inserts `bundle` to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    fn load_with<'a>(
        &'a mut self,
        loadable_ref: LoadableRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>;
}

impl<'w, 's> NodeLoadingExt<'w, 's> for UiBuilder<'w, 's, '_, UiRoot>
{
    fn load_theme<'a, C: DefaultTheme>(
        &'a mut self,
        theme_ref: LoadableRef,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        node.entity_commands().load_theme::<C>(theme_ref.clone());
        if theme_ref != loadable_ref {
            node.entity_commands().load(loadable_ref.clone());
        }
        (callback)(&mut node, loadable_ref);
        node
    }

    fn load_with<'a>(
        &'a mut self,
        loadable_ref: LoadableRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        let mut node = self.spawn(bundle);
        node.entity_commands().load(loadable_ref.clone());
        (callback)(&mut node, loadable_ref);
        node
    }
}

impl<'w, 's> NodeLoadingExt<'w, 's> for UiBuilder<'w, 's, '_, Entity>
{
    fn load_theme<'a, C: DefaultTheme>(
        &'a mut self,
        theme_ref: LoadableRef,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        child.entity_commands().load_theme::<C>(theme_ref.clone());
        if theme_ref != loadable_ref {
            child.entity_commands().load(loadable_ref.clone());
        }
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }

    fn load_with<'a>(
        &'a mut self,
        loadable_ref: LoadableRef,
        bundle: impl Bundle,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<'w, 's, 'a, Entity>
    {
        let mut child = self.spawn(bundle);
        child.entity_commands().load(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

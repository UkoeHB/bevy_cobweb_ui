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
    ) -> UiBuilder<Entity>;
    /// Loads a theme into the current entity from `loadable_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> UiBuilder<Entity>;
    /// Spawns a new node registered to load a theme from `loadable_ref`.
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback(node_builder, loadable_ref)` for interacting with the entity.
    fn load_theme_with<C: DefaultTheme>(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>;
    /// Loads a subtheme into the current entity from `loadable_ref`.
    ///
    /// The subtheme will be tied to `Ctx`, and can be mapped to an entity in a theme instantiation using
    /// [`UiContext`].
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> UiBuilder<Entity>;
    /// Spawns a new node registered to load a subtheme from `loadable_ref`.
    ///
    /// The subtheme will be tied to `Ctx`, and can be mapped to an entity in a theme instantiation using
    /// [`UiContext`].
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback(node_builder, loadable_ref)` for interacting with the entity.
    fn load_subtheme_with<C: DefaultTheme, Ctx: TypeName>(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>;
}

impl NodeLoadingExt for UiBuilder<'_, UiRoot>
{
    fn load(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        node.entity_commands().load(loadable_ref.clone());
        (callback)(&mut node, loadable_ref);
        node
    }

    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> UiBuilder<Entity>
    {
        self.load_theme_with::<C>(loadable_ref, |_, _| {})
    }

    fn load_theme_with<C: DefaultTheme>(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        node.entity_commands().load_theme::<C>(loadable_ref.clone());
        (callback)(&mut node, loadable_ref);
        node
    }

    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> UiBuilder<Entity>
    {
        self.load_subtheme_with::<C, Ctx>(loadable_ref, |_, _| {})
    }

    fn load_subtheme_with<C: DefaultTheme, Ctx: TypeName>(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        node.entity_commands()
            .load_subtheme::<C, Ctx>(loadable_ref.clone());
        (callback)(&mut node, loadable_ref);
        node
    }
}

impl NodeLoadingExt for UiBuilder<'_, Entity>
{
    fn load(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        child.entity_commands().load(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }

    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> UiBuilder<Entity>
    {
        self.entity_commands().load_theme::<C>(loadable_ref.clone());
        let id = self.id();
        self.commands().ui_builder(id)
    }

    fn load_theme_with<C: DefaultTheme>(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        child
            .entity_commands()
            .load_theme::<C>(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }

    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> UiBuilder<Entity>
    {
        self.entity_commands()
            .load_subtheme::<C, Ctx>(loadable_ref.clone());
        let id = self.id();
        self.commands().ui_builder(id)
    }

    fn load_subtheme_with<C: DefaultTheme, Ctx: TypeName>(
        &mut self,
        loadable_ref: LoadableRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        child
            .entity_commands()
            .load_subtheme::<C, Ctx>(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

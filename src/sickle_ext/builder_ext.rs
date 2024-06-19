use bevy::prelude::*;
use sickle_ui::theme::DefaultTheme;
use sickle_ui::ui_builder::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for building node entities with style loading.
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
    /// Spawns a new node registered to load a theme from `loadable_ref`.
    ///
    /// The parameter `entity` will be set with the new node's entity id. This is often inserted to widget
    /// components for use in [`UiContexts`](UiContext).
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback(node_builder, loadable_ref)` for interacting with the entity.
    fn load_with_theme<C: DefaultTheme>(
        &mut self,
        loadable_ref: LoadableRef,
        entity: &mut Entity,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>;
    /// Spawns a new node registered to load a subtheme from `loadable_ref`.
    ///
    /// The parameter `entity` will be set with the new node's entity id. This is often inserted to widget
    /// components for use in [`UiContexts`](UiContext).
    ///
    /// The subtheme will be tied to `Ctx`, and can be mapped to an entity in a theme instantiation using
    /// [`UiContext`].
    ///
    /// Inserts a [`NodeBundle::default()`] to the entity.
    ///
    /// Includes a `callback(node_builder, loadable_ref)` for interacting with the entity.
    fn load_with_subtheme<C: DefaultTheme, Ctx: TypeName>(
        &mut self,
        loadable_ref: LoadableRef,
        entity: &mut Entity,
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

    fn load_with_theme<C: DefaultTheme>(
        &mut self,
        loadable_ref: LoadableRef,
        entity: &mut Entity,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        *entity = node.id();
        node.entity_commands().load_theme::<C>(loadable_ref.clone());
        (callback)(&mut node, loadable_ref);
        node
    }

    fn load_with_subtheme<C: DefaultTheme, Ctx: TypeName>(
        &mut self,
        loadable_ref: LoadableRef,
        entity: &mut Entity,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(NodeBundle::default());
        *entity = node.id();
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

    fn load_with_theme<C: DefaultTheme>(
        &mut self,
        loadable_ref: LoadableRef,
        entity: &mut Entity,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        *entity = child.id();
        child
            .entity_commands()
            .load_theme::<C>(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }

    fn load_with_subtheme<C: DefaultTheme, Ctx: TypeName>(
        &mut self,
        loadable_ref: LoadableRef,
        entity: &mut Entity,
        callback: impl FnOnce(&mut UiBuilder<Entity>, LoadableRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(NodeBundle::default());
        *entity = child.id();
        child
            .entity_commands()
            .load_subtheme::<C, Ctx>(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for dealing with loadable themes.
pub trait LoadableThemeBuilderExt
{
    /// See [`ThemeLoadingEntityCommandsExt::set_theme`].
    fn set_theme<C: DefaultTheme>(&mut self) -> &mut Self;
    /// See [`ThemeLoadingEntityCommandsExt::set_subtheme`].
    fn set_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self) -> &mut Self;
    /// See [`ThemeLoadingEntityCommandsExt::prepare_theme`].
    fn prepare_theme<C: DefaultTheme>(&mut self) -> &mut Self;
    /// See [`ThemeLoadingEntityCommandsExt::load_theme`].
    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> &mut Self;
    /// See [`ThemeLoadingEntityCommandsExt::load_subtheme`].
    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> &mut Self;
}

impl LoadableThemeBuilderExt for UiBuilder<'_, Entity>
{
    fn set_theme<C: DefaultTheme>(&mut self) -> &mut Self
    {
        self.entity_commands().set_theme::<C>();
        self
    }

    fn set_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self) -> &mut Self
    {
        self.entity_commands().set_subtheme::<C, Ctx>();
        self
    }

    fn prepare_theme<C: DefaultTheme>(&mut self) -> &mut Self
    {
        self.entity_commands().prepare_theme::<C>();
        self
    }

    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        self.entity_commands().load_theme::<C>(loadable_ref.clone());
        self
    }

    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        self.entity_commands()
            .load_subtheme::<C, Ctx>(loadable_ref.clone());
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

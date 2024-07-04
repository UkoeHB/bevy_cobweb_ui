use std::any::type_name;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use sickle_ui::theme::{DefaultTheme, UiContext};
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
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
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
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
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
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
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
    /// Provides access to the entity of a subtheme of a themed widget.
    ///
    /// The callback paramaters are: commands, core entity where `C` is found, child entity where `Ctx` is found.
    ///
    /// Type `C` should be a themed component on the current entity. Type `Ctx` should be a subtheme accessible
    /// via [`UiContext`] on `C`.
    fn edit_child<C: Component + UiContext, Ctx: TypeName>(
        &mut self,
        callback: impl FnOnce(&mut Commands, Entity, Entity) + Send + Sync + 'static,
    ) -> &mut Self;
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

    fn edit_child<C: Component + UiContext, Ctx: TypeName>(
        &mut self,
        callback: impl FnOnce(&mut Commands, Entity, Entity) + Send + Sync + 'static,
    ) -> &mut Self
    {
        let entity = self.id();
        self.commands().add(move |world: &mut World| {
            let Some(themed_component) = world.get::<C>(entity) else {
                tracing::warn!("failed editing child w/ subtheme {} on entity {entity:?} with themed component {}, \
                    entity is missing or does not have {}", type_name::<Ctx>(), type_name::<C>(), type_name::<C>());
                return;
            };

            let Ok(child_entity) = themed_component.get(Ctx::NAME) else {
                tracing::warn!("failed editing child w/ subtheme {} on entity {entity:?} with themed component {}, \
                    subtheme is not available in entity's UiContext",
                    type_name::<Ctx>(), type_name::<C>());
                return;
            };

            let mut c = world.commands();
            (callback)(&mut c, entity, child_entity);
            world.flush();
        });
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl scene_traits::SceneNodeLoader for UiBuilder<'_, UiRoot>
{
    type Loaded<'a> = UiBuilder<'a, Entity>;

    fn commands(&mut self) -> Commands
    {
        self.commands().reborrow()
    }

    fn scene_parent_entity(&self) -> Option<Entity>
    {
        None
    }

    fn initialize_scene_node(ec: &mut EntityCommands)
    {
        ec.insert(NodeBundle::default());
    }

    fn loaded_scene_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Loaded<'a>
    {
        commands.ui_builder(entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl scene_traits::SceneNodeLoader for UiBuilder<'_, Entity>
{
    type Loaded<'a> = UiBuilder<'a, Entity>;

    fn commands(&mut self) -> Commands
    {
        self.commands().reborrow()
    }

    fn scene_parent_entity(&self) -> Option<Entity>
    {
        Some(self.id())
    }

    fn initialize_scene_node(ec: &mut EntityCommands)
    {
        ec.insert(NodeBundle::default());
    }

    fn loaded_scene_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Loaded<'a>
    {
        commands.ui_builder(entity)
    }
}

impl<'a> scene_traits::LoadedSceneBuilder<'a> for UiBuilder<'a, Entity> {}

//-------------------------------------------------------------------------------------------------------------------

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use crate::prelude::*;
use crate::sickle_ext::ui_builder::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for building node entities with style loading.
pub trait NodeLoadingExt
{
    /// Spawns a new node registered to load styles from `loadable_ref`.
    ///
    /// Inserts a [`Node::default()`] to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn load(
        &mut self,
        loadable_ref: SceneRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, SceneRef),
    ) -> UiBuilder<Entity>;
}

impl NodeLoadingExt for UiBuilder<'_, UiRoot>
{
    fn load(
        &mut self,
        loadable_ref: SceneRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, SceneRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(Node::default());
        node.entity_commands().load(loadable_ref.clone());
        (callback)(&mut node, loadable_ref);
        node
    }
}

impl NodeLoadingExt for UiBuilder<'_, Entity>
{
    fn load(
        &mut self,
        loadable_ref: SceneRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, SceneRef),
    ) -> UiBuilder<Entity>
    {
        let mut child = self.spawn(Node::default());
        child.entity_commands().load(loadable_ref.clone());
        (callback)(&mut child, loadable_ref);
        let id = self.id();
        self.commands().ui_builder(id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for dealing with widget controls.
pub trait ControlBuilderExt
{
    /// Provides access to a sub-entity of a widget.
    ///
    /// Does nothing if the current entity doesn't have `ControlMap` (see [`ControlRoot`]), or if there is no
    /// child entity with a [`ControlLabel`] equal to the requested `child`.
    ///
    /// The callback paramaters are: commands, current entity, child entity.
    fn edit_child(
        &mut self,
        child: &'static str,
        callback: impl FnOnce(&mut Commands, Entity, Entity) + Send + Sync + 'static,
    ) -> &mut Self;
}

impl ControlBuilderExt for UiBuilder<'_, Entity>
{
    fn edit_child(
        &mut self,
        child: &'static str,
        callback: impl FnOnce(&mut Commands, Entity, Entity) + Send + Sync + 'static,
    ) -> &mut Self
    {
        let entity = self.id();
        self.commands().queue(move |world: &mut World| {
            let Some(control_map) = world.get::<ControlMap>(entity) else {
                tracing::warn!(
                    "failed editing child {child} of entity {entity:?}, \
                    entity is missing or does not have ControlMap (see ControlRoot)"
                );
                return;
            };

            let Some(child_entity) = control_map.get_entity(child) else {
                tracing::warn!(
                    "failed editing child {child} of entity {entity:?}, \
                    entity's ControlMap does not have an entry for {child} (see ControlLabel)"
                );
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
        ec.insert(Node::default());
    }

    fn loaded_scene_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Loaded<'a>
    {
        commands.ui_builder(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Loaded<'_>
    {
        self.commands().ui_builder(entity)
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
        ec.insert(Node::default());
    }

    fn loaded_scene_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Loaded<'a>
    {
        commands.ui_builder(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Loaded<'_>
    {
        self.commands().ui_builder(entity)
    }
}

impl<'a> scene_traits::LoadedSceneBuilder<'a> for UiBuilder<'a, Entity> {}

//-------------------------------------------------------------------------------------------------------------------

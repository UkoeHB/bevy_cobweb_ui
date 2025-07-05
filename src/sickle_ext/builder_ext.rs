use bevy::prelude::*;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for building scene node entities.
pub trait NodeSpawningExt
{
    /// Spawns a new entity and builds it with the scene node at `scene_ref`.
    ///
    /// Inserts a [`Node::default()`] to the entity.
    ///
    /// Includes a `callback` for interacting with the entity.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    ///
    /// Will do nothing if the current entity does not exist.
    fn spawn_scene_node(
        &mut self,
        scene_ref: SceneRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, SceneRef),
    ) -> UiBuilder<Entity>;
}

impl NodeSpawningExt for UiBuilder<'_, UiRoot>
{
    fn spawn_scene_node(
        &mut self,
        scene_ref: SceneRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, SceneRef),
    ) -> UiBuilder<Entity>
    {
        let mut node = self.spawn(Node::default());
        node.entity_commands().build(scene_ref.clone());
        (callback)(&mut node, scene_ref);
        node
    }
}

impl NodeSpawningExt for UiBuilder<'_, Entity>
{
    fn spawn_scene_node(
        &mut self,
        scene_ref: SceneRef,
        callback: impl FnOnce(&mut UiBuilder<Entity>, SceneRef),
    ) -> UiBuilder<Entity>
    {
        let id = self.id();
        if self.commands().get_entity(id).is_err() {
            return self.reborrow();
        }
        let mut child = self.spawn(Node::default());
        child.entity_commands().build(scene_ref.clone());
        (callback)(&mut child, scene_ref);
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
    /// child entity with a [`ControlMember`] ID equal to the requested `child`.
    ///
    /// The callback paramaters are: commands, current entity, child entity.
    fn edit_control_child(
        &mut self,
        child: &'static str,
        callback: impl FnOnce(&mut Commands, Entity, Entity) + Send + Sync + 'static,
    ) -> &mut Self;
}

impl ControlBuilderExt for UiBuilder<'_, Entity>
{
    fn edit_control_child(
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
                    entity's ControlMap does not have an entry for {child} (see ControlMember)"
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

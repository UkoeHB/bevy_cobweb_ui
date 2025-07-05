use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use cob_sickle_ui_scaffold::*;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl scene_traits::SceneNodeBuilder for UiBuilder<'_, UiRoot>
{
    type Builder<'a> = UiBuilder<'a, Entity>;

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
        ec.apply(FlexNode::default());
    }

    fn scene_node_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Builder<'a>
    {
        commands.ui_builder(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Builder<'_>
    {
        self.commands().ui_builder(entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl scene_traits::SceneNodeBuilder for UiBuilder<'_, Entity>
{
    type Builder<'a> = UiBuilder<'a, Entity>;

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
        ec.apply(FlexNode::default());
    }

    fn scene_node_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Builder<'a>
    {
        commands.ui_builder(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Builder<'_>
    {
        self.commands().ui_builder(entity)
    }
}

impl<'a> scene_traits::SceneNodeBuilderOuter<'a> for UiBuilder<'a, Entity> {}

//-------------------------------------------------------------------------------------------------------------------

impl InstructionExt for UiBuilder<'_, Entity>
{
    fn apply(&mut self, instruction: impl Instruction + Send + Sync + 'static) -> &mut Self
    {
        let id = self.id();
        if let Ok(mut ec) = self.commands().get_entity(id) {
            ec.apply(instruction);
        }
        self
    }

    fn revert<T: Instruction>(&mut self) -> &mut Self
    {
        let id = self.id();
        if let Ok(mut ec) = self.commands().get_entity(id) {
            ec.revert::<T>();
        }
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Shortcut for using [`SceneRef`] as a function parameter.
pub type UiSceneHandle<'a> = SceneHandle<'a, UiBuilder<'a, Entity>>;

//-------------------------------------------------------------------------------------------------------------------

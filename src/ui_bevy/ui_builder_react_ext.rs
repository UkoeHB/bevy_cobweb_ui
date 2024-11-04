use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui_scaffold::prelude::UiBuilder;

use crate::prelude::*;
//-------------------------------------------------------------------------------------------------------------------

impl InstructionExt for UiBuilder<'_, Entity>
{
    fn apply(&mut self, instruction: impl Instruction + Send + Sync + 'static) -> &mut Self
    {
        self.entity_commands().apply(instruction);
        self
    }

    fn revert<T: Instruction>(&mut self) -> &mut Self
    {
        self.entity_commands().revert::<T>();
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering reactors for node entities using [`UiBuilder`].
pub trait UiBuilderReactExt
{
    /// Mirrors [`ReactEntityCommandsExt::add_world_reactor`].
    fn add_reactor<T: EntityWorldReactor>(&mut self, data: T::Local);

    /// Mirrors [`UiReactEntityCommandsExt::insert_reactive`].
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self;

    /// Mirrors [`ReactEntityCommandsExt::react`].
    fn react(&mut self) -> ReactCommands<'_, '_>;

    /// Mirrors [`UiReactEntityCommandsExt::on_event`].
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>;

    /// Mirrors [`UiReactEntityCommandsExt::despawn_on_event`].
    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::despawn_on_broadcast`].
    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::update_on`].
    fn update_on<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        triggers: impl ReactionTriggerBundle,
        reactor: impl FnOnce(Entity) -> C,
    ) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::update`].
    fn update<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        reactor: impl FnOnce(Entity) -> C,
    ) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::modify`].
    fn modify(&mut self, callback: impl FnMut(EntityCommands) + Send + Sync + 'static) -> &mut Self;
}

impl UiBuilderReactExt for UiBuilder<'_, Entity>
{
    fn add_reactor<T: EntityWorldReactor>(&mut self, data: T::Local)
    {
        self.entity_commands().add_world_reactor::<T>(data);
    }

    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self
    {
        self.entity_commands().insert_reactive(component);
        self
    }

    fn react(&mut self) -> ReactCommands<'_, '_>
    {
        self.commands().react()
    }

    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>
    {
        OnEventExt::new(self.entity_commands())
    }

    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        self.entity_commands().despawn_on_event::<T>();
        self
    }

    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        self.entity_commands().despawn_on_broadcast::<T>();
        self
    }

    fn update_on<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        triggers: impl ReactionTriggerBundle,
        reactor: impl FnOnce(Entity) -> C,
    ) -> &mut Self
    {
        self.entity_commands().update_on(triggers, reactor);
        self
    }

    fn update<M, C: IntoSystem<(), (), M> + Send + Sync + 'static>(
        &mut self,
        reactor: impl FnOnce(Entity) -> C,
    ) -> &mut Self
    {
        self.entity_commands().update_on((), reactor);
        self
    }

    fn modify(&mut self, callback: impl FnMut(EntityCommands) + Send + Sync + 'static) -> &mut Self
    {
        self.entity_commands().modify(callback);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

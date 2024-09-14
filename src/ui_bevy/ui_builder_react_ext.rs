use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::ui_builder::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering reactors for node entities using [`UiBuilder`].
pub trait UiBuilderReactExt
{
    /// Mirrors [`ReactEntityCommandsExt::add_reactor`].
    fn add_reactor<T: EntityWorldReactor>(&mut self, data: T::Local);

    /// Mirrors [`UiReactEntityCommandsExt::insert_reactive`].
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::insert_derived`].
    fn insert_derived<T: ApplyLoadable>(&mut self, value: T) -> &mut Self;

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

    fn insert_derived<T: ApplyLoadable>(&mut self, value: T) -> &mut Self
    {
        self.entity_commands().insert_derived(value);
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
}

//-------------------------------------------------------------------------------------------------------------------

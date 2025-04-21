use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

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

/// Helper trait for registering reactors for node entities using [`UiBuilder`].
pub trait UiBuilderReactExt
{
    /// Mirrors [`ReactEntityCommandsExt::add_world_reactor`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn add_reactor<T: EntityWorldReactor>(&mut self, data: T::Local);

    /// Mirrors [`UiReactEntityCommandsExt::insert_reactive`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self;

    /// Mirrors [`ReactEntityCommandsExt::react`].
    fn react(&mut self) -> ReactCommands<'_, '_>;

    /// Mirrors [`UiReactEntityCommandsExt::on_event`].
    ///
    /// The `OneEventExt` will do nothing if the entity doesn't exist.
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>;

    /// Mirrors [`UiReactEntityCommandsExt::despawn_on_event`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::despawn_on_broadcast`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::reactor`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn reactor<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static;

    /// Mirrors [`UiReactEntityCommandsExt::update`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn update<M, R: CobwebResult, C: IntoSystem<TargetId, R, M> + Send + Sync + 'static>(
        &mut self,
        reactor: C,
    ) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::update_on`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static;

    /// Updates text with a static string using [`Self::update`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn update_text(&mut self, text: impl Into<String>) -> &mut Self;

    /// Mirrors [`UiReactEntityCommandsExt::modify`].
    ///
    /// Does nothing if the entity doesn't exist.
    fn modify(&mut self, callback: impl FnMut(EntityCommands) + Send + Sync + 'static) -> &mut Self;
}

impl UiBuilderReactExt for UiBuilder<'_, Entity>
{
    fn add_reactor<T: EntityWorldReactor>(&mut self, data: T::Local)
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.add_world_reactor::<T>(data);
        }
    }

    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.insert_reactive(component);
        }
        self
    }

    fn react(&mut self) -> ReactCommands<'_, '_>
    {
        self.commands().react()
    }

    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>
    {
        let id = self.id();
        OnEventExt::new(self.commands().reborrow(), id)
    }

    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.despawn_on_event::<T>();
        }
        self
    }

    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.despawn_on_broadcast::<T>();
        }
        self
    }

    fn reactor<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static,
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.reactor(triggers, reactor);
        }
        self
    }

    fn update<M, R: CobwebResult, C: IntoSystem<TargetId, R, M> + Send + Sync + 'static>(
        &mut self,
        reactor: C,
    ) -> &mut Self
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.update_on((), reactor);
        }
        self
    }

    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static,
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.update_on(triggers, reactor);
        }
        self
    }

    fn update_text(&mut self, text: impl Into<String>) -> &mut Self
    {
        let text = text.into();
        self.update(move |id: TargetId, mut e: TextEditor| {
            write_text!(e, *id, "{}", text.as_str());
        })
    }

    fn modify(&mut self, callback: impl FnMut(EntityCommands) + Send + Sync + 'static) -> &mut Self
    {
        let id = self.id();
        if let Ok(mut emut) = self.commands().get_entity(id) {
            emut.modify(callback);
        }
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

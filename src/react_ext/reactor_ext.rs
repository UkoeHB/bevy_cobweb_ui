use std::any::TypeId;
use std::marker::PhantomData;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
fn register_update_on_reactor<Triggers: ReactionTriggerBundle>(
    In((entity, syscommand, triggers)): In<(Entity, SystemCommand, Triggers)>,
    mut c: Commands,
    loaded: Query<(), With<HasLoadables>>,
)
{
    // If there are no triggers then we should despawn the reactor immediately.
    let is_loaded = loaded.contains(entity);
    if !is_loaded && (TypeId::of::<Triggers>() == TypeId::of::<()>()) {
        c.queue(syscommand);
        c.queue(move |world: &mut World| {
            world.despawn(*syscommand);
        });
        return;
    }

    // Otherwise, prepare the reactor.
    let revoke_token = if is_loaded {
        let triggers = (triggers, entity_event::<SceneNodeBuilt>(entity));

        c.react()
            .with(triggers, syscommand, ReactorMode::Revokable)
            .unwrap()
    } else {
        c.react()
            .with(triggers, syscommand, ReactorMode::Revokable)
            .unwrap()
    };

    //todo: more efficient cleanup mechanism
    // - need to be careful here, if we make the system a child of the target then despawn_descendants will kill it
    cleanup_reactor_on_despawn(&mut c, entity, revoke_token);

    // Run the system to apply it.
    c.queue(syscommand);
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(not(feature = "hot_reload"))]
fn register_update_on_reactor<Triggers: ReactionTriggerBundle>(
    c: &mut Commands,
    entity: Entity,
    syscommand: SystemCommand,
    triggers: Triggers,
)
{
    // If there are no triggers then we should despawn the reactor immediately after running it.
    if TypeId::of::<Triggers>() == TypeId::of::<()>() {
        c.queue(syscommand);
        c.queue(move |world: &mut World| {
            world.despawn(*syscommand);
        });
        return;
    }

    // Otherwise, prepare reactor.
    let revoke_token = c
        .react()
        .with(triggers, syscommand, ReactorMode::Revokable)
        .unwrap();

    //todo: more efficient cleanup mechanism
    cleanup_reactor_on_despawn(c, entity, revoke_token);

    // Run the system to apply it.
    c.queue(syscommand);
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper struct returned by [`on_event`](UiReactEntityCommandsExt::on_event).
///
/// Call [`Self::r`] to add a reactor.
pub struct OnEventExt<'a, T: Send + Sync + 'static>
{
    c: Commands<'a, 'a>,
    entity: Entity,
    _p: PhantomData<T>,
}

impl<'a, T: Send + Sync + 'static> OnEventExt<'a, T>
{
    pub(crate) fn new(c: Commands<'a, 'a>, entity: Entity) -> OnEventExt<'a, T>
    {
        Self { c, entity, _p: PhantomData }
    }

    /// Adds a reactor to an [`on_event`](UiReactEntityCommandsExt::on_event) request.
    ///
    /// Does nothing if the target entity doesn't exist.
    pub fn r<R: ReactorResult, M>(mut self, callback: impl IntoSystem<(), R, M> + Send + Sync + 'static)
    {
        self.c.react().on(entity_event::<T>(self.entity), callback);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// [`SystemInput`] implementation for use in [`UiReactEntityCommandsExt::update_on`].
#[derive(Debug)]
pub struct UpdateId(pub Entity);

impl SystemInput for UpdateId
{
    type Param<'i> = UpdateId;
    type Inner<'i> = Entity;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_>
    {
        UpdateId(this)
    }
}

impl Deref for UpdateId
{
    type Target = Entity;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl DerefMut for UpdateId
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for managing COB scene node entities.
pub trait UiReactEntityCommandsExt
{
    /// Inserts a reactive component to the entity.
    ///
    /// The component can be accessed with the [`React<T>`] component, or with the
    /// [`Reactive`]/[`ReactiveMut`] system parameters.
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self;

    /// Registers an [`entity_event`] reactor for the current entity.
    ///
    /// Use [`OnEventExt::r`] to register the reactor.
    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>;

    /// Recursively despawns the current entity on entity event `T`.
    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Recursively despawns the current entity on broadcast event `T`.
    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self;

    /// Updates an entity with a oneshot system.
    ///
    /// The system runs:
    /// - Immediately after being registered.
    /// - When an entity with the internal `HasLoadables` component receives `Loaded` events (`hot_reload` feature
    ///   only).
    fn update<M, R: ReactorResult, C: IntoSystem<UpdateId, R, M> + Send + Sync + 'static>(
        &mut self,
        reactor: C,
    ) -> &mut Self;

    /// Updates an entity with a reactor system.
    ///
    /// The system runs:
    /// - Immediately after being registered.
    /// - Whenever the triggers fire.
    /// - When an entity with the internal `HasLoadables` component receives `Loaded` events (`hot_reload` feature
    ///   only).
    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: ReactorResult,
        C: IntoSystem<UpdateId, R, M> + Send + Sync + 'static;

    /// Provides access to entity commands for the entity.
    ///
    /// Useful if you need to insert/modify components or instructions on an entity that has COB instructions that
    /// overlap with those components/instructions. For example, you might have a COB instruction
    /// [`FlexNode`], and then in rust manually apply the instruction [`Width`] (which will modify
    /// `FlexNode`). If the `FlexNode` instruction is changed and hot reloaded, then the `Width`
    /// instruction's effects will be erased. However, if you use `modify` to apply the `Width` instruction,
    /// then it will be re-applied whenever the entity hot-reloads loaded instructions.
    ///
    /// The callback runs:
    /// - Immediately after being registered.
    /// - When an entity with the internal `HasLoadables` component receives `Loaded` events (`hot_reload` feature
    ///   only).
    fn modify(&mut self, callback: impl FnMut(EntityCommands) + Send + Sync + 'static) -> &mut Self;
}

impl UiReactEntityCommandsExt for EntityCommands<'_>
{
    fn insert_reactive<T: ReactComponent>(&mut self, component: T) -> &mut Self
    {
        let id = self.id();
        self.commands().react().insert(id, component);
        self
    }

    fn on_event<T: Send + Sync + 'static>(&mut self) -> OnEventExt<'_, T>
    {
        let id = self.id();
        OnEventExt::new(self.commands(), id)
    }

    fn despawn_on_event<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        let entity = self.id();
        self.on_event::<T>().r(move |mut c: Commands| {
            c.get_entity(entity).map(|e| e.despawn_recursive());
        });
        self
    }

    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        let entity = self.id();
        self.react().once(broadcast::<T>(), move |mut c: Commands| {
            c.get_entity(entity).map(|e| e.despawn_recursive());
        });
        self
    }

    fn update<M, R: ReactorResult, C: IntoSystem<UpdateId, R, M> + Send + Sync + 'static>(
        &mut self,
        reactor: C,
    ) -> &mut Self
    {
        self.update_on((), reactor)
    }

    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: ReactorResult,
        C: IntoSystem<UpdateId, R, M> + Send + Sync + 'static,
    {
        let id = self.id();
        let mut reactor = RawCallbackSystem::new(reactor);
        let callback = move |world: &mut World| {
            let result = reactor.run_with_cleanup(world, id, |_| {});
            result.handle(world);
        };
        let syscommand = self.commands().spawn_system_command(callback);
        #[cfg(feature = "hot_reload")]
        {
            self.commands()
                .syscall((id, syscommand, triggers), register_update_on_reactor);
        }
        #[cfg(not(feature = "hot_reload"))]
        {
            register_update_on_reactor(&mut self.commands(), id, syscommand, triggers);
        }

        self
    }

    fn modify(&mut self, mut callback: impl FnMut(EntityCommands) + Send + Sync + 'static) -> &mut Self
    {
        self.update_on((), move |id: UpdateId, mut c: Commands| {
            let Some(ec) = c.get_entity(*id) else { return };
            (callback)(ec)
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ReactorExtPlugin;

impl Plugin for ReactorExtPlugin
{
    fn build(&self, _app: &mut App) {}
}

//-------------------------------------------------------------------------------------------------------------------

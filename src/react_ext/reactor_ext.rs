use std::any::TypeId;
use std::marker::PhantomData;

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn register_reactor<Triggers: ReactionTriggerBundle>(
    c: &mut Commands,
    entity: Entity,
    syscommand: SystemCommand,
    triggers: Triggers,
)
{
    let revoke_token = c
        .react()
        .with(triggers, syscommand, ReactorMode::Revokable)
        .unwrap();

    c.entity(*syscommand)
        .insert((ReactorAttachedTo(entity), RevokeTokenCache(revoke_token)));
}

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

    c.entity(*syscommand)
        .insert((ReactorAttachedTo(entity), RevokeTokenCache(revoke_token)));

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

    c.entity(*syscommand)
        .insert((ReactorAttachedTo(entity), RevokeTokenCache(revoke_token)));

    // Run the system to apply it.
    c.queue(syscommand);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct RevokeTokenCache(RevokeToken);

/// Custom relationship to attach reactors to target entities.
/// We don't use `ChildOf` because we don't want the reactors do be despawned when `.despawn_related::<Children>()`
/// is called.
#[derive(Component)]
#[relationship(relationship_target = AttachedReactors)]
struct ReactorAttachedTo(Entity);

#[derive(Component)]
#[relationship_target(relationship = ReactorAttachedTo)]
#[component(on_despawn = revoke_tokens)]
struct AttachedReactors(Vec<Entity>);

/// Revoking the attached entities' tokens will cause the entities to be garbage collected (despawned). We
/// don't need to despawn them here.
fn revoke_tokens(mut w: DeferredWorld, context: HookContext)
{
    let (entities, mut c) = w.entities_and_commands();
    let reactors = entities
        .get(context.entity)
        .unwrap()
        .get::<AttachedReactors>()
        .unwrap();

    for reactor in reactors.iter() {
        let Ok(entity) = entities.get(reactor) else { continue };
        let Some(cache) = entity.get::<RevokeTokenCache>() else { continue };
        c.react().revoke(cache.0.clone());
    }
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
    pub fn r<R: CobwebResult, M>(mut self, callback: impl IntoSystem<(), R, M> + Send + Sync + 'static)
    {
        self.c.react().on(entity_event::<T>(self.entity), callback);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// [`SystemInput`] implementation for use in [`UiReactEntityCommandsExt`] methods.
///
/// Contains the entity targeted by a system callback.
#[derive(Debug, Copy, Clone)]
pub struct TargetId(pub Entity);

impl SystemInput for TargetId
{
    type Param<'i> = TargetId;
    type Inner<'i> = Entity;

    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_>
    {
        TargetId(this)
    }
}

impl Deref for TargetId
{
    type Target = Entity;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl DerefMut for TargetId
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

impl Into<Entity> for TargetId
{
    fn into(self) -> Entity
    {
        self.0
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

    /// Attaches a reactor to an entity.
    ///
    /// The system runs:
    /// - Whenever the triggers fire.
    ///
    /// The reactor will be cleaned up when the entity is despawned.
    ///
    /// Useful if, for example, you want the entity to react on event broadcast (so [`Self::on_event`] won't work).
    ///
    /// Use [`Self::update_on`] if you want the callback to always run on spawn (and then every time the
    /// entity is hot-reloaded).
    fn reactor<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static;

    /// Updates an entity with a oneshot system.
    ///
    /// The system runs:
    /// - Immediately after being registered.
    /// - When an entity with the internal `HasLoadables` component receives `SceneNodeBuilt` events (`hot_reload`
    ///   feature only).
    fn update<M, R: CobwebResult, C: IntoSystem<TargetId, R, M> + Send + Sync + 'static>(
        &mut self,
        callback: C,
    ) -> &mut Self;

    /// Updates an entity with a reactor system.
    ///
    /// The system runs:
    /// - Immediately after being registered.
    /// - Whenever the triggers fire.
    /// - When an entity with the internal `HasLoadables` component receives `SceneNodeBuilt` events (`hot_reload`
    ///   feature only).
    ///
    /// Use [`Self::reactor`] if you only want the callback to run in reaction to triggers.
    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static;

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
    /// - When an entity with the internal `HasLoadables` component receives `SceneNodeBuilt` events (`hot_reload`
    ///   feature only).
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
            let _ = c.get_entity(entity).map(|mut e| e.despawn());
        });
        self
    }

    fn despawn_on_broadcast<T: Send + Sync + 'static>(&mut self) -> &mut Self
    {
        let entity = self.id();
        self.react().once(broadcast::<T>(), move |mut c: Commands| {
            let _ = c.get_entity(entity).map(|mut e| e.despawn());
        });
        self
    }

    fn reactor<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static,
    {
        // Do nothing if there are no triggers.
        if TypeId::of::<T>() == TypeId::of::<()>() {
            return self;
        }
        let id = self.id();
        let mut reactor = RawCallbackSystem::new(reactor);
        let callback = move |world: &mut World| {
            let result = reactor.run_with_cleanup(world, id, |_| {});
            result.handle(world);
        };
        let syscommand = self.commands().spawn_system_command(callback);
        register_reactor(&mut self.commands(), id, syscommand, triggers);

        self
    }

    fn update<M, R: CobwebResult, C: IntoSystem<TargetId, R, M> + Send + Sync + 'static>(
        &mut self,
        callback: C,
    ) -> &mut Self
    {
        self.update_on((), callback)
    }

    fn update_on<M, C, T, R>(&mut self, triggers: T, reactor: C) -> &mut Self
    where
        T: ReactionTriggerBundle,
        R: CobwebResult,
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static,
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
        self.update_on((), move |id: TargetId, mut c: Commands| {
            let Ok(ec) = c.get_entity(*id) else { return };
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

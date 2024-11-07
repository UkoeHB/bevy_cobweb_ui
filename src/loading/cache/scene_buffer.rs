use std::any::TypeId;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
#[cfg(feature = "hot_reload")]
use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

use bevy::ecs::world::Command;
use bevy::prelude::*;
#[cfg(feature = "hot_reload")]
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

// This function assumes loadables are unique within a scene node.
fn insert_node_loadable_entry(
    loadables: &mut HashMap<SceneRef, SmallVec<[ErasedLoadable; 4]>>,
    loadable_ref: &SceneRef,
    index: usize,
    loadable: ReflectedLoadable,
    type_id: TypeId,
    full_type_name: &str,
) -> InsertNodeResult
{
    match loadables.entry(loadable_ref.clone()) {
        Vacant(entry) => {
            if index != 0 {
                tracing::error!("failed inserting node loadable {:?} at {:?}; expected to insert at index {} but \
                    the current loadables length is 0", full_type_name, loadable_ref, index);
                return InsertNodeResult::NoChange;
            }
            let mut vec = SmallVec::default();
            vec.push(ErasedLoadable { type_id, loadable });
            entry.insert(vec);

            InsertNodeResult::Added
        }
        Occupied(mut entry) => {
            // Insert if the loadable value changed.
            if let Some(pos) = entry.get().iter().position(|e| e.type_id == type_id) {
                // Check if the value is changing.
                let erased_loadable = &mut entry.get_mut()[pos];
                match erased_loadable.loadable.equals(&loadable) {
                    Some(true) => {
                        if pos == index {
                            InsertNodeResult::NoChange
                        } else {
                            if pos < index {
                                tracing::error!("error updating loadable {:?} at {:?}, detected previous instance of loadable \
                                    at index {} which is lower than the target index {} indicating there's a duplicate in the \
                                    scene node list (this is a bug)", full_type_name, loadable_ref, pos, index);
                            }
                            entry.get_mut().swap(pos, index);
                            InsertNodeResult::Rearranged
                        }
                    }
                    Some(false) => {
                        // Replace the existing value.
                        if pos < index {
                            tracing::error!("error updating loadable {:?} at {:?}, detected previous instance of loadable \
                                at index {} which is lower than the target index {} indicating there's a duplicate in the \
                                scene node list (this is a bug)", full_type_name, loadable_ref, pos, index);
                        }
                        *erased_loadable = ErasedLoadable { type_id, loadable };
                        entry.get_mut().swap(pos, index);
                        InsertNodeResult::Changed
                    }
                    None => {
                        tracing::error!("failed updating loadable {:?} at {:?}, its reflected value doesn't implement \
                            PartialEq", full_type_name, loadable_ref);
                        InsertNodeResult::NoChange
                    }
                }
            } else if index <= entry.get().len() {
                entry
                    .get_mut()
                    .insert(index, ErasedLoadable { type_id, loadable });
                InsertNodeResult::Added
            } else {
                tracing::error!("failed inserting node loadable {:?} at {:?}; expected to insert at index {} but \
                    the current loadables' length is {}", full_type_name, loadable_ref, index, entry.get().len());
                InsertNodeResult::NoChange
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(PartialEq)]
enum InsertNodeResult
{
    Changed,
    Rearranged,
    Added,
    NoChange,
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
struct RevertCommand
{
    entity: Entity,
    reverter: fn(Entity, &mut World),
}

#[cfg(feature = "hot_reload")]
impl Command for RevertCommand
{
    fn apply(self, world: &mut World)
    {
        (self.reverter)(self.entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct NodeLoadCommand
{
    callback: fn(&mut World, Entity, ReflectedLoadable, SceneRef),
    entity: Entity,
    loadable_ref: SceneRef,
    loadable: ReflectedLoadable,
}

impl Command for NodeLoadCommand
{
    fn apply(self, world: &mut World)
    {
        (self.callback)(world, self.entity, self.loadable, self.loadable_ref);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
#[derive(Debug, Default)]
struct RefreshCtx
{
    /// type ids of loadables that need to be reverted on specific entities.
    needs_revert: Vec<(Entity, HashSet<TypeId>)>,
    /// Records entities that need loadable updates.
    needs_updates: Vec<(Entity, NodeInitializer, SceneRef)>,
}

#[cfg(feature = "hot_reload")]
impl RefreshCtx
{
    fn add_revert(&mut self, subscription: SubscriptionRef, type_id: TypeId)
    {
        match self
            .needs_revert
            .iter()
            .position(|(e, _)| *e == subscription.entity)
        {
            Some(pos) => {
                self.needs_revert[pos].1.insert(type_id);
            }
            None => self
                .needs_revert
                .push((subscription.entity, HashSet::from_iter([type_id]))),
        }
    }
    fn add_update(&mut self, subscription: SubscriptionRef, loadable_ref: SceneRef)
    {
        if self
            .needs_updates
            .iter()
            .any(|(e, _, _)| *e == subscription.entity)
        {
            return;
        };
        self.needs_updates
            .push((subscription.entity, subscription.initializer, loadable_ref.clone()));
    }

    fn reverts(&mut self) -> impl Iterator<Item = (Entity, HashSet<TypeId>)> + '_
    {
        self.needs_revert.drain(..)
    }
    fn updates(&mut self) -> impl Iterator<Item = (Entity, NodeInitializer, SceneRef)> + '_
    {
        self.needs_updates.drain(..)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
struct SubscriptionRef
{
    entity: Entity,
    initializer: NodeInitializer,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct SceneBuffer
{
    /// Tracks manifest data.
    manifest_map: Arc<Mutex<ManifestMap>>,

    /// Tracks loadables from all loaded files.
    /// - Note: If a scene node is hot-removed, then this map will *not* be updated. However, the scene's loader
    /// will correctly update, so new scene spawns won't include dead nodes (and existing scenes will be
    /// repaired).
    loadables: HashMap<SceneRef, SmallVec<[ErasedLoadable; 4]>>,

    /// Tracks subscriptions to scene paths.
    #[cfg(feature = "hot_reload")]
    subscriptions: HashMap<SceneRef, SmallVec<[SubscriptionRef; 1]>>,
    /// Tracks entities for cleanup and enables manual reloads.
    #[cfg(feature = "hot_reload")]
    subscriptions_rev: HashMap<Entity, (SceneRef, NodeInitializer)>,

    /// Records loadables that need to be reverted/updated.
    #[cfg(feature = "hot_reload")]
    refresh_ctx: RefreshCtx,
}

impl SceneBuffer
{
    pub(super) fn new(manifest_map: Arc<Mutex<ManifestMap>>) -> Self
    {
        Self { manifest_map, ..default() }
    }

    fn manifest_map(&mut self) -> MutexGuard<ManifestMap>
    {
        self.manifest_map.lock().unwrap()
    }

    /// Prepares a scene node.
    ///
    /// We need to prepare scene nodes because they may be empty.
    pub(crate) fn prepare_scene_node(&mut self, loadable_ref: SceneRef)
    {
        self.loadables.entry(loadable_ref).or_default();
    }

    /// Inserts a loadable at the specified path and index if its value will change.
    pub(crate) fn insert_loadable(
        &mut self,
        loadable_ref: &SceneRef,
        index: usize,
        loadable: ReflectedLoadable,
        type_id: TypeId,
        full_type_name: &str,
    )
    {
        let res = insert_node_loadable_entry(
            &mut self.loadables,
            loadable_ref,
            index,
            loadable.clone(),
            type_id,
            full_type_name,
        );
        if res == InsertNodeResult::NoChange {
            return;
        }

        // Identify entites that should update.
        #[cfg(feature = "hot_reload")]
        {
            let Some(subscriptions) = self.subscriptions.get(loadable_ref) else { return };
            if subscriptions.is_empty() {
                return;
            }

            for subscription in subscriptions {
                if res == InsertNodeResult::Changed {
                    self.refresh_ctx.add_revert(*subscription, type_id);
                }
                self.refresh_ctx
                    .add_update(*subscription, loadable_ref.clone());
            }
        }
    }

    /// Cleans up any removed loadables if the loadable set became smaller after a hot reload.
    ///
    /// Runs after all loadables in a scene node have been inserted.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn end_loadable_insertion(&mut self, loadable_ref: &SceneRef, count: usize)
    {
        let Some(subscriptions) = self.subscriptions.get(loadable_ref) else { return };
        if subscriptions.is_empty() {
            return;
        }

        // Revert trailing removals
        for removed in self
            .loadables
            .get_mut(loadable_ref)
            .into_iter()
            .flat_map(|l| l.drain(count..))
        {
            for subscription in subscriptions {
                self.refresh_ctx.add_revert(*subscription, removed.type_id);
                self.refresh_ctx
                    .add_update(*subscription, loadable_ref.clone());
            }
        }
    }

    fn load_entity(
        &self,
        subscription: SubscriptionRef,
        loadable_ref: SceneRef,
        callbacks: &LoaderCallbacks,
        c: &mut Commands,
    )
    {
        // Initialize
        let Some(mut ec) = c.get_entity(subscription.entity) else { return };
        (subscription.initializer.initializer)(&mut ec);

        // Queue loadables
        let Some(loadables) = self.loadables.get(&loadable_ref) else {
            tracing::warn!("failed loading {loadable_ref:?} into {:?}, path is unknown; either the path is \
                invalid or you loaded the entity before LoadState::Done", subscription.entity);
            return;
        };

        for loadable in loadables.iter() {
            let Some(callback) = callbacks.get_for_node(loadable.type_id) else {
                tracing::warn!("found loadable at {:?} that wasn't registered with CobwebAssetRegistrationAppExt",
                    loadable_ref);
                continue;
            };

            c.queue(NodeLoadCommand {
                callback,
                entity: subscription.entity,
                loadable_ref: loadable_ref.clone(),
                loadable: loadable.loadable.clone(),
            });
        }

        // Notify the entity that it loaded.
        #[cfg(feature = "hot_reload")]
        {
            if !loadables.is_empty() {
                c.react().entity_event(subscription.entity, Loaded);
            }
        }
    }

    /// Adds an entity to the tracking context.
    ///
    /// Schedules callbacks that will run to handle pending updates for the entity.
    pub(crate) fn track_entity(
        &mut self,
        entity: Entity,
        mut loadable_ref: SceneRef,
        initializer: NodeInitializer,
        callbacks: &LoaderCallbacks,
        c: &mut Commands,
    )
    {
        // Replace manifest key in the requested loadable.
        self.manifest_map().swap_for_file(&mut loadable_ref.file);

        // Add to subscriptions.
        let subscription = SubscriptionRef { entity, initializer };
        #[cfg(feature = "hot_reload")]
        {
            self.subscriptions
                .entry(loadable_ref.clone())
                .or_default()
                .push(subscription);
            if let Some((prev_loadable_ref, _)) = self.subscriptions_rev.get(&entity) {
                // Prints if multiple scene nodes are loaded to the same entity.
                tracing::warn!("overwriting scene node tracking for entity {:?}; prev: {:?}, new {:?}",
                    entity, prev_loadable_ref, loadable_ref);
            }
            self.subscriptions_rev
                .insert(entity, (loadable_ref.clone(), initializer));
        }

        // Load the entity immediately.
        self.load_entity(subscription, loadable_ref, callbacks, c);
    }

    /// Adds an entity to the tracking context.
    ///
    /// Queues the entity to be loaded. This allows synchronizing a new entity (e.g. a new scene entity) with
    /// other refresh-edits to ancestors in the scene hierarchy (those edits are also queeud - if we did
    /// .load_entity() immediately then it would happen *before* ancestors are updated).
    #[cfg(feature = "hot_reload")]
    pub(crate) fn track_entity_queued(
        &mut self,
        entity: Entity,
        mut loadable_ref: SceneRef,
        initializer: NodeInitializer,
    )
    {
        // Replace manifest key in the requested loadable.
        self.manifest_map().swap_for_file(&mut loadable_ref.file);

        // Add to subscriptions.
        let subscription = SubscriptionRef { entity, initializer };
        self.subscriptions
            .entry(loadable_ref.clone())
            .or_default()
            .push(subscription);
        if let Some((prev_loadable_ref, _)) = self.subscriptions_rev.get(&entity) {
            // Prints if multiple scene nodes are loaded to the same entity.
            tracing::warn!("overwriting scene node tracking for entity {:?}; prev: {:?}, new {:?}",
                entity, prev_loadable_ref, loadable_ref);
        }
        self.subscriptions_rev
            .insert(entity, (loadable_ref.clone(), initializer));

        // Queue the entity to be loaded.
        self.refresh_ctx
            .add_update(subscription, loadable_ref.clone());
    }

    /// Requests that the scene node an entity is subscribed to be reloaded on that entity.
    #[cfg(feature = "hot_reload")]
    pub fn request_reload(&mut self, entity: Entity)
    {
        let Some((loadable_ref, initializer)) = self.subscriptions_rev.get(&entity) else {
            tracing::warn!("requested reload of entity {entity:?} that is not subscribed to any loadables");
            return;
        };
        self.refresh_ctx.add_update(
            SubscriptionRef { entity, initializer: *initializer },
            loadable_ref.clone(),
        );
    }

    #[cfg(feature = "hot_reload")]
    pub(super) fn apply_pending_node_updates(&mut self, c: &mut Commands, callbacks: &LoaderCallbacks)
    {
        // Revert loadables as needed.
        // - Note: we currently assume the order of reverts doesn't matter.
        for (entity, type_ids) in self.refresh_ctx.reverts() {
            for type_id in type_ids {
                let Some(reverter) = callbacks.get_for_revert(type_id) else { continue };
                c.queue(RevertCommand { entity, reverter });
            }
        }

        // Reload entities.
        let needs_updates = self.refresh_ctx.updates().collect::<Vec<_>>();
        for (entity, initializer, loadable_ref) in needs_updates {
            self.load_entity(SubscriptionRef { entity, initializer }, loadable_ref, callbacks, c);
        }
    }

    /// Cleans up despawned entities.
    #[cfg(feature = "hot_reload")]
    pub(super) fn remove_entity(&mut self, scene_loader: &mut SceneLoader, dead_entity: Entity)
    {
        let Some((loadable_ref, _)) = self.subscriptions_rev.remove(&dead_entity) else { return };

        // Clean up scenes.
        scene_loader.cleanup_dead_entity(&loadable_ref, dead_entity);

        // Clean up subscription.
        let Some(subscribed) = self.subscriptions.get_mut(&loadable_ref) else { return };
        let Some(dead) = subscribed.iter().position(|s| s.entity == dead_entity) else { return };
        subscribed.swap_remove(dead);
    }
}

//-------------------------------------------------------------------------------------------------------------------

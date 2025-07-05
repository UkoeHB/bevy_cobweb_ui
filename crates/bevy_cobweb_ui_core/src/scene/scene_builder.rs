use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy::prelude::*;
#[cfg(feature = "hot_reload")]
use bevy_cobweb::prelude::*;
#[cfg(feature = "hot_reload")]
use smallvec::SmallVec;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
fn find_scene_child_pos(world: &World, parent_entity: Entity, new_index: usize) -> usize
{
    let mut count = 0;
    let children = world.get::<Children>(parent_entity);
    let num_children = children.map(|c| c.len()).unwrap_or_default();
    let calculated_position = world
        .get::<Children>(parent_entity)
        .and_then(|c| {
            c.iter().position(|entity| {
                // Skip children without loadables.
                if !world.get::<HasLoadables>(entity).is_some() {
                    return false;
                }

                if count < new_index {
                    count += 1;
                    return false;
                }

                true
            })
        })
        .unwrap_or(num_children);

    std::cmp::min(calculated_position, new_index)
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) enum SceneLayerInsertionResult<'a>
{
    #[cfg(feature = "hot_reload")]
    NoChange(&'a mut SceneLayer),
    #[cfg(feature = "hot_reload")]
    Updated(usize, &'a mut SceneLayer),
    Added(usize, &'a mut SceneLayer),
}

impl<'a> SceneLayerInsertionResult<'a>
{
    fn new(method: SceneLayerInsertionMethod, position: usize, layer: &'a mut SceneLayer) -> Self
    {
        // Note: this roundabout way to create results is due to lifetime issues with other approaches.
        match method {
            #[cfg(feature = "hot_reload")]
            SceneLayerInsertionMethod::NoChange => Self::NoChange(layer),
            #[cfg(feature = "hot_reload")]
            SceneLayerInsertionMethod::Updated => Self::Updated(position, layer),
            SceneLayerInsertionMethod::Added => Self::Added(position, layer),
        }
    }
}

enum SceneLayerInsertionMethod
{
    #[cfg(feature = "hot_reload")]
    NoChange,
    #[cfg(feature = "hot_reload")]
    Updated,
    Added,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct SceneLayerData
{
    pub(crate) id: ScenePath,
    pub(crate) layer: SceneLayer,
}

impl SceneLayerData
{
    /// Gets the total number of child nodes.
    pub(crate) fn total_child_nodes(&self) -> usize
    {
        self.layer.total_child_nodes()
    }

    /// Inspects this layer's id, then traverses its child layer.
    pub(crate) fn traverse(&self, inspector: &mut impl FnMut(&ScenePath))
    {
        (inspector)(&self.id);
        self.layer.traverse(inspector);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub(crate) struct SceneLayer
{
    children: Vec<SceneLayerData>,
    end_index: usize,
    total_child_nodes: usize,
}

impl SceneLayer
{
    /// Begins an update of the layer's children.
    pub(crate) fn start_update(&mut self, layer_size: usize)
    {
        self.children
            .reserve(layer_size.saturating_sub(self.children.len()));
        self.end_index = 0;
        self.total_child_nodes = 0;
    }

    /// Inserts a node with `id` at the current update position.
    pub(crate) fn insert(&mut self, id: &ScenePath) -> SceneLayerInsertionResult
    {
        let position = self.end_index;
        self.end_index += 1;

        #[cfg(not(feature = "hot_reload"))]
        {
            debug_assert_eq!(self.children.len(), position);
        }

        #[allow(unused_labels)]
        let insertion_method = 'm: {
            #[cfg(feature = "hot_reload")]
            {
                // Check if this node already exists.
                // Note: we assume children before `position` can never equal `id`.
                if let Some(offset_pos) = self.children[position..]
                    .iter()
                    .position(|data| data.id == *id)
                {
                    // Case: the requested id is at the current update position.
                    if offset_pos == 0 {
                        break 'm SceneLayerInsertionMethod::NoChange;
                    }
                    // Case: move the requested id to the update position.
                    self.children.swap(position, position + offset_pos);
                    break 'm SceneLayerInsertionMethod::Updated;
                }
            }

            // Insert a new node at the update position.
            self.children.insert(
                position,
                SceneLayerData { id: id.clone(), layer: SceneLayer::default() },
            );
            SceneLayerInsertionMethod::Added
        };

        SceneLayerInsertionResult::new(insertion_method, position, &mut self.children[position].layer)
    }

    /// Ends an update process.
    ///
    /// Returns nodes that were removed.
    pub(crate) fn end_update(&mut self) -> impl Iterator<Item = SceneLayerData> + '_
    {
        let end = self.end_index;
        self.end_index = 0;
        self.total_child_nodes = end;
        self.total_child_nodes += self.children[..end]
            .iter()
            .map(SceneLayerData::total_child_nodes)
            .reduce(|a, b| a + b)
            .unwrap_or_default();
        self.children.drain(end..)
    }

    /// Gets the total number of child nodes.
    pub(crate) fn total_child_nodes(&self) -> usize
    {
        self.total_child_nodes
    }

    /// Iterates over the node's children in order, applying the inspector function to each one.
    pub(crate) fn traverse(&self, inspector: &mut impl FnMut(&ScenePath))
    {
        for child in self.children.iter() {
            child.traverse(inspector);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(crate) struct SceneRegistry
{
    /// [ root node reference : root node's child layer ]
    scenes: HashMap<SceneRef, SceneLayer>,
}

impl SceneRegistry
{
    /// Accesses the child layer of a scene's root node to edit it.
    pub(crate) fn get_or_insert(&mut self, scene_ref: SceneRef) -> &mut SceneLayer
    {
        self.scenes.entry(scene_ref).or_default()
    }

    /// Accesses the child layer of a scene's root node.
    pub(crate) fn get(&self, scene_ref: &SceneRef) -> Option<&SceneLayer>
    {
        self.scenes.get(scene_ref)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub(crate) struct SceneInstance
{
    /// Reference to the scene root.
    scene_ref: SceneRef,
    /// Root entity.
    entity: Entity,
    /// Prep function for new nodes.
    new_node_prep_fn: NodeInitializer,
    // [ scene node path : scene node entity ]
    nodes: HashMap<ScenePath, Entity>,
}

impl SceneInstance
{
    fn new_for_ref(scene_ref: SceneRef) -> Self
    {
        Self {
            scene_ref,
            entity: Entity::PLACEHOLDER,
            new_node_prep_fn: NodeInitializer { initializer: |_| {} },
            nodes: HashMap::default(),
        }
    }

    /// Prepares the instance to be filled by a scene.
    pub(crate) fn prepare(
        &mut self,
        scene_ref: SceneRef,
        entity: Entity,
        new_node_prep_fn: fn(&mut EntityCommands),
        node_count: usize,
    )
    {
        self.scene_ref = scene_ref;
        self.entity = entity;
        self.new_node_prep_fn = NodeInitializer { initializer: new_node_prep_fn };
        self.nodes.clear();
        self.nodes.reserve(node_count);
    }

    /// Gets the current capacity of the inner map.
    pub(crate) fn capacity(&self) -> usize
    {
        self.nodes.capacity()
    }

    /// Inserts a scene node.
    pub(crate) fn insert(&mut self, path: ScenePath, entity: Entity)
    {
        self.nodes.insert(path, entity);
    }

    /// Removes a scene node with the given path.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn remove(&mut self, path: &ScenePath) -> Option<Entity>
    {
        self.nodes.remove(path)
    }

    /// Returns the file location of this scene.
    pub(crate) fn scene_ref(&self) -> &SceneRef
    {
        &self.scene_ref
    }

    /// Returns the root entity of this instance.
    pub(crate) fn root_entity(&self) -> Entity
    {
        self.entity
    }

    /// Returns the function that should be used to initialize new nodes.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn node_prep_fn(&self) -> NodeInitializer
    {
        self.new_node_prep_fn
    }

    /// Gets a scene node at the given path.
    pub(crate) fn get(&self, path: &ScenePath) -> Option<Entity>
    {
        self.nodes.get(path).cloned().or_else(|| {
            if self.scene_ref.path == *path {
                Some(self.entity)
            } else {
                None
            }
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Manages loaded scene definitions and used to spawn scene instances.
///
/// Use the [`SceneBuilder`] system parameter instead of this resource.
#[derive(Resource, Default)]
pub struct SceneBuilderInner
{
    /// Tracks manifest data.
    /// - Inside an arc/mutex so the CobAssetCache can also use it.
    manifest_map: Arc<Mutex<ManifestMap>>,

    /// Tracks the currently active scenes.
    active_scene_stack: Vec<SceneInstance>,
    /// Cached scene instances for memory reuse.
    scene_instance_cache: Vec<SceneInstance>,
    /// Records the structure of all known scenes. Used to spawn new scenes and aid hot reloading of scenes.
    /// - The registry is `Option` so it can be removed for mutation and traversal while parsing a file.
    scene_registry: Option<SceneRegistry>,
    /// Entity vector cached for reuse when constructing scene instances.
    scene_parent_stack_cached: Vec<Entity>,
    /// Tracks scene instances that exist in the world (hierarchies of entities).
    ///
    /// Used to update scene structures (add/remove/rearrange entities) in response to hot reloaded changes.
    #[cfg(feature = "hot_reload")]
    scene_instances: HashMap<SceneRef, SmallVec<[SceneInstance; 1]>>,
}

impl SceneBuilderInner
{
    /// Makes a new scene loader from a shared manifest map.
    pub(crate) fn new(manifest_map: Arc<Mutex<ManifestMap>>) -> Self
    {
        Self { manifest_map, ..default() }
    }

    fn manifest_map(&self) -> MutexGuard<ManifestMap>
    {
        self.manifest_map.lock().unwrap()
    }

    /// Extracts the scene registry so it can be updated.
    pub(crate) fn take_scene_registry(&mut self) -> SceneRegistry
    {
        self.scene_registry.take().unwrap_or_default()
    }

    /// Returns the scene registry after it has been updated.
    pub(crate) fn return_scene_registry(&mut self, registry: SceneRegistry)
    {
        if self.scene_registry.is_some() {
            tracing::error!("returning scene registry after file load but there is already a scene registry (this \
                is a bug)");
        }
        self.scene_registry = Some(registry);
    }

    /// Spawns new scene nodes in existing scenes instances.
    ///
    /// Used to fill in missing slots in hierarchies after a scene structure change is hot reloaded.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn handle_inserted_scene_node(
        &mut self,
        c: &mut Commands,
        scene: &SceneRef,
        parent: &ScenePath,
        inserted: &ScenePath,
        insertion_index: usize,
    )
    {
        // Look up scene.
        let Some(scene_instances) = self.scene_instances.get_mut(scene) else { return };

        // Update each instance.
        for scene_instance in scene_instances.iter_mut() {
            // Get parent entity.
            let parent_entity = {
                if parent.len() == 1 {
                    scene_instance.root_entity()
                } else {
                    let Some(parent_entity) = scene_instance.get(parent) else {
                        tracing::error!("failed updating scene instance of {:?} for {:?} with hot-inserted node {:?}, node's \
                            parent {:?} is missing (this is a bug)", scene, scene_instance.root_entity(), inserted, parent);
                        continue;
                    };
                    parent_entity
                }
            };

            // Spawn entity.
            let node_entity = c.spawn_empty().id();

            // Insert the entity to the proper parent index.
            // - We add an extra step of truncating the insertion index to avoid panics on insert.
            let scene_inner = scene.clone();
            let root_entity = scene_instance.root_entity();
            let inserted_inner = inserted.clone();
            c.queue(move |world: &mut World| {
                let position = find_scene_child_pos(world, parent_entity, insertion_index);

                let Ok(mut emut) = world.get_entity_mut(parent_entity) else {
                    tracing::warn!("failed updating scene instance of {:?} for {:?} with hot-inserted node {:?}, node's \
                        parent {:?} was despawned", scene_inner, root_entity, inserted_inner, parent_entity);
                    return;
                };
                emut.insert_children(position, &[node_entity]);
            });

            // Load the scene node to the entity.
            // - We load this 'queued' so side effects will be synchronized with other edits to the scene
            //   hierarchy.
            let node_ref = SceneRef { file: scene.file.clone(), path: inserted.clone() };
            c.syscall(
                (node_entity, node_ref, scene_instance.node_prep_fn()),
                load_queued_from_ref,
            );

            // Save the entity.
            scene_instance.insert(inserted.clone(), node_entity);
        }
    }

    /// Modifies the [`Children`] ordering of scene nodes in existing scene instances.
    ///
    /// Used to adjust scene node positions in hierarchies after a scene structure change is hot reloaded.
    ///
    /// Panics if scene nodes were despawned manually, causing the new insertion index to be invalid.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn handle_rearranged_scene_node(
        &self,
        c: &mut Commands,
        scene: &SceneRef,
        parent: &ScenePath,
        moved: &ScenePath,
        new_index: usize,
    )
    {
        // Look up scene.
        let Some(scene_instances) = self.scene_instances.get(scene) else { return };

        // Update each instance.
        for scene_instance in scene_instances.iter() {
            // Get parent entity.
            let Some(parent_entity) = scene_instance.get(parent) else {
                tracing::error!("failed updating scene instance of {:?} for {:?} with hot-rearranged node {:?}, node's \
                    parent {:?} is missing (this is a bug)", scene, scene_instance.root_entity(), moved, parent);
                continue;
            };

            // Get the target entity.
            let Some(node_entity) = scene_instance.get(moved) else {
                tracing::error!("failed updating scene instance of {:?} for {:?} with hot-rearranged node {:?}, node
                    is missing (this is a bug)", scene, scene_instance.root_entity(), moved);
                continue;
            };

            // Insert at the desired index.
            // - We add an extra step of truncating the insertion index to avoid panics on insert.
            // - This correctly rearranges entities that are already in the parent's child list.
            let scene = scene.clone();
            let root_entity = scene_instance.root_entity();
            let moved = moved.clone();
            c.queue(move |world: &mut World| {
                let position = find_scene_child_pos(world, parent_entity, new_index);

                let Ok(mut emut) = world.get_entity_mut(parent_entity) else {
                    tracing::warn!("failed updating scene instance of {:?} for {:?} with hot-rearranged node {:?}, node's \
                        parent {:?} was despawned", scene, root_entity, moved, parent_entity);
                    return;
                };
                emut.insert_children(position, &[node_entity]);
            });
        }
    }

    /// Despawns scene branches from existing scene instances.
    ///
    /// Used to repair hierarchies after a scene structure change is hot reloaded.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn cleanup_deleted_scene_node(
        &mut self,
        c: &mut Commands,
        scene_buffer: &mut SceneBuffer,
        loadables: &LoadableRegistry,
        scene: &SceneRef,
        deleted: &ScenePath,
    )
    {
        // Revert loadables on the removed node.
        // TODO: revisit if bevy adds command batching that moves despawns before normal commands.
        scene_buffer.remove_scene_node(
            c,
            loadables,
            SceneRef { file: scene.file.clone(), path: deleted.clone() },
        );

        // Look up scene.
        let Some(scene_instances) = self.scene_instances.get_mut(scene) else { return };

        // Update each instance.
        for scene_instance in scene_instances.iter_mut() {
            // Remove the node and get the target entity.
            let Some(node_entity) = scene_instance.remove(deleted) else {
                tracing::error!("failed updating scene instance of {:?} for {:?} with hot-removed node {:?}, node \
                    is missing (this is a bug)", scene, scene_instance.root_entity(), deleted);
                continue;
            };

            // Recursively despawn the node.
            let Ok(mut ec) = c.get_entity(node_entity) else {
                tracing::warn!("failed updating scene instance of {:?} for {:?} with hot-removed node {:?}, node \
                    {:?} was already despawned", scene, scene_instance.root_entity(), deleted, node_entity);
                continue;
            };

            ec.despawn();
        }
    }

    /// Despawns existing scene instances.
    ///
    /// Used to clean up hierarchies after a scene deletion is hot reloaded.
    //todo: how to detect these? maybe iterate base layer to check if existing scene instances don't match?
    #[cfg(feature = "hot_reload")]
    pub(crate) fn _cleanup_deleted_scene(&mut self, c: &mut Commands, scene: &SceneRef)
    {
        // Get scene.
        let Some(mut scene_instances) = self.scene_instances.remove(scene) else { return };

        // Remove and despawn all instances.
        for dead_instance in scene_instances.drain(..) {
            let Ok(mut ec) = c.get_entity(dead_instance.root_entity()) else { continue };
            ec.despawn();
        }
    }

    /// Cleans up despawned root entities.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn cleanup_dead_entity(&mut self, scene_ref: &SceneRef, dead_entity: Entity)
    {
        let Some(scene_instances) = self.scene_instances.get_mut(&scene_ref) else { return };
        let Some(dead_idx) = scene_instances
            .iter()
            .position(|i| i.root_entity() == dead_entity)
        else {
            return;
        };
        let dead = scene_instances.swap_remove(dead_idx);
        self.scene_instance_cache.push(dead);
    }

    /// Builds a scene into a target entity, which will be the root of the scene.
    ///
    /// The scene hierarchy is saved temporarily in a `SceneInstance`. It will be discarded when
    /// [`Self::release_active_scene`] is called unless the `hot_reload` feature is active.
    pub(crate) fn build_scene<T>(&mut self, c: &mut Commands, root_entity: Entity, mut scene_ref: SceneRef) -> bool
    where
        T: crate::scene::spawn_scene_ext::scene_traits::SceneNodeBuilder,
    {
        // Reject non-root nodes.
        if scene_ref.path.len() != 1 {
            tracing::warn!("failed loading scene {:?} into {:?}, the requested location has a scene path length of {} \
                but only root scene nodes (path length 1) can be used to load a scene",
                scene_ref, root_entity, scene_ref.path.len());
            return false;
        }

        // Replace manifest key in the requested scene.
        self.manifest_map().swap_for_file(&mut scene_ref.file);

        // Look up the requested scene.
        let Some(scene_registry) = &self.scene_registry else {
            tracing::error!("scene load of {:?} into {:?} failed, scene registry is missing; it's likely the scene's \
                file has not loaded yet; wait to load scenes until in LoadState::Done", scene_ref, root_entity);
            return false;
        };
        let Some(root_scene_layer) = scene_registry.get(&scene_ref) else {
            tracing::error!("failed loading scene {:?} into {:?}, there is no scene at that location OR the \
                scene's file has not loaded; wait to load scenes until in LoadState::Done", scene_ref, root_entity);
            return false;
        };

        // Prepare scene instance.
        let mut scene_instance = {
            // Note: this reduces total memory use if there are very large scenes that get repeatedly constructed.
            let mut largest_capacity = 0;
            let mut largest_idx = 0;
            if let Some(cache_index) = self
                .scene_instance_cache
                .iter()
                .enumerate()
                .position(|(idx, i)| {
                    if largest_capacity < i.capacity() {
                        largest_capacity = i.capacity();
                        largest_idx = idx;
                    }
                    largest_capacity >= root_scene_layer.total_child_nodes()
                })
            {
                self.scene_instance_cache.swap_remove(cache_index)
            } else if self.scene_instance_cache.len() > 0 {
                self.scene_instance_cache.swap_remove(largest_idx)
            } else {
                SceneInstance::new_for_ref(scene_ref.clone())
            }
        };
        scene_instance.prepare(
            scene_ref.clone(),
            root_entity,
            T::initialize_scene_node,
            root_scene_layer.total_child_nodes(),
        );

        // Load the root entity.
        let mut root_ec = c.entity(root_entity);
        root_ec.build_with_initializer(scene_ref.clone(), T::initialize_scene_node);

        // Spawn hierarchy, loading all child paths.
        // - Hierarchy spawn order matches the order in cob files.
        // - NOTE: We do not use ChildBuilder here, even though it would be more efficient, because node parents
        //   must be set before we call `.build()` on them. ChildBuilder defers parent assignment.
        let parent_stack = &mut self.scene_parent_stack_cached;
        parent_stack.clear();
        let mut prev_entity = root_entity;
        let mut prev_path_length = 1;

        root_scene_layer.traverse(&mut |scene_node_path| {
            debug_assert!(scene_node_path.len() > 1);
            let path_change = (scene_node_path.len() as i32) - (prev_path_length as i32);

            // Update the parent stack.
            // Case: increasing the path means the previous node is a parent.
            if path_change > 0 {
                debug_assert_eq!(path_change, 1);
                parent_stack.push(prev_entity);
            }
            // Case: same path length means add a child node to the current parent.
            else if path_change == 0 {
                // Nothing to do.
            }
            // Case: reduced path length means the current parent is done adding children.
            else {
                parent_stack.truncate(parent_stack.len() - (path_change.unsigned_abs() as usize));
            }

            // Spawn entity.
            let mut ec = c.spawn_empty();
            ec.insert(ChildOf(*parent_stack.last().unwrap()));

            // Load the scene node to the entity.
            let node_ref = SceneRef { file: scene_ref.file.clone(), path: scene_node_path.clone() };
            ec.build_with_initializer(node_ref.clone(), T::initialize_scene_node);

            // Save the entity.
            let node_entity = ec.id();
            scene_instance.insert(node_ref.path, node_entity);

            prev_entity = node_entity;
            prev_path_length = scene_node_path.len();
        });

        // Save the scene stack for use when editing the scene contents.
        self.active_scene_stack.push(scene_instance);
        true
    }

    /// Gets the current active scene (the topmost entry in the active scene stack).
    pub(crate) fn active_scene(&self) -> Option<&SceneInstance>
    {
        self.active_scene_stack.last()
    }

    /// Pops an entry from the active `SceneInstance` stack.
    ///
    /// When `hot_reload` is not enabled, the scene hierarchy cache will be discarded here. We assume the scene
    /// hierarchy only needs to be accessed during construction and for hot reloading nodes.
    pub(crate) fn release_active_scene(&mut self)
    {
        // Remove scene stack.
        let Some(released) = self.active_scene_stack.pop() else {
            tracing::error!("failed releasing active scene, no scene is active (this is a bug)");
            return;
        };

        // On hot reload, save so scene entities can be adjusted when the scene file changes.
        #[cfg(feature = "hot_reload")]
        {
            self.scene_instances
                .entry(released.scene_ref().clone())
                .or_default()
                .push(released);
        }

        // Otherwise, save the scene instance data for reuse.
        #[cfg(not(feature = "hot_reload"))]
        {
            self.scene_instance_cache.push(released);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System parameter that is used to spawn scene instances.
///
/// See [`SpawnSceneExt`].
///
/// Derefs to [`SceneBuilderInner`].
#[derive(SystemParam)]
pub struct SceneBuilder<'w>
{
    inner: ResMut<'w, SceneBuilderInner>,
}

impl Deref for SceneBuilder<'_>
{
    type Target = SceneBuilderInner;

    fn deref(&self) -> &Self::Target
    {
        &self.inner
    }
}

impl DerefMut for SceneBuilder<'_>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.inner
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that enables scene loading.
pub(crate) struct SceneBuilderPlugin;

impl Plugin for SceneBuilderPlugin
{
    fn build(&self, app: &mut App)
    {
        let manifest_map = app.world().resource::<CobAssetCache>().manifest_map_clone();
        app.insert_resource(SceneBuilderInner::new(manifest_map));
    }
}

//-------------------------------------------------------------------------------------------------------------------

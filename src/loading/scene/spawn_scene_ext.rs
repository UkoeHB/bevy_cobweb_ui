pub use std::ops::{Deref, DerefMut}; // Re-export for ease of use.

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_from_ref(
    In((id, scene_ref, initializer)): In<(Entity, SceneRef, NodeInitializer)>,
    mut c: Commands,
    loadables: Res<LoadableRegistry>,
    mut scene_buffer: ResMut<SceneBuffer>,
    load_state: Res<State<LoadState>>,
    #[cfg(feature = "hot_reload")] commands_buffer: Res<CommandsBuffer>,
)
{
    if *load_state.get() != LoadState::Done {
        tracing::error!("failed building scene node {scene_ref:?} into {id:?}, app state is not LoadState::Done");
        return;
    }

    scene_buffer.track_entity(
        id,
        scene_ref,
        initializer,
        &loadables,
        &mut c,
        #[cfg(feature = "hot_reload")]
        &commands_buffer,
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_scene_impl<'b, T, C, R>(
    builder: &'b mut T,
    path: impl Into<SceneRef>,
    scene_builder: &'b mut SceneBuilderInner,
    callback: C,
) -> &'b mut T
where
    T: scene_traits::SceneNodeBuilder,
    C: for<'a> FnOnce(&mut SceneHandle<'a, <T as scene_traits::SceneNodeBuilder>::Builder<'a>>) -> R,
    R: CobwebResult,
{
    let path = path.into();

    // Spawn either a child or a raw entity to be the scene's root node.
    let root_entity = builder
        .scene_parent_entity()
        .map(|parent| builder.commands().spawn_empty().set_parent(parent).id())
        .unwrap_or_else(|| builder.commands().spawn_empty().id());

    // Avoid panicking if the parent is invalid.
    if builder.commands().get_entity(root_entity).is_none() {
        tracing::warn!("failed loading scene at {:?}; parent {root_entity:?} does not exist", path);
        return builder;
    }

    // Load the scene into the root entity.
    let mut commands = builder.commands();
    if !scene_builder.build_scene::<T>(&mut commands, root_entity, path.clone()) {
        return builder;
    }

    // Allow editing the scene via callback.
    let result = {
        let mut root_node = SceneHandle {
            scene_builder,
            builder: T::scene_node_builder(&mut commands, root_entity),
            scene: path,
        };

        (callback)(&mut root_node)
    };
    if result.need_to_handle() {
        commands.queue(move |w: &mut World| {
            result.handle(w);
        });
    }

    // Cleanup
    scene_builder.release_active_scene();

    builder
}

//-------------------------------------------------------------------------------------------------------------------

// Put this trait in a separate module so it doesn't pollute the `prelude`.
pub mod scene_traits
{
    use bevy::ecs::system::EntityCommands;
    use bevy::prelude::*;

    #[allow(unused_imports)]
    use crate::prelude::*;

    /// Helper trait for spawning a scene. See [`SceneRef`] and [`SpawnSceneExt::spawn_scene`].
    pub trait SceneNodeBuilder
    {
        /// The type returned by [`Self::scene_node_builder`].
        type Builder<'a>: SceneNodeBuilderOuter<'a>;

        /// Gets a [`Commands`] instance.
        fn commands(&mut self) -> Commands;
        /// Gets the parent entity for scenes loaded with this `SceneBuilder`.
        fn scene_parent_entity(&self) -> Option<Entity>;
        /// Prepares a scene node to receive scene data.
        ///
        /// For example, UI nodes need `NodeBundles`, which this method can auto-insert.
        fn initialize_scene_node(ec: &mut EntityCommands);
        /// Gets a [`SceneNodeBuilderOuter`] instance in order to edit a node in the loaded scene.
        fn scene_node_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Builder<'a>;
        /// Gets a [`SceneNodeBuilderOuter`] instance in order to edit a node in the loaded scene.
        fn new_with(&mut self, entity: Entity) -> Self::Builder<'_>;
    }

    /// Helper trait for editing nodes in a loaded scene. See [`SceneRef`] and
    /// [`SpawnSceneExt::spawn_scene`].
    pub trait SceneNodeBuilderOuter<'a>: SceneNodeBuilder {}
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper struct for editing spawned scenes.
///
/// The struct will dereference to the inner `T`, which should be a [`Commands`]-based entity builder (e.g.
/// [`EntityCommands`] or [`UiBuilder<Entity>`](crate::prelude::UiBuilder)) that can be used to arbitrarily
/// modify the referenced scene node entity.
pub struct SceneHandle<'a, T>
where
    T: scene_traits::SceneNodeBuilderOuter<'a>,
{
    scene_builder: &'a mut SceneBuilderInner,
    builder: T,
    scene: SceneRef,
}

impl<'a, T> SceneHandle<'a, T>
where
    T: scene_traits::SceneNodeBuilderOuter<'a>,
{
    fn get_impl(&mut self, scene: SceneRef) -> SceneHandle<T::Builder<'_>>
    {
        let Some(entity) = self
            .scene_builder
            .active_scene()
            .map(|s| s.get(&scene.path))
            .flatten()
        else {
            match self.scene_builder.active_scene() {
                Some(s) => {
                    tracing::warn!("edit failed for scene node {:?}, path is not present in the active scene {:?} on {:?}",
                        scene, s.scene_ref(), s.root_entity());
                }
                None => {
                    tracing::error!("edit failed for scene node {:?}, no scene is active (this is a bug)", scene);
                }
            }
            return SceneHandle {
                scene_builder: self.scene_builder,
                builder: self.builder.new_with(Entity::PLACEHOLDER),
                scene: self.scene.clone(),
            };
        };

        SceneHandle {
            scene_builder: self.scene_builder,
            builder: self.builder.new_with(entity),
            scene,
        }
    }

    fn edit_impl<C, R>(&mut self, scene: SceneRef, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut SceneHandle<'c, T::Builder<'c>>) -> R,
        R: CobwebResult,
    {
        let Some(entity) = self
            .scene_builder
            .active_scene()
            .map(|s| s.get(&scene.path))
            .flatten()
        else {
            match self.scene_builder.active_scene() {
                Some(s) => {
                    tracing::warn!("edit failed for scene node {:?}, path is not present in the active scene {:?} on {:?}",
                        scene, s.scene_ref(), s.root_entity());
                }
                None => {
                    tracing::error!("edit failed for scene node {:?}, no scene is active (this is a bug)", scene);
                }
            }
            return self;
        };

        let result = {
            let mut commands = self.builder.commands();
            let mut child_scene = SceneHandle {
                scene_builder: self.scene_builder,
                builder: T::scene_node_builder(&mut commands, entity),
                scene,
            };

            (callback)(&mut child_scene)
        };
        if result.need_to_handle() {
            self.builder.commands().queue(move |w: &mut World| {
                result.handle(w);
            });
        }

        self
    }

    /// Gets a specific child in order to edit it directly.
    pub fn get(&mut self, child: impl AsRef<str>) -> SceneHandle<T::Builder<'_>>
    {
        let scene = self.scene.e(child);
        self.get_impl(scene)
    }

    /// Gets a specific child positioned relative to the root node in order to edit it directly.
    pub fn get_from_root(&mut self, path: impl AsRef<str>) -> SceneHandle<T::Builder<'_>>
    {
        let scene = self.scene.extend_from_index(0, path);
        self.get_impl(scene)
    }

    /// Calls `callback` on the `child` of the current scene node.
    ///
    /// Prints a warning and does nothing if `child` does not point to a child of the current node in the scene
    /// that is currently being edited.
    ///
    /// Note that looking up the scene node allocates.
    pub fn edit<C, R>(&mut self, child: impl AsRef<str>, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut SceneHandle<'c, T::Builder<'c>>) -> R,
        R: CobwebResult,
    {
        let scene = self.scene.e(child);
        self.edit_impl(scene, callback)
    }

    /// Calls `callback` on the scene node designated by `path` relative to the root node of the scene.
    ///
    /// Prints a warning and does nothing if `path` does not point to a node in the scene
    /// that is currently being edited.
    ///
    /// Note that looking up the scene node allocates.
    pub fn edit_from_root<C, R>(&mut self, path: impl AsRef<str>, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut SceneHandle<'c, T::Builder<'c>>) -> R,
        R: CobwebResult,
    {
        let scene = self.scene.extend_from_index(0, path);
        self.edit_impl(scene, callback)
    }

    /// Gets an entity relative to the current node.
    ///
    /// Note that this lookup allocates.
    pub fn get_entity(&self, child: impl AsRef<str>) -> Result<Entity, SceneHandleError>
    {
        let child_path = self.scene.path.extend(child);
        self.scene_builder
            .active_scene()
            .map(|s| s.get(&child_path))
            .flatten()
            .ok_or_else(move || SceneHandleError::GetEntity(child_path))
    }

    /// Gets an entity relative to the root node.
    ///
    /// Note that this lookup allocates.
    pub fn get_entity_from_root(&self, path: impl AsRef<str>) -> Result<Entity, SceneHandleError>
    {
        let scene = self.scene.path.extend_from_index(0, path);
        self.scene_builder
            .active_scene()
            .map(|s| s.get(&scene))
            .flatten()
            .ok_or_else(move || SceneHandleError::GetEntityFromRoot(scene))
    }

    /// See [`SpawnSceneExt::spawn_scene_simple`].
    pub fn spawn_scene_simple(&mut self, path: impl Into<SceneRef>) -> &mut Self
    {
        self.spawn_scene(path, |_| {})
    }

    /// See [`SpawnSceneExt::spawn_scene`].
    pub fn spawn_scene<C, R>(&mut self, path: impl Into<SceneRef>, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut SceneHandle<'c, <T as scene_traits::SceneNodeBuilder>::Builder<'c>>) -> R,
        R: CobwebResult,
    {
        spawn_scene_impl(&mut self.builder, path, self.scene_builder, callback);
        self
    }

    /// Gets the location of the current scene node.
    pub fn path(&self) -> &SceneRef
    {
        &self.scene
    }

    /// Accesses the inner spawner and builder.
    pub fn inner(&mut self) -> (&mut SceneBuilderInner, &mut T)
    {
        (self.scene_builder, &mut self.builder)
    }
}

impl<'a, T> Deref for SceneHandle<'a, T>
where
    T: scene_traits::SceneNodeBuilderOuter<'a>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        &self.builder
    }
}

impl<'a, T> DerefMut for SceneHandle<'a, T>
where
    T: scene_traits::SceneNodeBuilderOuter<'a>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.builder
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extention trait for building scenes into entities.
pub trait SpawnSceneExt: scene_traits::SceneNodeBuilder
{
    /// Equivalent to [`SpawnSceneExt::spawn_scene`] with no callback.
    fn spawn_scene_simple<'b>(
        &'b mut self,
        path: impl Into<SceneRef>,
        scene_builder: &'b mut SceneBuilderInner,
    ) -> &'b mut Self;

    /// Spawns an entity (and optionally makes it a child of
    /// [`Self::scene_parent_entity`](scene_traits::SceneNodeBuilder::scene_parent_entity)), then builds the scene
    /// at `path` into it.
    ///
    /// Building a scene involves applying loadables (components and instructions) to nodes and spawning new
    /// child nodes.
    ///
    /// The `callback` can be used to edit the scene's root node, which in turn can be used to edit inner nodes
    /// of the scene via [`SceneHandle::edit`].
    ///
    /// Will log a warning and do nothing if the parent entity does not exist.
    fn spawn_scene<'b, C, R>(
        &'b mut self,
        path: impl Into<SceneRef>,
        scene_builder: &'b mut SceneBuilderInner,
        callback: C,
    ) -> &'b mut Self
    where
        C: for<'a> FnOnce(&mut SceneHandle<'a, <Self as scene_traits::SceneNodeBuilder>::Builder<'a>>) -> R,
        R: CobwebResult;
}

impl<T> SpawnSceneExt for T
where
    T: scene_traits::SceneNodeBuilder,
{
    fn spawn_scene_simple<'b>(
        &'b mut self,
        path: impl Into<SceneRef>,
        scene_builder: &'b mut SceneBuilderInner,
    ) -> &'b mut Self
    {
        self.spawn_scene(path, scene_builder, |_| {})
    }

    fn spawn_scene<'b, C, R>(
        &'b mut self,
        path: impl Into<SceneRef>,
        scene_builder: &'b mut SceneBuilderInner,
        callback: C,
    ) -> &'b mut Self
    where
        C: for<'a> FnOnce(&mut SceneHandle<'a, <T as scene_traits::SceneNodeBuilder>::Builder<'a>>) -> R,
        R: CobwebResult,
    {
        spawn_scene_impl(self, path, scene_builder, callback)
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<'w, 's> scene_traits::SceneNodeBuilder for Commands<'w, 's>
{
    type Builder<'a> = EntityCommands<'a>;

    fn commands(&mut self) -> Commands
    {
        self.reborrow()
    }

    fn scene_parent_entity(&self) -> Option<Entity>
    {
        None
    }

    fn initialize_scene_node(_ec: &mut EntityCommands) {}

    fn scene_node_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Builder<'a>
    {
        commands.entity(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Builder<'_>
    {
        self.entity(entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl scene_traits::SceneNodeBuilder for EntityCommands<'_>
{
    type Builder<'a> = EntityCommands<'a>;

    fn commands(&mut self) -> Commands
    {
        self.commands()
    }

    fn scene_parent_entity(&self) -> Option<Entity>
    {
        Some(self.id())
    }

    fn initialize_scene_node(_ec: &mut EntityCommands) {}

    fn scene_node_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Builder<'a>
    {
        commands.entity(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Builder<'_>
    {
        self.commands_mut().entity(entity)
    }
}

impl<'a> scene_traits::SceneNodeBuilderOuter<'a> for EntityCommands<'a> {}

//-------------------------------------------------------------------------------------------------------------------

/// Shortcut for using [`SceneRef`] as a function parameter.
pub type EcsSceneHandle<'a> = SceneHandle<'a, EntityCommands<'a>>;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for registering entities to acquire loadables from scene nodes.
pub trait NodeBuilderExt
{
    /// Registers the current entity to acquire loadables from `scene_ref`.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn build(&mut self, scene_ref: SceneRef) -> &mut Self;

    /// Registers the current entity to acquire loadables from `scene_ref`.
    ///
    /// The `initializer` callback will be called before refreshing the `scene_ref` loadable set on the entity.
    ///
    /// This should only be called after entering state [`LoadState::Done`], because reacting to loads is disabled
    /// when the `hot_reload` feature is not present (which will typically be the case in production builds).
    fn build_with_initializer(&mut self, scene_ref: SceneRef, initializer: fn(&mut EntityCommands)) -> &mut Self;
}

impl NodeBuilderExt for EntityCommands<'_>
{
    fn build(&mut self, scene_ref: SceneRef) -> &mut Self
    {
        self.build_with_initializer(scene_ref, |_| {})
    }

    fn build_with_initializer(&mut self, scene_ref: SceneRef, initializer: fn(&mut EntityCommands)) -> &mut Self
    {
        self.insert(HasLoadables);

        let id = self.id();
        self.commands()
            .syscall((id, scene_ref, NodeInitializer { initializer }), build_from_ref);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

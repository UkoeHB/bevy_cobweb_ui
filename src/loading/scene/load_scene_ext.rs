pub use std::ops::{Deref, DerefMut}; // Re-export for ease of use.

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Put this trait in a separate module so it doesn't pollute the `prelude`.
pub mod scene_traits
{
    use bevy::ecs::system::EntityCommands;
    use bevy::prelude::*;

    #[allow(unused_imports)]
    use crate::prelude::*;

    /// Helper trait for loading a scene. See [`LoadedScene`] and [`LoadSceneExt::load_scene`].
    pub trait SceneNodeLoader
    {
        /// The type returned by [`Self::loaded_scene_builder`].
        type Loaded<'a>: LoadedSceneBuilder<'a>;

        /// Gets a [`Commands`] instance.
        fn commands(&mut self) -> Commands;
        /// Gets the parent entity for scenes loaded with this `SceneLoader`.
        fn scene_parent_entity(&self) -> Option<Entity>;
        /// Prepares a scene node to receive scene data.
        ///
        /// For example, UI nodes need `NodeBundles`, which this method can auto-insert.
        fn initialize_scene_node(ec: &mut EntityCommands);
        /// Gets a [`LoadedSceneBuilder`] instance in order to edit a node in the loaded scene.
        fn loaded_scene_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Loaded<'a>;
        /// Gets a [`LoadedSceneBuilder`] instance in order to edit a node in the loaded scene.
        fn new_with(&mut self, entity: Entity) -> Self::Loaded<'_>;
    }

    /// Helper trait for editing nodes in a loaded scene. See [`LoadedScene`] and
    /// [`LoadSceneExt::load_scene_and_edit`].
    pub trait LoadedSceneBuilder<'a>: SceneNodeLoader {}
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper struct for editing loaded scenes.
///
/// The struct will dereference to the inner `T`, which should be a [`Commands`]-based entity builder (e.g.
/// [`EntityCommands`] or [`UiBuilder<Entity>`](crate::prelude::UiBuilder)) that can be used to arbitrarily
/// modify the scene node entity.
pub struct LoadedScene<'a, T>
where
    T: scene_traits::LoadedSceneBuilder<'a>,
{
    scene_loader: &'a mut SceneLoader,
    builder: T,
    scene: SceneRef,
}

impl<'a, T> LoadedScene<'a, T>
where
    T: scene_traits::LoadedSceneBuilder<'a>,
{
    fn get_impl(&mut self, scene: SceneRef) -> LoadedScene<T::Loaded<'_>>
    {
        let Some(entity) = self
            .scene_loader
            .active_scene()
            .map(|s| s.get(&scene.path))
            .flatten()
        else {
            match self.scene_loader.active_scene() {
                Some(s) => {
                    tracing::warn!("edit failed for scene node {:?}, path is not present in the active scene {:?} on {:?}",
                        scene, s.scene_ref(), s.root_entity());
                }
                None => {
                    tracing::error!("edit failed for scene node {:?}, no scene is active (this is a bug)", scene);
                }
            }
            return LoadedScene {
                scene_loader: self.scene_loader,
                builder: self.builder.new_with(Entity::PLACEHOLDER),
                scene: self.scene.clone(),
            };
        };

        LoadedScene {
            scene_loader: self.scene_loader,
            builder: self.builder.new_with(entity),
            scene,
        }
    }

    fn edit_impl<C>(&mut self, scene: SceneRef, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut LoadedScene<'c, T::Loaded<'c>>),
    {
        let Some(entity) = self
            .scene_loader
            .active_scene()
            .map(|s| s.get(&scene.path))
            .flatten()
        else {
            match self.scene_loader.active_scene() {
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

        {
            let mut commands = self.builder.commands();
            let mut child_scene = LoadedScene {
                scene_loader: self.scene_loader,
                builder: T::loaded_scene_builder(&mut commands, entity),
                scene,
            };

            (callback)(&mut child_scene);
        }

        self
    }

    /// Gets a specific child in order to edit it directly.
    pub fn get(&mut self, child: impl AsRef<str>) -> LoadedScene<T::Loaded<'_>>
    {
        let scene = self.scene.e(child);
        self.get_impl(scene)
    }

    /// Gets a specific child positioned relative to the root node in order to edit it directly.
    pub fn get_from_root(&mut self, path: impl AsRef<str>) -> LoadedScene<T::Loaded<'_>>
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
    pub fn edit<C>(&mut self, child: impl AsRef<str>, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut LoadedScene<'c, T::Loaded<'c>>),
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
    pub fn edit_from_root<C>(&mut self, path: impl AsRef<str>, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut LoadedScene<'c, T::Loaded<'c>>),
    {
        let scene = self.scene.extend_from_index(0, path);
        self.edit_impl(scene, callback)
    }

    /// Gets an entity relative to the current node.
    ///
    /// Note that this lookup allocates.
    pub fn get_entity(&self, child: impl AsRef<str>) -> Option<Entity>
    {
        let child_path = self.scene.path.extend(child);
        self.scene_loader
            .active_scene()
            .map(|s| s.get(&child_path))
            .flatten()
    }

    /// Gets an entity relative to the root node.
    ///
    /// Note that this lookup allocates.
    pub fn get_entity_from_root(&self, path: impl AsRef<str>) -> Option<Entity>
    {
        let scene = self.scene.path.extend_from_index(0, path);
        self.scene_loader
            .active_scene()
            .map(|s| s.get(&scene))
            .flatten()
    }

    /// See [`LoadSceneExt::load_scene`].
    pub fn load_scene(&mut self, path: impl Into<SceneRef>) -> &mut Self
    {
        self.builder.load_scene(path, self.scene_loader);
        self
    }

    /// See [`LoadSceneExt::load_scene_and_edit`].
    pub fn load_scene_and_edit<C>(&mut self, path: impl Into<SceneRef>, callback: C) -> &mut Self
    where
        C: for<'c> FnOnce(&mut LoadedScene<'c, <T as scene_traits::SceneNodeLoader>::Loaded<'c>>),
    {
        self.builder
            .load_scene_and_edit(path, self.scene_loader, callback);
        self
    }

    /// Gets the location of the current scene node.
    pub fn path(&self) -> &SceneRef
    {
        &self.scene
    }

    /// Accesses the inner loader and builder.
    pub fn inner(&mut self) -> (&mut SceneLoader, &mut T)
    {
        (self.scene_loader, &mut self.builder)
    }
}

impl<'a, T> Deref for LoadedScene<'a, T>
where
    T: scene_traits::LoadedSceneBuilder<'a>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        &self.builder
    }
}

impl<'a, T> DerefMut for LoadedScene<'a, T>
where
    T: scene_traits::LoadedSceneBuilder<'a>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.builder
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extention trait for loading scenes into entities.
pub trait LoadSceneExt: scene_traits::SceneNodeLoader
{
    /// Equivalent to [`LoadSceneExt::load_scene_and_edit`] with no callback.
    fn load_scene(&mut self, path: impl Into<SceneRef>, scene_loader: &mut SceneLoader) -> &mut Self;

    /// Spawns an entity (and optionally makes it a child of
    /// [`Self::scene_parent_entity`](scene_traits::SceneNodeLoader::scene_parent_entity)), then loads the scene at
    /// `path` into it.
    ///
    /// The `callback` can be used to edit the scene's root node, which in turn can be used to edit inner nodes
    /// of the scene via [`LoadedScene::edit`].
    ///
    /// Will log a warning and do nothing if the parent entity does not exist.
    fn load_scene_and_edit<C>(
        &mut self,
        path: impl Into<SceneRef>,
        scene_loader: &mut SceneLoader,
        callback: C,
    ) -> &mut Self
    where
        C: for<'a> FnOnce(&mut LoadedScene<'a, <Self as scene_traits::SceneNodeLoader>::Loaded<'a>>);
}

impl<T> LoadSceneExt for T
where
    T: scene_traits::SceneNodeLoader,
{
    fn load_scene(&mut self, path: impl Into<SceneRef>, scene_loader: &mut SceneLoader) -> &mut Self
    {
        self.load_scene_and_edit(path, scene_loader, |_| {})
    }

    fn load_scene_and_edit<C>(
        &mut self,
        path: impl Into<SceneRef>,
        scene_loader: &mut SceneLoader,
        callback: C,
    ) -> &mut Self
    where
        C: for<'a> FnOnce(&mut LoadedScene<'a, <T as scene_traits::SceneNodeLoader>::Loaded<'a>>),
    {
        let path = path.into();

        // Spawn either a child or a raw entity to be the scene's root node.
        let root_entity = self
            .scene_parent_entity()
            .map(|parent| self.commands().spawn_empty().set_parent(parent).id())
            .unwrap_or_else(|| self.commands().spawn_empty().id());

        // Avoid panicking if the parent is invalid.
        if self.commands().get_entity(root_entity).is_none() {
            tracing::warn!("failed loading scene at {:?}; parent {root_entity:?} does not exist", path);
            return self;
        }

        // Load the scene into the root entity.
        let mut commands = self.commands();
        if !scene_loader.load_scene::<T>(&mut commands, root_entity, path.clone()) {
            return self;
        }

        // Allow editing the scene via callback.
        {
            let mut root_node = LoadedScene {
                scene_loader,
                builder: T::loaded_scene_builder(&mut commands, root_entity),
                scene: path,
            };

            (callback)(&mut root_node);
        }

        // Cleanup
        scene_loader.release_active_scene();

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<'w, 's> scene_traits::SceneNodeLoader for Commands<'w, 's>
{
    type Loaded<'a> = EntityCommands<'a>;

    fn commands(&mut self) -> Commands
    {
        self.reborrow()
    }

    fn scene_parent_entity(&self) -> Option<Entity>
    {
        None
    }

    fn initialize_scene_node(_ec: &mut EntityCommands) {}

    fn loaded_scene_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Loaded<'a>
    {
        commands.entity(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Loaded<'_>
    {
        self.entity(entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl scene_traits::SceneNodeLoader for EntityCommands<'_>
{
    type Loaded<'a> = EntityCommands<'a>;

    fn commands(&mut self) -> Commands
    {
        self.commands()
    }

    fn scene_parent_entity(&self) -> Option<Entity>
    {
        Some(self.id())
    }

    fn initialize_scene_node(_ec: &mut EntityCommands) {}

    fn loaded_scene_builder<'a>(commands: &'a mut Commands, entity: Entity) -> Self::Loaded<'a>
    {
        commands.entity(entity)
    }

    fn new_with(&mut self, entity: Entity) -> Self::Loaded<'_>
    {
        self.commands_mut().entity(entity)
    }
}

impl<'a> scene_traits::LoadedSceneBuilder<'a> for EntityCommands<'a> {}

//-------------------------------------------------------------------------------------------------------------------

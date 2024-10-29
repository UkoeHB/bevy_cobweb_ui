use std::any::{type_name, TypeId};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::utils::warn_once;
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn preprocess_cobweb_asset_files(
    mut manifest_buffer: Local<HashMap<CafFile, ManifestKey>>,
    asset_server: Res<AssetServer>,
    mut events: EventReader<AssetEvent<CobwebAssetFile>>,
    mut caf_files: ResMut<LoadedCobwebAssetFiles>,
    mut assets: ResMut<Assets<CobwebAssetFile>>,
    mut caf_cache: ResMut<CobwebAssetCache>,
)
{
    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => id,
            _ => {
                tracing::debug!("ignoring CobwebAssetCache asset event {:?}", event);
                continue;
            }
        };

        let Some(handle) = caf_files.get_handle(*id) else {
            tracing::warn!("encountered CobwebAssetCache asset event {:?} for an untracked asset", id);
            continue;
        };

        let Some(asset) = assets.remove(&handle) else {
            tracing::error!("failed to remove CobwebAssetCache asset {:?}", handle);
            continue;
        };

        preprocess_caf_file(&mut manifest_buffer, &asset_server, &mut caf_files, &mut caf_cache, asset.0);
    }

    // Note: we don't try to handle asset load failures here because a file load failure is assumed to be
    // catastrophic.
}

//-------------------------------------------------------------------------------------------------------------------

fn process_cobweb_asset_files(
    types: Res<AppTypeRegistry>,
    mut caf_cache: ResMut<CobwebAssetCache>,
    mut c: Commands,
    mut scene_loader: ResMut<SceneLoader>,
)
{
    let type_registry = types.read();

    if caf_cache
        .process_cobweb_asset_files(&type_registry, &mut c, &mut scene_loader)
    {
        c.broadcast(CafCacheUpdated);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_pending_commands(mut c: Commands, mut caf_cache: ResMut<CobwebAssetCache>, loaders: Res<LoaderCallbacks>)
{
    caf_cache.apply_pending_commands(&mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

/// Only enabled for hot_reload because normally entities are loaded only once, the first time they subscribe
/// to a loadable ref.
#[cfg(feature = "hot_reload")]
fn apply_pending_updates(mut c: Commands, mut caf_cache: ResMut<CobwebAssetCache>, loaders: Res<LoaderCallbacks>)
{
    caf_cache.apply_pending_node_updates(&mut c, &loaders);
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
fn cleanup_cobweb_asset_cache(
    mut caf_cache: ResMut<CobwebAssetCache>,
    mut scene_loader: ResMut<SceneLoader>,
    mut removed: RemovedComponents<HasLoadables>,
)
{
    for removed in removed.read() {
        caf_cache.remove_entity(&mut scene_loader, removed);
    }

    caf_cache.cleanup_pending_updates();
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(not(feature = "hot_reload"))]
fn cleanup_cobweb_asset_cache(mut caf_cache: ResMut<CobwebAssetCache>)
{
    caf_cache.cleanup_pending_updates();
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: this needs to return Added/Updated/NoChange
fn insert_command_loadable_entry(
    loadables: &mut HashMap<SceneRef, SmallVec<[ErasedLoadable; 4]>>,
    loadable_ref: &SceneRef,
    loadable: ReflectedLoadable,
    type_id: TypeId,
    full_type_name: &str,
) -> bool
{
    match loadables.entry(loadable_ref.clone()) {
        Vacant(entry) => {
            let mut vec = SmallVec::default();
            vec.push(ErasedLoadable { type_id, loadable });
            entry.insert(vec);
        }
        Occupied(mut entry) => {
            // Insert if the loadable value changed.
            if let Some(erased_loadable) = entry.get_mut().iter_mut().find(|e| e.type_id == type_id) {
                match erased_loadable.loadable.equals(&loadable) {
                    Some(true) => return false,
                    Some(false) => {
                        // Replace the existing value.
                        *erased_loadable = ErasedLoadable { type_id, loadable };
                    }
                    None => {
                        tracing::error!("failed updating loadable {:?} at {:?}, its reflected value doesn't implement \
                            PartialEq", full_type_name, loadable_ref);
                        return false;
                    }
                }
            } else {
                entry.get_mut().push(ErasedLoadable { type_id, loadable });
            }
        }
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------

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
                return false;
            }
            let mut vec = SmallVec::default();
            vec.push(ErasedLoadable { type_id, loadable });
            entry.insert(vec);

            InsertNodeResult::Added
        }
        Occupied(mut entry) => {
            // Insert if the loadable value changed.
            if let Some(erased_loadable) = entry.get_mut().get_mut(index) {
                // Check if the type is changing.
                if erased_loadable.type_id != type_id {
                    let removed_type_id = erased_loadable.type_id;
                    *erased_loadable = ErasedLoadable { type_id, loadable };
                    return InsertNodeResult::Replaced(removed_type_id, type_id);
                }

                // Check if the value is changing.
                match erased_loadable.loadable.equals(&loadable) {
                    Some(true) => InsertNodeResult::NoChange,
                    Some(false) => {
                        // Replace the existing value.
                        *erased_loadable = ErasedLoadable { type_id, loadable };
                        InsertNodeResult::Changed
                    }
                    None => {
                        tracing::error!("failed updating loadable {:?} at {:?}, its reflected value doesn't implement \
                            PartialEq", full_type_name, loadable_ref);
                        InsertNodeResult::NoChange;
                    }
                }
            } else if index == entry.get().len() {
                entry.get_mut().push(ErasedLoadable { type_id, loadable });
                InsertNodeResult::Changed
            } else {
                tracing::error!("failed inserting node loadable {:?} at {:?}; expected to insert at index {} but \
                    the current loadables length is {}", full_type_name, loadable_ref, index, entry.get().len());
                InsertNodeResult::NoChange;
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(PartialEq)]
enum InsertNodeResult
{
    /// (removed, added)
    Replaced(TypeId, TypeId),
    Changed,
    Added,
    NoChange,
}

//-------------------------------------------------------------------------------------------------------------------

struct RevertCommand
{
    entity: Entity,
    reverter: fn(Entity, &mut World),
}

impl Command for RevertCommand
{
    fn apply(self, world: &mut World)
    {
        (self.reverter)(self.entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct CommandLoadCommand
{
    callback: fn(&mut World, ReflectedLoadable, LoadableRef),
    loadable_ref: SceneRef,
    loadable: ReflectedLoadable
}

impl Command for CommandLoadCommand
{
    fn apply(self, world: &mut World)
    {
        (self.callback)(world, self.loadable, self.loadable_ref);
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct NodeLoadCommand
{
    callback: fn(&mut World, Entity, ReflectedLoadable, LoadableRef),
    entity: Entity,
    loadable_ref: SceneRef,
    loadable: ReflectedLoadable
}

impl Command for NodeLoadCommand
{
    fn apply(self, world: &mut World)
    {
        (self.callback)(world, self.entity, self.loadable, self.loadable_ref);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
struct PreprocessedSceneFile
{
    /// This file.
    file: CafFile,
    /// Imports for detecting when a re-load is required.
    /// - Can include both manifest keys and file paths.
    imports: HashMap<ManifestKey, CafImportAlias>,
    /// Data cached for re-loading when dependencies are reloaded.
    data: Caf,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
struct ProcessedSceneFile
{
    /// Using info cached for use by dependents.
    using: HashMap<&'static str, &'static str>,
    /// Constants info cached for use by dependents.
    constants_buff: ConstantsBuffer,
    /// Specs that can be imported into other files.
    specs: SpecsMap,
    /// Imports for detecting when a re-load is required.
    /// - Can include both manifest keys and file paths.
    #[cfg(feature = "hot_reload")]
    imports: HashMap<ManifestKey, CafImportAlias>,
    /// Data cached for re-loading when dependencies are reloaded.
    #[cfg(feature = "hot_reload")]
    data: Caf,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct ErasedLoadable
{
    type_id: TypeId,
    loadable: ReflectedLoadable,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub(crate) struct SubscriptionRef
{
    pub(crate) entity: Entity,
    pub(crate) initializer: NodeInitializer,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) enum ReflectedLoadable
{
    Value(Arc<Box<dyn Reflect + 'static>>),
    DeserializationFailed(Arc<CafError>),
}

impl ReflectedLoadable
{
    pub(crate) fn equals(&self, other: &ReflectedLoadable) -> Option<bool>
    {
        let (Self::Value(this), Self::Value(other)) = (self, other) else {
            return Some(false);
        };

        this.reflect_partial_eq(other.as_reflect())
    }

    pub(crate) fn get_value<T: Loadable>(&self, loadable_ref: &SceneRef) -> Option<T>
    {
        match self {
            ReflectedLoadable::Value(loadable) => {
                let Some(new_value) = T::from_reflect(loadable.as_reflect()) else {
                    let hint = Self::make_hint::<T>();
                    tracing::error!("failed reflecting loadable {:?} at path {:?} in file {:?}\n\
                        serialization hint: {}",
                        type_name::<T>(), loadable_ref.path.path, loadable_ref.file, hint.as_str());
                    return None;
                };
                Some(new_value)
            }
            ReflectedLoadable::DeserializationFailed(err) => {
                let hint = Self::make_hint::<T>();
                tracing::error!("failed deserializing loadable {:?} at path {:?} in file {:?}, {:?}\n\
                    serialization hint: {}",
                    type_name::<T>(), loadable_ref.path.path, loadable_ref.file, **err, hint.as_str());
                None
            }
        }
    }

    fn make_hint<T: Loadable>() -> String
    {
        let temp = T::default();
        match CafValue::extract(&temp) {
            Ok(value) => {
                let mut buff = Vec::<u8>::default();
                let mut serializer = DefaultRawSerializer::new(&mut buff);
                value.write_to(&mut serializer).unwrap();
                String::from_utf8(buff).unwrap()
            }
            Err(err) => format!("! hint serialization failed: {:?}", err),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that manages content loaded from cobweb asset files (`.caf.json` files).
///
/// Can be used to load scenes with [`LoadSceneExt::load_scene`], or load individual scene nodes with
/// [`CafLoadingEntityCommandsExt::load`].
///
/// Note that command loadables in caf files are automatically applied to the world.
#[derive(Resource, Default, Debug)]
pub struct CobwebAssetCache
{
    /// Tracks which files have not initialized yet.
    pending: HashSet<CafFile>,
    /// Tracks the total number of files that should load.
    ///
    /// Used for progress tracking on initial load.
    total_expected_sheets: usize,

    /// Tracks manifest data.
    /// - Inside an arc/mutex so the SceneLoader can also use it.
    manifest_map: Arc<Mutex<ManifestMap>>,
    /// Tracks which files have been assigned manifest keys.
    file_to_manifest_key: HashMap<CafFile, Option<ManifestKey>>,

    /// Tracks pre-processed files.
    preprocessed: Vec<PreprocessedSceneFile>,

    /// Records processed files.
    processed: HashMap<CafFile, ProcessedSceneFile>,

    /// Tracks loadable commands from all loaded files.
    command_loadables: HashMap<SceneRef, SmallVec<[ErasedLoadable; 4]>>,
    /// Tracks loadables from all loaded files.
    loadables: HashMap<SceneRef, SmallVec<[ErasedLoadable; 4]>>,

    /// Tracks subscriptions to scene paths.
    #[cfg(feature = "hot_reload")]
    subscriptions: HashMap<SceneRef, SmallVec<[SubscriptionRef; 1]>>,
    /// Tracks entities for cleanup.
    #[cfg(feature = "hot_reload")]
    subscriptions_rev: HashMap<Entity, SmallVec<[SceneRef; 1]>>,

    /// Records commands that need to be applied.
    commands_need_updates: Vec<(ErasedLoadable, SceneRef)>,
    /// Records type ids of loadables that need to be reverted on specific entities.
    #[cfg(feature = "hot_reload")]
    needs_revert: EntityHashMap<HashSet<TypeId>>,
    /// Records entities that need loadable updates.
    #[cfg(feature = "hot_reload")]
    needs_updates: EntityHashMap<(NodeInitializer, SceneRef)>,
}

impl CobwebAssetCache
{
    fn new(manifest_map: Arc<Mutex<ManifestMap>>) -> Self
    {
        Self { manifest_map, ..default() }
    }

    fn manifest_map(&mut self) -> MutexGuard<ManifestMap>
    {
        self.manifest_map.lock().unwrap()
    }

    /// Gets the CobwebAssetCache's loading progress on startup.
    ///
    /// Returns `(num uninitialized files, num total files)`.
    ///
    /// Does not include files recursively loaded via manifests.
    pub fn loading_progress(&self) -> (usize, usize)
    {
        (self.pending.len(), self.total_expected_sheets)
    }

    /// Gets the number of files waiting to be processed.
    fn num_preprocessed_pending(&self) -> usize
    {
        self.preprocessed.len()
    }

    /// Prepares a cobweb asset file.
    pub(crate) fn prepare_file(&mut self, file: CafFile)
    {
        let _ = self.pending.insert(file.clone());
        self.total_expected_sheets += 1;

        // Make sure this file has a manifest key entry, which indicates it doesn't need to be initialized again
        // if it is present in a manifest section.
        self.register_manifest_key(file, None);
    }

    /// Sets the manifest key for a file.
    ///
    /// The `manifest_key` may be `None` if loaded via the App extension. We use manifest key presence as a proxy
    /// for whether or not a file has been initialized.
    ///
    /// Returns `true` if this is the first time this file's manifest key has been registered. This can be used
    /// to decide whether to start loading transitive imports/manifests.
    pub(crate) fn register_manifest_key(&mut self, file: CafFile, manifest_key: Option<ManifestKey>) -> bool
    {
        match self.file_to_manifest_key.entry(file.clone()) {
            Vacant(entry) => {
                entry.insert(manifest_key.clone());

                if let Some(new_key) = manifest_key {
                    if let Some(prev_file) = self.manifest_map().insert(new_key.clone(), file.clone()) {
                        tracing::warn!("replacing file for manifest key {:?} (old: {:?}, new: {:?})",
                            new_key, prev_file, file);
                    }
                }

                true
            }
            Occupied(mut entry) => {
                // Manifest key can be None when a file is loaded via the App extension.
                let Some(new_key) = manifest_key else {
                    return false;
                };

                match entry.get_mut() {
                    // Error case: manifest key changing.
                    Some(prev_key) => {
                        if *prev_key == new_key {
                            return false;
                        }

                        tracing::warn!("changing manifest key for {:?} (old: {:?}, new: {:?})",
                            file, prev_key, new_key);
                        let prev = prev_key.clone();
                        *prev_key = new_key.clone();
                        self.manifest_map().remove(&prev);
                    }
                    // Normal case: setting the manifest key.
                    None => {
                        *entry.get_mut() = Some(new_key.clone());
                    }
                }

                if let Some(prev_file) = self.manifest_map().insert(new_key.clone(), file.clone()) {
                    tracing::warn!("replacing file for manifest key {:?} (old: {:?}, new: {:?})",
                        new_key, prev_file, file);
                }

                false
            }
        }
    }

    /// Initializes a file that has been loaded.
    pub(crate) fn initialize_file(&mut self, file: &CafFile)
    {
        let _ = self.pending.remove(file);
    }

    /// Inserts a preprocessed file for later processing.
    pub(crate) fn add_preprocessed_file(
        &mut self,
        file: CafFile,
        imports: HashMap<ManifestKey, CafImportAlias>,
        data: Caf,
    )
    {
        // Check that all dependencies are known.
        // - Note: We don't need to check for circular dependencies here. It can be checked after processing files
        //   by seeing if there are any pending files remaining. Once all pending files are loaded, if a file fails
        //   to process that implies it has circular dependencies.
        for import in imports.keys() {
            // Try to convert to file. This may fail if the imported file is not initialized yet.
            let Some(import_file) = self.manifest_map().get(import) else { continue };

            // Check if pending.
            if self.pending.contains(&import_file) {
                continue;
            }

            // Check if processed.
            if self.processed.contains_key(&import_file) {
                continue;
            }

            // Check if preprocessed.
            if self.preprocessed.iter().any(|p| p.file == import_file) {
                continue;
            }

            tracing::error!("ignoring loadable file {:?} that points to an untracked file {}; this is a bug",
                file.as_str(), import.as_str());
            return;
        }

        let preprocessed = PreprocessedSceneFile { file, imports, data };
        self.preprocessed.push(preprocessed);
    }

    /// Converts a preprocessed file to a processed file.
    ///
    /// Assumes all imports are available.
    fn process_cobweb_asset_file(
        &mut self,
        preprocessed: PreprocessedSceneFile,
        type_registry: &TypeRegistry,
        c: &mut Commands,
        scene_loader: &mut SceneLoader,
    )
    {
        // Initialize using/constants maps from dependencies.
        // [ shortname : longname ]
        let mut name_shortcuts: HashMap<&'static str, &'static str> = HashMap::default();
        let mut constants_buff = ConstantsBuffer::default();
        // specs collector
        let mut specs = SpecsMap::default();

        for (dependency, alias) in preprocessed.imports.iter() {
            let Some(dependency) = self.manifest_map().get(&dependency) else {
                tracing::error!("failed extracting import {:?} for {:?}; failed manifest key lookup (this is a bug)",
                    dependency, preprocessed.file);
                continue;
            };
            let Some(processed) = self.processed.get(&dependency) else {
                tracing::error!("failed extracting import {:?} for {:?}; dependency is not processed (this is a bug)",
                    dependency, preprocessed.file);
                continue;
            };

            name_shortcuts.extend(processed.using.iter());
            constants_buff.append(alias, &processed.constants_buff);
            specs.import_specs(&dependency, &preprocessed.file, &processed.specs);
        }

        // Prepare to process the file.
        let mut processed = ProcessedSceneFile::default();

        #[cfg(feature = "hot_reload")]
        {
            processed.imports = preprocessed.imports;
            processed.data = preprocessed.data.clone();
        }

        // Process the file.
        // - This updates the using/constants/specs maps with info extracted from the file.
        extract_caf_data(
            type_registry,
            c,
            self,
            scene_loader,
            preprocessed.file.clone(),
            preprocessed.data,
            &mut name_shortcuts,
            &mut constants_buff,
            &mut specs,
        );

        // Save final maps.
        processed.using = name_shortcuts;
        processed.constants_buff = constants_buff;
        processed.specs = specs;

        self.processed.insert(preprocessed.file.clone(), processed);

        // Check for already-processed files that need to rebuild since they depend on this file.
        #[cfg(feature = "hot_reload")]
        {
            if let Some(manifest_key) = self
                .file_to_manifest_key
                .get(&preprocessed.file)
                .cloned()
                .flatten()
            {
                let needs_rebuild: Vec<CafFile> = self
                    .processed
                    .iter()
                    .filter_map(|(file, processed)| {
                        if processed.imports.contains_key(&manifest_key) {
                            return Some(file.clone());
                        }
                        None
                    })
                    .collect();

                for needs_rebuild in needs_rebuild {
                    let processed = self.processed.remove(&needs_rebuild).unwrap();
                    // Add via API to check for recursive dependencies.
                    self.add_preprocessed_file(needs_rebuild, processed.imports, processed.data);
                }
            }
        }
    }

    /// Converts preprocessed files to processed files.
    ///
    /// Returns `true` if at least one file was processed.
    fn process_cobweb_asset_files(
        &mut self,
        type_registry: &TypeRegistry,
        c: &mut Commands,
        scene_loader: &mut SceneLoader,
    ) -> bool
    {
        // Loop preprocessed until nothing can be processed.
        let mut num_processed = 0;
        let mut preprocessed = Vec::new();

        while !self.preprocessed.is_empty() {
            let num_already_processed = num_processed;
            preprocessed.clear();
            std::mem::swap(&mut self.preprocessed, &mut preprocessed);

            for preprocessed in preprocessed.drain(..) {
                // Check if any dependency is not ready.
                {
                    let manifest_map = self.manifest_map.lock().unwrap();
                    if preprocessed
                        .imports
                        .keys()
                        .any(|i| match manifest_map.get(i) {
                            Some(i) => !self.processed.contains_key(&i),
                            None => true,
                        })
                    {
                        self.preprocessed.push(preprocessed);
                        continue;
                    }
                }

                self.process_cobweb_asset_file(preprocessed, type_registry, c, scene_loader);
                num_processed += 1;
            }

            // Exit if no changes made.
            if num_already_processed == num_processed {
                break;
            }
        }

        // Check for failed loads.
        if self.pending.is_empty() && !self.preprocessed.is_empty() {
            for preproc in self.preprocessed.drain(..) {
                tracing::error!("discarding CAF file {:?} that failed to resolve imports; it either has a \
                    dependency cycle or tries to import unknown manifest keys", preproc.file.as_str());
            }
        }

        // Clean up memory once all files are loaded and processed.
        #[cfg(not(feature = "hot_reload"))]
        {
            if self.pending.is_empty() && self.preprocessed.is_empty() {
                tracing::info!("done loading (enable hot_reload feature if you want to reload files)");
                self.pending = HashSet::default();
                self.preprocessed = Vec::default();
                self.processed = HashMap::default();
            }
        }

        num_processed > 0
    }

    /// Inserts a loadable command if its value will change.
    ///
    /// Returns `true` if the command's saved value changed.
    pub(crate) fn insert_command(
        &mut self,
        loadable_ref: &SceneRef,
        loadable: ReflectedLoadable,
        type_id: TypeId,
        full_type_name: &str,
    ) -> bool
    {
        if !insert_command_loadable_entry(
            &mut self.command_loadables,
            loadable_ref,
            loadable.clone(),
            type_id,
            full_type_name,
        ) {
            return false;
        }

        // TODO: rework this so commands are globally ordered
        self.commands_need_updates.push((ErasedLoadable{ type_id, loadable: loadable.clone() }, loadable_ref.clone()));

        true
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
                // Note: we revert for additions because additions encompass both 'fresh adds' and
                // 'rearranged'.
                let revert_entry = self.needs_revert.entry(subscription.entity).or_default();
                if let InsertNodeResult::Replaced(removed, _) = &res {
                    entry.insert(*removed);
                }
                revert_entry.insert(type_id);
                self.needs_updates.insert(subscription.entity, (subscription.initializer, loadable_ref.clone()));
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
        for removed in self.loadables.get(loadable_ref).into_iter().flat_map(|l| l[count..]) {
            for subscription in subscriptions {
                self.needs_revert.entry(subscription.entity).or_default().insert(type_id);
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
        (subscription.initializer)(&mut ec);

        // Queue loadables
        let Some(loadables) = self.loadables.get(&loadable_ref) else {
            tracing::warn!("failed loading {loadable_ref:?} into {:?}, path is unknown; either the path is \
                invalid or you loaded the entity before LoadState::Done", subscription.entity);
            return;
        };

        for loadable in loadables.iter() {
            let Some(callback) = callbacks.get(loadable.type_id) else {
                tracing::warn!("found loadable at {:?} that wasn't registered with CobwebAssetRegistrationAppExt",
                    loadable_ref);
                continue;
            };

            c.add(NodeLoadCommand{
                callback,
                subscription.entity,
                loadable_ref: loadable_ref.clone(),
                loadable: loadable.loadable.clone()
            });
        }

        // Notify the entity that it loaded.
        #[cfg(feature = "hot_reload")]
        {
            if !loadables.is_empty() {
                c.react().entity_event(entity, Loaded);
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
            self.subscriptions_rev
                .entry(entity)
                .or_default()
                .push(loadable_ref.clone());
        }

        // Load the entity immediately.
        self.load_entity(subscription, loadable_ref, callbacks, c);
    }

    /// Schedules all pending commands to be processed.
    fn apply_pending_commands(&mut self, c: &mut Commands, callbacks: &LoaderCallbacks)
    {
        for (loadable, loadable_ref) in self.commands_need_updates.drain(..) {
            let Some(callback) = callbacks.get(loadable.type_id) else {
                tracing::warn!("found loadable at {:?} that wasn't registered with CobwebAssetRegistrationAppExt",
                    loadable_ref);
                continue;
            };

            c.add(CommandLoadCommand{
                callback,
                loadable_ref: loadable_ref.clone(),
                loadable: loadable.loadable.clone()
            });
        }
    }

    #[cfg(feature = "hot_reload")]
    fn apply_pending_node_updates(&mut self, c: &mut Commands, callbacks: &LoaderCallbacks)
    {
        // Revert loadables as needed.
        for (entity, type_ids) in self.needs_revert.drain(..) {
            for type_id in type_ids {
                let Some(reverter) = callbacks.get_for_revert(type_id) else { continue };
                c.add(RevertCommand{ entity, reverter });
            }
        }

        // Reload entities.
        let mut need_updates = std::mem::take(&mut self.needs_updates);
        for (entity, (initializer, loadable_ref)) in need_updates.drain(..) {
            self.load_entity(SubscriptionRef{ entity, initializer}, loadable_ref, callbacks, c);
        }
        self.needs_updates = needs_updates;
    }

    /// Cleans up despawned entities.
    #[cfg(feature = "hot_reload")]
    fn remove_entity(&mut self, scene_loader: &mut SceneLoader, dead_entity: Entity)
    {
        let Some(loadable_refs) = self.subscriptions_rev.remove(&dead_entity) else { return };
        for loadable_ref in loadable_refs {
            // Clean up scenes.
            scene_loader.cleanup_dead_entity(&loadable_ref, dead_entity);

            // Clean up subscription.
            let Some(subscribed) = self.subscriptions.get_mut(&loadable_ref) else { continue };
            let Some(dead) = subscribed.iter().position(|s| s.entity == dead_entity) else { continue };
            subscribed.swap_remove(dead);
        }
    }

    /// Cleans up pending updates that failed to be processed.
    // TODO: remove? does rework make this unnecessary?
    fn cleanup_pending_updates(&mut self)
    {

    }
}

impl AssetLoadProgress for CobwebAssetCache
{
    fn pending_assets(&self) -> usize
    {
        self.loading_progress().0
    }

    fn total_assets(&self) -> usize
    {
        self.loading_progress().1
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when the [`CobwebAssetCache`] has been updated with CAF asset data.
pub struct CafCacheUpdated;

//-------------------------------------------------------------------------------------------------------------------

/// System set in [`First`] where files are processed.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct FileProcessingSet;

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that enables loading.
pub(crate) struct CobwebAssetCachePlugin;

impl Plugin for CobwebAssetCachePlugin
{
    fn build(&self, app: &mut App)
    {
        let manifest_map = Arc::new(Mutex::new(ManifestMap::default()));
        app.insert_resource(CobwebAssetCache::new(manifest_map.clone()))
            .insert_resource(SceneLoader::new(manifest_map))
            .register_asset_tracker::<CobwebAssetCache>()
            .add_systems(
                First,
                (
                    preprocess_cobweb_asset_files,
                    process_cobweb_asset_files
                        .run_if(|s: Res<CobwebAssetCache>| s.num_preprocessed_pending() > 0),
                    apply_pending_commands,
                    #[cfg(feature = "hot_reload")]
                    apply_pending_node_updates,
                )
                    .chain()
                    .in_set(FileProcessingSet),
            )
            .add_systems(Last, cleanup_cobweb_asset_cache);
    }
}

//-------------------------------------------------------------------------------------------------------------------

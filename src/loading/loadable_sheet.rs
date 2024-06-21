use std::any::{type_name, TypeId};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::utils::warn_once;
use bevy_cobweb::prelude::*;
use serde_json::{Map, Value};
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn preprocess_loadable_files(
    asset_server: Res<AssetServer>,
    mut events: EventReader<AssetEvent<LoadableSheetAsset>>,
    mut sheet_list: ResMut<LoadableSheetList>,
    mut assets: ResMut<Assets<LoadableSheetAsset>>,
    mut loadablesheet: ReactResMut<LoadableSheet>,
)
{
    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => id,
            _ => {
                tracing::debug!("ignoring loadablesheet asset event {:?}", event);
                continue;
            }
        };

        let Some(handle) = sheet_list.get_handle(*id) else {
            tracing::warn!("encountered loadablesheet asset event {:?} for an untracked asset", id);
            continue;
        };

        let Some(asset) = assets.remove(handle) else {
            tracing::error!("failed to remove loadablesheet asset {:?}", handle);
            continue;
        };

        let loadablesheet = loadablesheet.get_noreact();
        preprocess_loadablesheet_file(&asset_server, &mut sheet_list, loadablesheet, asset.file, asset.data);
    }

    // Note: we don't try to handle asset load failures here because a file load failure is assumed to be
    // catastrophic.
}

//-------------------------------------------------------------------------------------------------------------------

fn process_loadable_files(
    mut c: Commands,
    mut loadablesheet: ReactResMut<LoadableSheet>,
    types: Res<AppTypeRegistry>,
)
{
    let type_registry = types.read();

    if loadablesheet.get_noreact().process_sheets(&type_registry) {
        loadablesheet.get_mut(&mut c);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn cleanup_loadablesheet(
    mut loadablesheet: ReactResMut<LoadableSheet>,
    mut removed: RemovedComponents<HasLoadables>,
)
{
    for removed in removed.read() {
        loadablesheet.get_noreact().remove_entity(removed);
    }

    loadablesheet.get_noreact().cleanup_pending_updates();
}

//-------------------------------------------------------------------------------------------------------------------

fn insert_loadable_entry(
    loadables: &mut HashMap<LoadableRef, SmallVec<[ErasedLoadable; 4]>>,
    loadable_ref: &LoadableRef,
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

#[derive(Default, Debug)]
struct PreprocessedLoadableFile
{
    /// This file.
    file: LoadableFile,
    /// Imports for detecting when a re-load is required.
    imports: HashMap<LoadableFile, SmolStr>,
    /// Data cached for re-loading when dependencies are reloaded.
    data: Map<String, Value>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
struct ProcessedLoadableFile
{
    /// Using info cached for use by dependents.
    using: HashMap<&'static str, &'static str>,
    /// Constants info cached for use by dependents.
    constants: HashMap<SmolStr, Map<String, Value>>,
    /// Specs that can be imported into other files.
    specs: SpecsMap,
    /// Imports for detecting when a re-load is required.
    #[cfg(feature = "hot_reload")]
    imports: HashMap<LoadableFile, SmolStr>,
    /// Data cached for re-loading when dependencies are reloaded.
    #[cfg(feature = "hot_reload")]
    data: Map<String, Value>,
}

//-------------------------------------------------------------------------------------------------------------------

struct ErasedLoadable
{
    type_id: TypeId,
    loadable: ReflectedLoadable,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone)]
struct RefSubscription
{
    entity: Entity,
    setter: ContextSetter,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) enum ReflectedLoadable
{
    Value(Arc<Box<dyn Reflect + 'static>>),
    DeserializationFailed(Arc<serde_json::Error>),
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

    pub(crate) fn get_value<T: Loadable>(&self, loadable_ref: &LoadableRef) -> Option<T>
    {
        match self {
            ReflectedLoadable::Value(loadable) => {
                let Some(new_value) = T::from_reflect(loadable.as_reflect()) else {
                    let temp = T::default();
                    let mut hint = serde_json::to_string_pretty(&temp).unwrap();
                    if hint.len() > 250 {
                        hint = serde_json::to_string(&temp).unwrap();
                    }
                    tracing::error!("failed reflecting loadable {:?} at path {:?} in file {:?}\n\
                        serialization hint: {}",
                        type_name::<T>(), loadable_ref.path.path, loadable_ref.file, hint.as_str());
                    return None;
                };
                Some(new_value)
            }
            ReflectedLoadable::DeserializationFailed(err) => {
                let temp = T::default();
                let mut hint = serde_json::to_string_pretty(&temp).unwrap();
                if hint.len() > 250 {
                    hint = serde_json::to_string(&temp).unwrap();
                }
                tracing::error!("failed deserializing loadable {:?} at path {:?} in file {:?}, {:?}\n\
                    serialization hint: {}",
                    type_name::<T>(), loadable_ref.path.path, loadable_ref.file, **err, hint.as_str());
                None
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive resource for managing loadables loaded from loadablesheet assets.
#[derive(ReactResource, Default)]
pub struct LoadableSheet
{
    /// Tracks which files have not initialized yet.
    pending: HashSet<LoadableFile>,
    /// Tracks the total number of loadable sheets that should load.
    ///
    /// Used for progress tracking on initial load.
    total_expected_sheets: usize,

    /// Tracks manifest data.
    manifest_map: HashMap<Arc<str>, LoadableFile>,
    /// Tracks which files have been assigned manifest keys.
    file_to_manifest_key: HashMap<LoadableFile, Option<Arc<str>>>,

    /// Tracks pre-processed files.
    preprocessed: Vec<PreprocessedLoadableFile>,

    /// Records processed files.
    processed: HashMap<LoadableFile, ProcessedLoadableFile>,

    /// Tracks loadable commands from all loaded files.
    command_loadables: HashMap<LoadableRef, SmallVec<[ErasedLoadable; 4]>>,
    /// Tracks loadables from all loaded files.
    loadables: HashMap<LoadableRef, SmallVec<[ErasedLoadable; 4]>>,

    /// Tracks subscriptions to loadable paths.
    subscriptions: HashMap<LoadableRef, SmallVec<[RefSubscription; 1]>>,
    /// Tracks entities for cleanup.
    subscriptions_rev: HashMap<Entity, SmallVec<[LoadableRef; 1]>>,

    /// Records commands that need to be applied.
    /// - We clear this at the end of every tick, so there should not be stale `ReflectedLoadable` values.
    commands_need_updates: HashMap<TypeId, SmallVec<[(ReflectedLoadable, LoadableRef); 1]>>,
    /// Records entities that need loadable updates.
    /// - We clear this at the end of every tick, so there should not be stale `ReflectedLoadable` values.
    needs_updates:
        HashMap<TypeId, SmallVec<[(ReflectedLoadable, LoadableRef, SmallVec<[RefSubscription; 1]>); 1]>>,
}

impl LoadableSheet
{
    /// Prepares a loadablesheet file.
    pub(crate) fn prepare_file(&mut self, file: LoadableFile)
    {
        let _ = self.pending.insert(file.clone());
        self.total_expected_sheets += 1;

        // Make sure this file has a manifest key entry, which indicates it doesn't need to be initialized again
        // if it is present in a manifest or import section.
        self.register_manifest_key(file, None);
    }

    /// Sets the manifest key for a file.
    ///
    /// The `manifest_key` may be `None` if this file is being imported. We use manifest key presence as a proxy
    /// for whether or not a file has been initialized.
    ///
    /// Returns `true` if this is the first time this file's manifest key has been registered. This can be used
    /// to decide whether to start loading transitive imports/manifests.
    pub(crate) fn register_manifest_key(&mut self, file: LoadableFile, manifest_key: Option<Arc<str>>) -> bool
    {
        match self.file_to_manifest_key.entry(file.clone()) {
            Vacant(entry) => {
                entry.insert(manifest_key.clone());

                if let Some(new_key) = manifest_key {
                    if let Some(prev_file) = self.manifest_map.insert(new_key.clone(), file.clone()) {
                        tracing::warn!("replacing file for manifest key {:?} (old: {:?}, new: {:?})",
                            new_key, prev_file, file);
                    }
                }

                true
            }
            Occupied(mut entry) => {
                // Manifest key can be None when a file is imported or loaded via the App extension.
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
                            file.clone(), prev_key.clone(), new_key.clone());
                        let prev = prev_key.clone();
                        *prev_key = new_key.clone();
                        self.manifest_map.remove(&prev);
                    }
                    // Normal case: setting the manifest key.
                    None => {
                        *entry.get_mut() = Some(new_key.clone());
                    }
                }

                if let Some(prev_file) = self.manifest_map.insert(new_key.clone(), file.clone()) {
                    tracing::warn!("replacing file for manifest key {:?} (old: {:?}, new: {:?})",
                        new_key, prev_file, file);
                }

                false
            }
        }
    }

    /// Initializes a file that has been loaded.
    pub(crate) fn initialize_file(&mut self, file: &LoadableFile)
    {
        let _ = self.pending.remove(file);
    }

    /// Gets the loadablesheet's loading progress on startup.
    ///
    /// Returns `(num uninitialized files, num total files)`.
    pub fn loading_progress(&self) -> (usize, usize)
    {
        (self.pending.len(), self.total_expected_sheets)
    }

    /// Gets the number of files waiting to be processed.
    fn num_preprocessed_pending(&self) -> usize
    {
        self.preprocessed.len()
    }

    /// Inserts a preprocessed file for later processing.
    pub(crate) fn add_preprocessed_file(
        &mut self,
        file: LoadableFile,
        mut imports: HashMap<LoadableFile, SmolStr>,
        data: Map<String, Value>,
    )
    {
        // The file should not reference itself.
        if imports.remove(&file).is_some() {
            tracing::warn!("loadable file {:?} tried to import itself", file.as_str());
        }

        // Check that all dependencies are known.
        // - Note: We don't need to check for circular dependencies here. It can be checked after processing files
        //   by seeing if there are any pending files remaining. Once all pending files are loaded, if a file fails
        //   to process that implies it has circular dependencies.
        for import in imports.keys() {
            // Check if pending.
            if self.pending.contains(import) {
                continue;
            }

            // Check if processed.
            if self.processed.contains_key(import) {
                continue;
            }

            // Check if preprocessed.
            if self.preprocessed.iter().any(|p| p.file == *import) {
                continue;
            }

            tracing::error!("ignoring loadable file {:?} that has unregistered import {:?}", file.as_str(), import.as_str());
            return;
        }

        let preprocessed = PreprocessedLoadableFile { file, imports, data };
        self.preprocessed.push(preprocessed);
    }

    /// Converts a preprocessed file to a processed file.
    ///
    /// Assumes all imports are available.
    fn process_sheet(&mut self, preprocessed: PreprocessedLoadableFile, type_registry: &TypeRegistry)
    {
        // Initialize using/constants maps from dependencies.
        // [ shortname : longname ]
        let mut name_shortcuts: HashMap<&'static str, &'static str> = HashMap::default();
        // [ path : [ terminal identifier : constant value ] ]
        let mut constants: HashMap<SmolStr, Map<String, Value>> = HashMap::default();
        // specs collector
        let mut specs = SpecsMap::default();

        for (dependency, alias) in preprocessed.imports.iter() {
            let processed = self.processed.get(dependency).unwrap();

            for (k, v) in processed.using.iter() {
                name_shortcuts.insert(k, v);
            }
            for (k, v) in processed.constants.iter() {
                // Prepend the import alias.
                let path = path_to_constant_string(&[alias.as_str(), k.as_str()]);
                constants.insert(path, v.clone());
            }
            specs.import_specs(dependency, &preprocessed.file, &processed.specs);
        }

        // Prepare to process the file.
        let mut processed = ProcessedLoadableFile::default();

        #[cfg(feature = "hot_reload")]
        {
            processed.imports = preprocessed.imports;
            processed.data = preprocessed.data.clone();
        }

        // Process the file.
        // - This updates the using/constants maps with info extracted from the file.
        parse_loadablesheet_file(
            type_registry,
            self,
            preprocessed.file.clone(),
            preprocessed.data,
            &mut constants,
            &mut specs,
            &mut name_shortcuts,
        );

        // Save final maps.
        processed.using = name_shortcuts;
        processed.constants = constants;
        processed.specs = specs;

        self.processed.insert(preprocessed.file.clone(), processed);

        // Check for already-processed files that need to rebuild since they depend on this file.
        #[cfg(feature = "hot_reload")]
        {
            let needs_rebuild: Vec<LoadableFile> = self
                .processed
                .iter()
                .filter_map(|(file, processed)| {
                    if processed.imports.contains_key(&preprocessed.file) {
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

    /// Converts preprocessed files to processed files.
    ///
    /// Returns `true` if at least one sheet was processed.
    fn process_sheets(&mut self, type_registry: &TypeRegistry) -> bool
    {
        // Loop preprocessed until nothing can be processed.
        let mut num_processed = 0;
        let mut preprocessed = Vec::new();

        while !self.preprocessed.is_empty() {
            let num_already_processed = num_processed;
            preprocessed.clear();
            std::mem::swap(&mut self.preprocessed, &mut preprocessed);

            for preprocessed in preprocessed.drain(..) {
                // Check if dependencies are ready.
                if preprocessed
                    .imports
                    .keys()
                    .any(|i| !self.processed.contains_key(i))
                {
                    self.preprocessed.push(preprocessed);
                    continue;
                }

                self.process_sheet(preprocessed, type_registry);
                num_processed += 1;
            }

            // Exit if no changes made.
            if num_already_processed == num_processed {
                break;
            }
        }

        // Check for circular dependencies.
        if self.pending.is_empty() && !self.preprocessed.is_empty() {
            for preproc in self.preprocessed.drain(..) {
                tracing::error!("discarding loadable file {:?} that failed to resolve imports; it probably has a \
                    dependency cycle; fix the cycle and restart your app", preproc.file.as_str());
            }
        }

        // Clean up memory once all files are loaded and processed.
        #[cfg(not(feature = "hot_reload"))]
        {
            tracing::info!("done loading (enable hot_reload feature if you want to reload files)");
            if self.pending.is_empty() && self.preprocessed.is_empty() {
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
        loadable_ref: &LoadableRef,
        loadable: ReflectedLoadable,
        type_id: TypeId,
        full_type_name: &str,
    ) -> bool
    {
        if !insert_loadable_entry(
            &mut self.command_loadables,
            loadable_ref,
            loadable.clone(),
            type_id,
            full_type_name,
        ) {
            return false;
        }

        let entry = self.commands_need_updates.entry(type_id).or_default();
        entry.push((loadable.clone(), loadable_ref.clone()));

        true
    }

    /// Inserts a loadable at the specified path if its value will change.
    ///
    /// Returns `true` if this method added any pending subscriber updates.
    pub(crate) fn insert_loadable(
        &mut self,
        loadable_ref: &LoadableRef,
        loadable: ReflectedLoadable,
        type_id: TypeId,
        full_type_name: &str,
    ) -> bool
    {
        if !insert_loadable_entry(
            &mut self.loadables,
            loadable_ref,
            loadable.clone(),
            type_id,
            full_type_name,
        ) {
            return false;
        }

        // Identify entites that should update.
        let Some(subscriptions) = self.subscriptions.get(loadable_ref) else { return false };
        if subscriptions.is_empty() {
            return false;
        }
        let entry = self.needs_updates.entry(type_id).or_default();
        entry.push((loadable, loadable_ref.clone(), subscriptions.clone()));

        true
    }

    /// Adds an entity to the tracking context.
    ///
    /// Schedules callbacks that will run to handle pending updates for the entity.
    pub(crate) fn track_entity(
        &mut self,
        entity: Entity,
        mut loadable_ref: LoadableRef,
        setter: ContextSetter,
        c: &mut Commands,
        callbacks: &LoaderCallbacks,
    )
    {
        // Replace manifest key in the requested loadable.
        self.swap_manifest_key_for_file(&mut loadable_ref.file);

        // Add to subscriptions.
        let subscription = RefSubscription { entity, setter };
        self.subscriptions
            .entry(loadable_ref.clone())
            .or_default()
            .push(subscription);
        self.subscriptions_rev
            .entry(entity)
            .or_default()
            .push(loadable_ref.clone());

        // Get already-loaded values that the entity is subscribed to.
        let Some(loadables) = self.loadables.get(&loadable_ref) else { return };

        // Schedule updates for each loadable so they will be applied to the entity.
        for loadable in loadables.iter() {
            let type_id = loadable.type_id;
            self.needs_updates.entry(type_id).or_default().push((
                loadable.loadable.clone(),
                loadable_ref.clone(),
                SmallVec::from_elem(subscription, 1),
            ));

            let Some(syscommand) = callbacks.get(type_id) else {
                tracing::warn!("found loadable at {:?} that wasn't registered as a loadable loadable", loadable_ref);
                continue;
            };

            c.add(syscommand);
        }

        // Notify the entity that some of its loadables have loaded.
        if !loadables.is_empty() {
            c.react().entity_event(entity, Loaded);
        }
    }

    /// Swaps a manifest key for a file reference.
    fn swap_manifest_key_for_file(&self, maybe_key: &mut LoadableFile)
    {
        let LoadableFile::ManifestKey(key) = maybe_key else { return };
        let Some(file_ref) = self.manifest_map.get(key) else {
            tracing::error!("tried accessing manifest key {:?} but no file was found", key);
            return;
        };
        *maybe_key = file_ref.clone();
    }

    /// Cleans up despawned entities.
    fn remove_entity(&mut self, dead_entity: Entity)
    {
        let Some(loadable_refs) = self.subscriptions_rev.remove(&dead_entity) else { return };
        for loadable_ref in loadable_refs {
            let Some(subscribed) = self.subscriptions.get_mut(&loadable_ref) else { continue };
            let Some(dead) = subscribed.iter().position(|s| s.entity == dead_entity) else { continue };
            subscribed.swap_remove(dead);
        }
    }

    /// Cleans up pending updates that failed to be processed.
    fn cleanup_pending_updates(&mut self)
    {
        if !self.commands_need_updates.is_empty() {
            warn_once!("The loadable sheet contains pending updates for command types that weren't registered. This warning \
                only prints once.");
        }
        if !self.needs_updates.is_empty() {
            // Note: This can technically print spuriously if the user spawns loaded entities in Last and doesn't
            // call `apply_deferred` before the cleanup system runs.
            warn_once!("The loadable sheet contains pending updates for types that weren't registered. This warning only \
                prints once, and may print spuriously if you spawn loaded entities in Last.");
        }
        self.commands_need_updates.clear();
        self.needs_updates.clear();
    }

    /// Applies loadables extracted from `#commands` sections.
    pub(crate) fn apply_commands<T: Loadable>(
        &mut self,
        mut callback: impl FnMut(&LoadableRef, &ReflectedLoadable),
    )
    {
        let Some(mut commands_need_updates) = self.commands_need_updates.remove(&TypeId::of::<T>()) else {
            return;
        };

        for (loadable, loadable_ref) in commands_need_updates.drain(..) {
            (callback)(&loadable_ref, &loadable);
        }
    }

    /// Updates entities that subscribed to `T` found at recently-updated loadable paths.
    pub(crate) fn update_loadables<T: Loadable>(
        &mut self,
        mut callback: impl FnMut(Entity, ContextSetter, &LoadableRef, &ReflectedLoadable),
    )
    {
        let Some(mut needs_updates) = self.needs_updates.remove(&TypeId::of::<T>()) else { return };

        for (loadable, loadable_ref, mut subscriptions) in needs_updates.drain(..) {
            for subscription in subscriptions.drain(..) {
                (callback)(subscription.entity, subscription.setter, &loadable_ref, &loadable);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System set in [`First`] where files are processed.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct FileProcessingSet;

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that enables loading.
pub(crate) struct LoadableSheetPlugin;

impl Plugin for LoadableSheetPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_react_resource::<LoadableSheet>()
            .add_systems(
                First,
                (
                    preprocess_loadable_files,
                    process_loadable_files.run_if(|s: ReactRes<LoadableSheet>| s.num_preprocessed_pending() > 0),
                )
                    .chain()
                    .in_set(FileProcessingSet),
            )
            .add_systems(Last, cleanup_loadablesheet);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use crate::cob::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct PreprocessedSceneFile
{
    /// This file.
    file: CobFile,
    /// Imports for detecting when a re-load is required.
    /// - Can include both manifest keys and file paths.
    imports: HashMap<ManifestKey, CobImportAlias>,
    /// Data cached for re-loading when dependencies are reloaded.
    data: Cob,
    /// File hash for editor use.
    #[cfg(feature = "editor")]
    hash: crate::editor::CobFileHash,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
struct ProcessedSceneFile
{
    /// Resolver info cached for use by dependents.
    resolver: CobResolver,
    /// Imports for detecting when a re-load is required.
    #[cfg(feature = "hot_reload")]
    imports: HashMap<ManifestKey, CobImportAlias>,
    /// Un-extracted data cached for re-loading when imports are reloaded.
    #[cfg(feature = "hot_reload")]
    data: Cob,
    /// File hash for editor use.
    #[cfg(feature = "editor")]
    hash: crate::editor::CobFileHash,
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that manages content extracted from cobweb asset files (`.cob` files).
///
/// Can be used to load scenes with [`SpawnSceneExt::spawn_scene`], or load individual scene nodes with
/// [`NodeBuilderExt::load`].
///
/// Note that command loadables in cob files are automatically applied to the world. Commands are globally
/// ordered by:
/// 1) Files manually registered to an app with [`LoadedCobAssetFilesAppExt::load`].
/// 2) Commands in a file's `#commands` section(s).
/// 3) Files loaded recursively via COB manifests. Commands in file A will be applied before any commands in
/// manifest files in file A.
#[derive(Resource, Default, Debug)]
pub(crate) struct CobAssetCache
{
    /// Tracks which files have not initialized yet.
    pending: HashSet<CobFile>,
    /// Tracks the total number of files that should load.
    ///
    /// Used for progress tracking on initial load.
    total_expected_sheets: usize,

    /// Tracks manifest data.
    /// - Inside an arc/mutex so other buffers can also use it.
    manifest_map: Arc<Mutex<ManifestMap>>,
    /// Tracks which files have been assigned manifest keys.
    file_to_manifest_key: HashMap<CobFile, Option<ManifestKey>>,

    /// Tracks pre-processed files.
    preprocessed: Vec<PreprocessedSceneFile>,
    /// Tracks all unique pre-processes files. Used for state file cleanup during hot reloading.
    #[cfg(feature = "hot_reload")]
    preprocessed_set: HashSet<CobFile>,

    /// Records processed files.
    processed: HashMap<CobFile, ProcessedSceneFile>,

    /// Tracks files that have been processed but not scene-extracted.
    #[cfg(feature = "hot_reload")]
    needs_scene_extraction: HashMap<CobFile, Cob>,
}

impl CobAssetCache
{
    pub(super) fn new(manifest_map: Arc<Mutex<ManifestMap>>) -> Self
    {
        Self { manifest_map, ..default() }
    }

    pub(crate) fn manifest_map_clone(&self) -> Arc<Mutex<ManifestMap>>
    {
        self.manifest_map.clone()
    }

    fn manifest_map(&mut self) -> MutexGuard<ManifestMap>
    {
        self.manifest_map.lock().unwrap()
    }

    /// Gets the CobAssetCache's loading progress on startup.
    ///
    /// Returns `(num uninitialized files, num total files)`.
    ///
    /// Does not include files recursively loaded via manifests.
    fn loading_progress(&self) -> (usize, usize)
    {
        (self.pending.len(), self.total_expected_sheets)
    }

    /// Gets the number of files waiting to be processed.
    pub(super) fn num_preprocessed_pending(&self) -> usize
    {
        self.preprocessed.len()
    }

    /// Returns `(hash, data, is_processed)`.
    ///
    /// Used for patching file data on editor changes.
    #[cfg(feature = "editor")]
    pub(crate) fn get_file_info_mut(
        &mut self,
        file: &CobFile,
    ) -> Option<(&mut crate::editor::CobFileHash, &mut Cob, bool)>
    {
        // Check processed first as most likely.
        if let Some(processed) = self.processed.get_mut(file) {
            return Some((&mut processed.hash, &mut processed.data, true));
        }

        if self.preprocessed_set.contains(file) {
            let preprocessed = self.preprocessed.iter_mut().find(|p| p.file == *file)?;
            return Some((&mut preprocessed.hash, &mut preprocessed.data, false));
        }

        None
    }

    /// Prepares a cobweb asset file.
    pub(crate) fn prepare_file(&mut self, file: CobFile)
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
    pub(crate) fn register_manifest_key(&mut self, file: CobFile, manifest_key: Option<ManifestKey>) -> bool
    {
        match self.file_to_manifest_key.entry(file.clone()) {
            Vacant(entry) => {
                entry.insert(manifest_key.clone());

                if let Some(new_key) = manifest_key {
                    if let Some(prev_file) = self.manifest_map().insert(new_key.clone(), file.clone()) {
                        if file != prev_file {
                            tracing::warn!("replacing file for manifest key {:?} (old: {:?}, new: {:?})",
                                new_key, prev_file, file);
                        }
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
                        if *prev_key != new_key {
                            tracing::warn!("changing manifest key for {:?} (old: {:?}, new: {:?})",
                                file, prev_key, new_key);
                            let prev = prev_key.clone();
                            *prev_key = new_key.clone();
                            self.manifest_map().remove(&prev);
                        }
                    }
                    // Normal case: setting the manifest key.
                    None => {
                        *entry.get_mut() = Some(new_key.clone());
                    }
                }

                // Note: self.file_to_manifest_key may contain 'dead' entries if a file is renamed and then
                // the manifest that pull it in is updated. The old file-name will linger in
                // self.file_to_manifest_key, but the manifest map will be updated to point to the
                // new file. If the file is renamed back to its original name, then the manifest
                // map will be updated again to point to the old file name.
                if let Some(prev_file) = self.manifest_map().insert(new_key.clone(), file.clone()) {
                    if file != prev_file {
                        tracing::warn!("replacing file for manifest key {:?} (old: {:?}, new: {:?})",
                            new_key, prev_file, file);
                    }
                }

                false
            }
        }
    }

    /// Initializes a file that has been loaded.
    pub(crate) fn initialize_file(&mut self, file: &CobFile)
    {
        let _ = self.pending.remove(file);
    }

    /// Inserts a preprocessed file for later processing.
    pub(crate) fn add_preprocessed_file(
        &mut self,
        file: CobFile,
        imports: HashMap<ManifestKey, CobImportAlias>,
        data: Cob,
        #[cfg(feature = "editor")] hash: crate::editor::CobFileHash,
    )
    {
        // Remove if already processed.
        // - This is avoids duplicate re-processing in this kind of situation:
        //   - file A has manifest and import for file B
        //   - file B is renamed to B2, and file A is updated with the new name
        //   - file A reloads with the new name, triggering a new load for file B2
        //   - file A will be added here then get stuck pending since it imports B2
        //   - when B2 loads, if file A is still in self.processed, then it will push the stale file A data onto
        //     the preprocessed stack here
        //   - the stale file A data will be inserted after the fresh file A data, causing it to be processed last,
        //     which is a bug
        #[cfg(feature = "hot_reload")]
        {
            self.processed.remove(&file);
        }

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

            tracing::error!("ignoring file {} that points to untracked file {}; this is a bug",
                file.as_str(), import.as_str());
            return;
        }

        // During hot reload it's possible for a file to be sent to self.preprocessed multiple times before
        // any processing occurs. For robustness we clean up stale updates here.
        #[cfg(feature = "hot_reload")]
        {
            if !self.preprocessed_set.insert(file.clone()) {
                if let Some(prev) = self.preprocessed.iter().position(|p| p.file == file) {
                    self.preprocessed.remove(prev);
                }
            }
        }

        let preprocessed = PreprocessedSceneFile {
            file,
            imports,
            data,
            #[cfg(feature = "editor")]
            hash,
        };
        self.preprocessed.push(preprocessed);
    }

    /// Converts a preprocessed file to a processed file.
    ///
    /// Assumes all imports are available.
    fn process_cobweb_asset_file(
        &mut self,
        mut preprocessed: PreprocessedSceneFile,
        type_registry: &TypeRegistry,
        loadables: &LoadableRegistry,
        _c: &mut Commands,
        commands_buffer: &mut CommandsBuffer,
        _scene_buffer: &mut SceneBuffer,
        _scene_loader: &mut SceneBuilder,
    )
    {
        // Initialize resolver from dependencies.
        let mut resolver = CobResolver::default();

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

            resolver.append(alias.as_str(), &processed.resolver);
        }

        // Prepare to process the file.
        let mut processed = ProcessedSceneFile::default();

        #[cfg(feature = "hot_reload")]
        {
            processed.imports = preprocessed.imports;
            // Data must be cloned before extraction, because extraction will modify the value in-place in order
            // to process definitions. Definitions always need to be re-processed when re-extracting a file.
            processed.data = preprocessed.data.clone();
        }

        #[cfg(feature = "editor")]
        {
            processed.hash = preprocessed.hash;
        }

        // Process the file.
        // - This updates the constants/specs maps with info extracted from the file.
        extract_cob_importables(preprocessed.file.clone(), &mut preprocessed.data, &mut resolver);

        extract_cob_commands(
            type_registry,
            commands_buffer,
            preprocessed.file.clone(),
            &mut preprocessed.data,
            loadables,
            &resolver,
        );

        #[cfg(not(feature = "hot_reload"))]
        {
            // Extract scenes immediately.
            extract_cob_scenes(
                type_registry,
                _c,
                _scene_buffer,
                _scene_loader,
                preprocessed.file.clone(),
                preprocessed.data,
                loadables,
                &mut resolver,
            );
        }
        #[cfg(feature = "hot_reload")]
        {
            // Defer scene extraction until it can be synchronized with loading entities.
            self.needs_scene_extraction
                .insert(preprocessed.file.clone(), preprocessed.data);
        }

        // Save final maps.
        processed.resolver = resolver;

        // Set fully processed
        self.processed.insert(preprocessed.file.clone(), processed);

        // Check for already-processed files that need to rebuild since they depend on this file.
        // TODO: It may be more efficient to cache a map of [file : importers]. Below will be quite expensive
        // if there are many files.
        #[cfg(feature = "hot_reload")]
        {
            if let Some(manifest_key) = self
                .file_to_manifest_key
                .get(&preprocessed.file)
                .cloned()
                .flatten()
            {
                let needs_rebuild: Vec<CobFile> = self
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
                    // Add via API to check for recursive dependencies.
                    commands_buffer.prep_commands_refresh(needs_rebuild.clone());
                    let processed = self.processed.remove(&needs_rebuild).unwrap();
                    self.add_preprocessed_file(
                        needs_rebuild,
                        processed.imports,
                        processed.data,
                        #[cfg(feature = "editor")]
                        processed.hash,
                    );
                }
            }
        }
    }

    /// Converts preprocessed files to processed files.
    ///
    /// Returns `true` if at least one file was processed.
    pub(super) fn process_cobweb_asset_files(
        &mut self,
        type_registry: &TypeRegistry,
        loadables: &LoadableRegistry,
        c: &mut Commands,
        commands_buffer: &mut CommandsBuffer,
        scene_buffer: &mut SceneBuffer,
        scene_loader: &mut SceneBuilder,
    ) -> bool
    {
        // Loop preprocessed until nothing can be processed.
        let mut num_processed = 0;
        let mut preprocessed = Vec::new();

        while !self.preprocessed.is_empty() {
            let num_already_processed = num_processed;
            preprocessed.clear();
            std::mem::swap(&mut self.preprocessed, &mut preprocessed);
            #[cfg(feature = "hot_reload")]
            self.preprocessed_set.clear();

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
                        #[cfg(feature = "hot_reload")]
                        {
                            let is_new = self.preprocessed_set.insert(preprocessed.file.clone());
                            debug_assert!(is_new);
                        }
                        self.preprocessed.push(preprocessed);
                        continue;
                    }
                }

                self.process_cobweb_asset_file(
                    preprocessed,
                    type_registry,
                    loadables,
                    c,
                    commands_buffer,
                    scene_buffer,
                    scene_loader,
                );
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
                tracing::error!("discarding COB file {:?} that failed to resolve imports; it either has a \
                    dependency cycle or tries to import unknown manifest keys; note that builtin colors/widgets/etc. \
                    require the corresponding feature to be enabled", preproc.file.as_str());
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

    #[cfg(feature = "hot_reload")]
    pub(crate) fn handle_pending_scene_extraction(
        &mut self,
        type_registry: &TypeRegistry,
        loadables: &LoadableRegistry,
        c: &mut Commands,
        scene_buffer: &mut SceneBuffer,
        scene_loader: &mut SceneBuilder,
        #[cfg(feature = "editor")] editor: &mut crate::editor::CobEditor,
    )
    {
        // Note: We assume it doesn't matter what file order scenes are extracted in.
        for (file, data) in self.needs_scene_extraction.drain() {
            let Some(processed) = self.processed.get_mut(&file) else { continue };

            extract_cob_scenes(
                type_registry,
                c,
                scene_buffer,
                scene_loader,
                file,
                data,
                loadables,
                &mut processed.resolver,
            );

            // Pass to editor.
            // - We wait until after scene extraction to make sure the editor view synchronizes with the
            // file's cached state.
            #[cfg(feature = "editor")]
            {
                editor.add_processed(c, processed.hash, &processed.data);
            }
        }
    }
}

impl AssetLoadProgress for CobAssetCache
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

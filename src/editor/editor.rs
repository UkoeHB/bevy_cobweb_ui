use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use bevy::asset::io::file::FileAssetReader;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct CobFileData
{
    pub(super) last_save_hash: CobFileHash,
    /// Data for the file. Defs in this data are *not* resolved.
    pub(super) data: Cob,
}

impl CobFileData
{
    pub(super) fn is_editable(&self) -> bool
    {
        // Temp hack to qualify files with non-default asset source as uneditable, since we don't have a good
        // way to save them yet.
        self.data.file.as_str().find("://").is_none()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Files cannot be saved on `wasm32` or `android` targets.
#[derive(Resource)]
pub(crate) struct CobEditor
{
    /// All files tracked by the editor.
    files: HashMap<CobFile, CobFileData>,

    /// Files waiting to be saved.
    unsaved: HashSet<CobFile>,

    /// Asset directory location.
    ///
    /// If there is no path then files cannot be saved.
    asset_dir: Option<PathBuf>,
}

impl CobEditor
{
    fn new(asset_path: String) -> Self
    {
        let asset_dir = {
            #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
            {
                Some(FileAssetReader::new(asset_path).root_path().clone())
            }

            #[cfg(any(target_arch = "wasm32", target_os = "android"))]
            None
        };

        Self {
            files: HashMap::default(),
            unsaved: HashSet::default(),
            asset_dir,
        }
    }

    pub(super) fn any_unsaved(&self) -> bool
    {
        !self.unsaved.is_empty()
    }

    pub(super) fn iter_files(&self) -> impl Iterator<Item = (&CobFile, &CobFileData)> + '_
    {
        self.files.iter()
    }

    pub(super) fn get_file(&self, file: &CobFile) -> Option<&CobFileData>
    {
        self.files.get(file)
    }

    pub(super) fn get_file_mut(&mut self, file: &CobFile) -> Option<&mut CobFileData>
    {
        self.files.get_mut(file)
    }

    /// Adds a file that was just processed by the CobwebAssetCache.
    pub(crate) fn add_processed(&mut self, c: &mut Commands, hash: CobFileHash, data: &Cob)
    {
        let Some(existing) = self.files.get_mut(&data.file) else {
            self.files.insert(
                data.file.clone(),
                CobFileData { last_save_hash: hash, data: data.clone() },
            );
            c.react()
                .broadcast(EditorNewFile { file: data.file.clone() });
            return;
        };

        // Always broadcast this in case the editor view needs to be respawned.
        c.react()
            .broadcast(EditorFileExternalChange { file: data.file.clone() });

        // Remove from unsaved.
        let removed = self.unsaved.remove(&data.file);

        // Avoid cloning the data if it's the same data unless we have unsaved changes that need to be discarded.
        if !removed && existing.last_save_hash == hash {
            return;
        }

        if removed {
            tracing::warn!("discarding unsaved changes for file {:?}; changes were overwritten by new \
                file data that was hot-reloaded", data.file);
            c.react()
                .broadcast(EditorFileSaved { file: data.file.clone(), hash });
        }

        // Save new data.
        existing.last_save_hash = hash;
        existing.data = data.clone();
    }

    pub(super) fn mark_unsaved(&mut self, c: &mut Commands, file: CobFile)
    {
        c.react()
            .broadcast(EditorFileUnsaved { file: file.clone() });
        self.unsaved.insert(file);
    }

    /// Saves currently-unsaved files.
    // TODO: currently blocks the main loop, maybe pass this off to the CPU thread pool? problem is how to
    // correctly synchronize with the editor; also need to be careful about not contesting the scratch file name
    pub(super) fn save(&mut self, c: &mut Commands, cob_cache: &mut CobAssetCache, registry: &CobHashRegistry)
    {
        let Some(asset_dir) = &self.asset_dir else {
            if !self.unsaved.is_empty() {
                tracing::error!("failed saving files {:?}, no asset directory is available (this is a bug)",
                    self.unsaved);
                self.unsaved.clear();
            }
            return;
        };

        for unsaved in self.unsaved.drain() {
            let Some(file_data) = self.files.get_mut(&unsaved) else {
                tracing::error!("file {:?} is missing on save (this is a bug)", unsaved);
                continue;
            };

            // Collect bytes.
            let mut buff = Vec::<u8>::default();
            let mut serializer = DefaultRawSerializer::new(&mut buff);
            file_data.data.write_to(&mut serializer).unwrap();

            // Compute hash.
            let hash = CobFileHash::new(&buff);

            // Notify listeners.
            c.react()
                .broadcast(EditorFileSaved { file: unsaved.clone(), hash });

            // If hash didn't change, no need to save the file since the 'unsaved' status is spurious.
            if hash == file_data.last_save_hash {
                continue;
            }

            // Update the asset cache.
            // - If the last_save_hash matches, then we know all editor mutations have been applied manually to the
            //   current
            // file data, so we can safely 'hack' in a new hash.
            if let Some((file_hash, _, _)) = cob_cache.get_file_info_mut(&unsaved) {
                // This can fail if a hot-reloaded file is being processed in the backend when a save occurs.
                if *file_hash == file_data.last_save_hash {
                    *file_hash = hash;
                }
            }

            // Update the hash registry.
            registry.set_file_for_save(unsaved.as_str(), file_data.last_save_hash, hash);

            // Save the file.
            file_data.last_save_hash = hash;

            #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
            {
                use std::io::Write;

                // Write to scratch file.
                let scratch = asset_dir.join(".__cob_editor_scratch");
                {
                    let mut file = match std::fs::File::create(&scratch) {
                        Ok(file) => file,
                        Err(err) => {
                            tracing::warn!("saving {unsaved:?} failed unexpectedly while opening scratch file: {err:?}");
                            continue;
                        }
                    };
                    // Set length to zero in case a previous use of the scratch file failed halfway.
                    if let Err(err) = file.set_len(0) {
                        tracing::warn!("saving {unsaved:?} failed unexpectedly while truncating scratch file to zero \
                            length: {err:?}");
                        continue;
                    }
                    if let Err(err) = file.write_all(&buff) {
                        tracing::warn!("saving {unsaved:?} failed unexpectedly while writing the buffer to \
                            scratch: {err:?}");
                        continue;
                    }
                    if let Err(err) = file.sync_all() {
                        tracing::warn!("saving {unsaved:?} failed unexpectedly while syncing: {err:?}");
                        continue;
                    }
                }

                // Safely replace the target file with the scratch.
                if let Err(err) = std::fs::rename(&scratch, asset_dir.join(unsaved.as_str())) {
                    tracing::warn!("saving {unsaved:?} failed unexpectedly while renaming scratch: {err:?}");
                }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct CobEditorImplPlugin;

impl Plugin for CobEditorImplPlugin
{
    fn build(&self, app: &mut App)
    {
        let added = app.get_added_plugins::<AssetPlugin>();
        let asset_plugin = added
            .get(0)
            .expect("AssetPlugin should be added before CobwebUiPlugin");
        app.insert_resource(CobEditor::new(asset_plugin.file_path.clone()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

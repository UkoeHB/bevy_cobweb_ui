use std::sync::Arc;

use bevy::prelude::*;
use serde::de::DeserializeSeed;

use super::*;
use crate::cob::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct SaveEditor;

impl Command for SaveEditor
{
    fn apply(self, world: &mut World)
    {
        world.resource_scope::<CobEditor, ()>(|world: &mut World, mut editor: Mut<CobEditor>| {
            world.resource_scope::<CobAssetCache, ()>(|world: &mut World, mut cob_cache: Mut<CobAssetCache>| {
                world.resource_scope::<CobHashRegistry, ()>(
                    |world: &mut World, registry: Mut<CobHashRegistry>| {
                        let mut c = world.commands();
                        editor.save(&mut c, &mut cob_cache, &registry);
                    },
                );
            });
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Command for patching a value in the editor (a command or scene node loadable).
#[derive(Debug)]
pub struct SubmitPatch
{
    pub editor_ref: CobEditorRef,
    pub value: Box<dyn PartialReflect + 'static>,
}

impl Command for SubmitPatch
{
    fn apply(self, world: &mut World)
    {
        let editor_ref = self.editor_ref;

        // Exit if source widget is dead.
        if editor_ref.death_signal.is_dead() {
            tracing::warn!("ignoring editor patch for {} in {:?}; originating widget is marked 'dead'",
                editor_ref.loadable_name, editor_ref.scene_ref);
            return;
        }

        world.resource_scope::<CobEditor, ()>(|world: &mut World, mut editor: Mut<CobEditor>| {
            // Get the file id.
            let SceneFile::File(file) = editor_ref.scene_ref.file.clone() else {
                tracing::error!("ignoring editor patch for {} in {:?}; scene ref unexpectedly has a manifest key instead \
                    of file",
                    editor_ref.loadable_name, editor_ref.scene_ref);
                return;
            };

            // Look up the targeted file.
            let Some(file_data) = editor.get_file_mut(&file) else {
                tracing::warn!("ignoring editor patch for {} in {:?}; file is unknown",
                    editor_ref.loadable_name, editor_ref.scene_ref);
                return;
            };

            // Exit if file hash doesn't match.
            if file_data.last_save_hash != editor_ref.file_hash {
                tracing::warn!("ignoring editor patch for {} in {:?}; widget has a stale editor reference",
                    editor_ref.loadable_name, editor_ref.scene_ref);
                return;
            }

            // Look up the targeted loadable.
            let Some(targeted) = get_targeted(&mut file_data.data, &editor_ref) else {
                tracing::warn!("ignoring editor patch for {} in {:?}; targeted loadable not found",
                    editor_ref.loadable_name, editor_ref.scene_ref);
                return;
            };

            // Prep deserializer for targeted loadable.
            let loadables = world.resource::<LoadableRegistry>();
            let type_registry = world.resource::<AppTypeRegistry>().read();
            let Some((deserializer, type_id, longname, _)) = get_deserializer(&type_registry, editor_ref.loadable_name, &loadables) else {
                tracing::warn!("ignoring editor patch for {} in {:?}; failed looking up loadable in type registry",
                    editor_ref.loadable_name, editor_ref.scene_ref);
                return;
            };

            // Get a PartialReflect for the targeted loadable.
            let mut reflected_target = match deserializer.deserialize(&*targeted) {
                Ok(r) => r,
                Err(err) => {
                    // This can occur if `targeted` is not fully resolved.
                    tracing::warn!("ignoring editor patch for {} in {:?}; original value failed to deserialize: {err:?}",
                        editor_ref.loadable_name, editor_ref.scene_ref);
                    return;
                }
            };

            // Patch in the given value.
            if let Err(err) = editor_ref.structure_path.try_patch_value(&mut reflected_target, self.value) {
                match err {
                    Some(err) => {
                        tracing::warn!("ignoring editor patch for {} in {:?}; failed applying patch with path {:?}; \
                            error: {err:?}",
                            editor_ref.loadable_name, editor_ref.scene_ref, editor_ref.structure_path);
                    }
                    None => {
                        tracing::error!("ignoring editor patch for {} in {:?}; invalid path {:?}",
                            editor_ref.loadable_name, editor_ref.scene_ref, editor_ref.structure_path);
                    }
                }
                return;
            }

            // Get a new CobLoadable value from the patched reflected target.
            let mut new_loadable = match CobLoadable::extract_partial_reflect(reflected_target.as_ref(), &type_registry) {
                Ok(l) => l,
                Err(err) => {
                    tracing::warn!("ignoring editor patch for {} in {:?}; failed extracting patched value: {err:?}",
                        editor_ref.loadable_name, editor_ref.scene_ref);
                    return;
                }
            };
            std::mem::drop(type_registry);

            // Recover fill from the previous value.
            // TODO: this will needlessly allocate new fills, maybe recover_fill should move values from targeted? or
            // add recover_fill_owned?
            new_loadable.recover_fill(targeted);

            // Replace the old value.
            *targeted = new_loadable.clone();

            // Mark the file as unsaved in the editor.
            let mut commands = world.commands();
            editor.mark_unsaved(&mut commands, file.clone());

            // Try to repair cob asset cache's preprocessed or processed file.
            let mut cob_cache = world.resource_mut::<CobAssetCache>();
            if let Some((cache_hash, cache_data, is_processed)) = cob_cache.get_file_info_mut(&file) {
                // Check file hash.
                if *cache_hash != editor_ref.file_hash {
                    tracing::warn!("failed propagating loadable patch for {} in {:?} to backend; target file \
                        is currently being re-processed, likely due to a hot-reloaded change; the current \
                        editor view of the file will likely be overwritten soon",
                        editor_ref.loadable_name, editor_ref.scene_ref);
                    return;
                }

                // Get targeted value.
                let Some(targeted) = get_targeted(cache_data, &editor_ref) else {
                    tracing::error!("failed propagating loadable patch for {} in {:?} to backend; targeted loadable \
                        not found in target file (processed={is_processed}) (this is a bug)",
                        editor_ref.loadable_name, editor_ref.scene_ref);
                    return;
                };

                // Note: targeted value does not need to be resolved, we currently only support fully resolved values.

                // Set targeted value.
                *targeted = new_loadable;

                // Pass value to the app for use.
                // - We only do this for processed files since preprocessed files will automatically propagate
                // values when they are processed.
                if is_processed {
                    let erased = ErasedLoadable{ type_id, loadable: ReflectedLoadable::Value(Arc::new(reflected_target)) };

                    match editor_ref.is_command() {
                        true => {
                            let mut commands_buffer = world.resource_mut::<CommandsBuffer>();
                            commands_buffer.patch_command(file, longname, erased);
                        }
                        false => {
                            let mut scenes_buffer = world.resource_mut::<SceneBuffer>();
                            // TODO: if constants become editable, it may not be safe to naively stick new values
                            // at the end of the update queue here
                            // - The main thing is editor changes should be 'transactional' and all effects move
                            // through as a single block of changes that synchronize with other 'transactional'
                            // changes such as hot-reloading a file.
                            scenes_buffer.insert_loadable(
                                &editor_ref.scene_ref,
                                None, // Insert in-place.
                                erased.loadable,
                                erased.type_id,
                                longname,
                            );
                        }
                    }
                }
            } else {
                tracing::error!("patch for loadable {} in {:?} could not be propagated to the app because the \
                    file is missing in the backend (this is a bug)",
                    editor_ref.loadable_name, editor_ref.scene_ref);
            }
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: add/remove struct/enum-struct field
// - requires re-spawning widgets
// - requires patching the CobLoadable directly, since fields of reflected values cannot be inserted/removed easily

//-------------------------------------------------------------------------------------------------------------------

// TODO: set enum variant
// - requires re-spawning widgets
// - might require special handling for Option<T>

//-------------------------------------------------------------------------------------------------------------------

// TODO: add/remove list entry
// - requires re-spawning widgets

//-------------------------------------------------------------------------------------------------------------------

// TODO: add/remove map entry
// - requires re-spawning widgets

//-------------------------------------------------------------------------------------------------------------------

// TODO: add/remove/move scene node
// - requires re-spawning widgets

//-------------------------------------------------------------------------------------------------------------------

// TODO: add/remove/move loadable
// - requires re-spawning widgets

//-------------------------------------------------------------------------------------------------------------------

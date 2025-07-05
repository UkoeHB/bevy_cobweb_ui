use std::collections::HashMap;

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use super::*;
use crate::cob::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Preprocesses a cobweb asset file and adds it to [`CobAssetCache`] for processing.
///
/// Only the manifest and imports sections of the file are extracted here.
pub(crate) fn preprocess_cob_file(
    asset_server: &AssetServer,
    cob_files: &mut LoadedCobAssetFiles,
    cob_cache: &mut CobAssetCache,
    commands_buffer: &mut CommandsBuffer,
    data: Cob,
    #[cfg(feature = "editor")] hash: crate::editor::CobFileHash,
)
{
    cob_cache.initialize_file(&data.file);

    // Extract manifest and import sections.
    let mut manifest = vec![];
    let mut imports: HashMap<ManifestKey, CobImportAlias> = HashMap::default();

    for section in data.sections.iter() {
        match section {
            CobSection::Manifest(section) => extract_manifest_section(&data.file, section, &mut manifest),
            CobSection::Import(section) => extract_import_section(section, &mut imports),
            _ => (),
        }
    }

    // Register manifest keys.
    let mut descendants = vec![];
    for (other_file, manifest_key) in manifest {
        // Cache file for commands buffer.
        // - We skip any self reference in the manifest.
        if other_file != data.file {
            descendants.push(other_file.clone());
        }

        // Continue if this file has been registered before.
        if !cob_cache.register_manifest_key(other_file.clone(), Some(manifest_key)) {
            continue;
        }

        // Load this manifest entry.
        cob_files.start_loading(other_file, cob_cache, asset_server);
    }

    // Update this file in the commands buffer.
    commands_buffer.set_file_descendants(data.file.clone(), descendants);

    // Save this file for processing once its import dependencies are ready.
    cob_cache.add_preprocessed_file(
        data.file.clone(),
        imports,
        data,
        #[cfg(feature = "editor")]
        hash,
    );
}

//-------------------------------------------------------------------------------------------------------------------

/// Extracts importable values (defs sections).
///
/// This is semi-destructive, because definitions will be removed and inserted to appropriate maps/buffers.
pub(crate) fn extract_cob_importables(file: CobFile, data: &mut Cob, resolver: &mut CobResolver)
{
    tracing::info!("extracting COB file {:?}", file.as_str());

    resolver.start_new_file();

    for section in data.sections.iter_mut() {
        match section {
            CobSection::Defs(section) => extract_defs_section(&file, section, resolver),
            _ => (),
        }
    }

    resolver.end_new_file();
}

//-------------------------------------------------------------------------------------------------------------------

/// Extracts commands from a `Cob`. Commands are updated in-place when resolving defs.
pub(crate) fn extract_cob_commands(
    type_registry: &TypeRegistry,
    commands_buffer: &mut CommandsBuffer,
    file: CobFile,
    data: &mut Cob,
    loadables: &LoadableRegistry,
    resolver: &CobResolver,
)
{
    let mut commands = vec![];

    for section in data.sections.iter_mut() {
        match section {
            CobSection::Commands(section) => {
                extract_commands_section(type_registry, &mut commands, &file, section, loadables, resolver)
            }
            _ => (),
        }
    }

    commands_buffer.set_file_commands(file, commands);
}

//-------------------------------------------------------------------------------------------------------------------

/// Extracts scenes from a `Cob`. Scene nodes are updated in-place when resolving defs.
pub(crate) fn extract_cob_scenes(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    scene_buffer: &mut SceneBuffer,
    scene_builder: &mut SceneBuilder,
    file: CobFile,
    mut data: Cob,
    loadables: &LoadableRegistry,
    resolver: &mut CobResolver,
)
{
    for section in data.sections.iter_mut() {
        match section {
            CobSection::Scenes(section) => extract_scenes(
                type_registry,
                c,
                scene_buffer,
                scene_builder,
                &file,
                section,
                loadables,
                resolver,
            ),
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

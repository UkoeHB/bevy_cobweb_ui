use std::collections::HashMap;

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Preprocesses a cobweb asset file and adds it to [`CobwebAssetCache`] for processing.
///
/// Only the manifest and imports sections of the file are extracted here.
pub(crate) fn preprocess_caf_file(
    manifest: &mut HashMap<CafFile, ManifestKey>,
    asset_server: &AssetServer,
    caf_files: &mut LoadedCobwebAssetFiles,
    caf_cache: &mut CobwebAssetCache,
    data: Caf,
)
{
    caf_cache.initialize_file(&data.file);

    // Extract manifest and import sections.
    manifest.clear();
    let mut imports: HashMap<ManifestKey, CafImportAlias> = HashMap::default();

    for section in data.sections.iter() {
        match section {
            CafSection::Manifest(section) => extract_manifest_section(&data.file, section, manifest),
            CafSection::Import(section) => extract_import_section(section, &mut imports),
            _ => (),
        }
    }

    // Register manifest keys.
    for (other_file, manifest_key) in manifest.drain() {
        // Continue if this file has been registered before.
        if !caf_cache.register_manifest_key(other_file.clone(), Some(manifest_key)) {
            continue;
        }

        // Load this manifest entry.
        caf_files.start_loading(other_file, caf_cache, asset_server);
    }

    // Save this file for processing once its import dependencies are ready.
    caf_cache.add_preprocessed_file(data.file.clone(), imports, data);
}

//-------------------------------------------------------------------------------------------------------------------

/// Consumes a cobweb asset file's data and loads it into [`CobwebAssetCache`].
pub(crate) fn extract_caf_data(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    file: CafFile,
    mut data: Caf,
    // [ shortname : longname ]
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
    _constants_buffer: &mut ConstantsBuffer,
    // tracks specs
    _specs: &mut SpecsMap,
)
{
    tracing::info!("extracting cobweb asset file {:?}", file.as_str());

    for section in data.sections.iter_mut() {
        match section {
            CafSection::Using(section) => extract_using_section(type_registry, &file, section, name_shortcuts),
            CafSection::Defs(_section) => {
                // TODO
                // extract_constants_section(&file, &mut data, constants_buffer)
                // extract_specs_section(&file, &mut data, specs)
                ()
            }
            CafSection::Commands(section) => {
                // search_and_replace_map_constants(&file, CONSTANT_MARKER, &mut data, constants_buffer);
                // insert_specs(&file, &mut data, specs);
                extract_commands_section(type_registry, caf_cache, &file, section, name_shortcuts);
            }
            CafSection::Scenes(section) => {
                // search_and_replace_map_constants(&file, CONSTANT_MARKER, &mut data, constants_buffer);
                // insert_specs(&file, &mut data, specs);
                extract_scenes(
                    type_registry,
                    c,
                    caf_cache,
                    scene_loader,
                    &file,
                    section,
                    name_shortcuts,
                );
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: remove/replace/implement this
#[derive(Default, Debug)]
pub(crate) struct SpecsMap;

impl SpecsMap
{
    pub(crate) fn import_specs(&mut self, _import_file: &CafFile, _file: &CafFile, _imported: &SpecsMap) {}
}

//-------------------------------------------------------------------------------------------------------------------

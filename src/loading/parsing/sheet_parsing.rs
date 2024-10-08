use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use serde_json::{Map, Value};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Preprocesses a cobweb asset file and adds it to [`CobwebAssetCache`] for processing.
///
/// Only the manifest and imports sections of the file are parsed here.
pub(crate) fn preprocess_caf_file(
    manifest: &mut HashMap<String, Arc<str>>,
    imports: &mut Vec<(String, SmolStr)>,
    asset_server: &AssetServer,
    caf_files: &mut LoadedCobwebAssetFiles,
    caf_cache: &mut CobwebAssetCache,
    file: SceneFile,
    data: Value,
)
{
    caf_cache.initialize_file(&file);

    let Value::Object(data) = data else {
        tracing::error!("failed preprocessing cobweb asset file {:?}, data base layer is not an Object", file);
        return;
    };

    // Extract manifest.
    manifest.clear();
    extract_manifest_section(&file, &data, manifest);

    // Extract imports.
    imports.clear();
    extract_import_section(&file, &data, imports);

    // Convert import file references.
    let mut imports_to_save: HashMap<SceneFile, SmolStr> = HashMap::default();
    for (file, alias) in imports.iter() {
        imports_to_save.insert(SceneFile::new(file.as_str()), alias.clone());
    }

    // Register manifest keys.
    // - We also register imported files for loading to ensure they are tracked properly and to reduce
    //   duplication/race conditions/complexity between manifest loading and imports.
    // - NOTE: We start with String keys and then convert to SceneFiles because files may target a specific asset
    //   loader (e.g. `embedded://my_file.caf.json`), but we strip that information when converting to a SceneFile.
    for (file, scenefile, manifest_key) in manifest
        .drain()
        .map(|(f, m)| {
            let scenefile = SceneFile::new(f.as_str());
            (f, scenefile, Some(m))
        })
        .chain(
            imports
                .drain(..)
                .map(|(k, _)| {
                    let scenefile = SceneFile::new(k.as_str());
                    (k, scenefile, None)
                })
                // Don't try to load imported manifest keys.
                .filter(|(_, scenefile, _)| scenefile.is_file_path()),
        )
    {
        // Continue if this file has been registered before.
        if !caf_cache.register_manifest_key(scenefile, manifest_key) {
            continue;
        }

        // Load this manifest entry.
        caf_files.start_loading(file, caf_cache, asset_server);
    }

    // Save this file for processing once its import dependencies are ready.
    caf_cache.add_preprocessed_file(file, imports_to_save, data);
}

//-------------------------------------------------------------------------------------------------------------------

/// Consumes a cobweb asset file's data and loads it into [`CobwebAssetCache`].
pub(crate) fn parse_caf_file(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    file: SceneFile,
    mut data: Map<String, Value>,
    // [ shortname : longname ]
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
    constants_buffer: &mut ConstantsBuffer,
    // tracks specs
    specs: &mut SpecsMap,
)
{
    tracing::info!("parsing cobweb asset file {:?}", file.as_str());

    // Extract using section.
    extract_using_section(type_registry, &file, &data, name_shortcuts);

    // Extract constants section.
    extract_constants_section(&file, &mut data, constants_buffer);

    // Search and replace constants.
    search_and_replace_map_constants(&file, CONSTANT_MARKER, &mut data, constants_buffer);

    // Extract specifications section.
    extract_specs_section(&file, &mut data, specs);

    // Insert spec definitions where requested in commands and data.
    // - Does not permanently mutate the specs, mutability is only for optimized spec insertion.
    insert_specs(&file, &mut data, specs);

    // Extract commands section.
    parse_commands_section(type_registry, caf_cache, &file, &mut data, name_shortcuts);

    // Parse scenes from the file.
    parse_scenes(type_registry, c, caf_cache, scene_loader, &file, data, name_shortcuts);
}

//-------------------------------------------------------------------------------------------------------------------

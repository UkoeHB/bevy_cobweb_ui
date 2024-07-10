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
    asset_server: &AssetServer,
    caf_files: &mut LoadedCobwebAssetFiles,
    caf_cache: &mut CobwebAssetCache,
    file: LoadableFile,
    data: Value,
)
{
    caf_cache.initialize_file(&file);

    let Value::Object(data) = data else {
        tracing::error!("failed preprocessing cobweb asset file {:?}, data base layer is not an Object", file);
        return;
    };

    // Extract manifest.
    let mut manifest: HashMap<LoadableFile, Arc<str>> = HashMap::default();
    extract_manifest_section(&file, &data, &mut manifest);

    // Extract imports.
    let mut imports: HashMap<LoadableFile, SmolStr> = HashMap::default();
    extract_import_section(&file, &data, &mut imports);

    // Register manifest keys.
    // - We also register imported files for loading to ensure they are tracked properly and to reduce
    //   duplication/race conditions/complexity between manifest loading and imports.
    for (file, manifest_key) in manifest
        .drain()
        .map(|(f, m)| (f, Some(m)))
        .chain(imports.keys().map(|k| (k.clone(), None)))
    {
        // Continue if this file has been registered before.
        if !caf_cache.register_manifest_key(file.clone(), manifest_key) {
            continue;
        }

        // Load this manifest entry.
        caf_files.start_loading(file, caf_cache, asset_server);
    }

    // Save this file for processing once its import dependencies are ready.
    caf_cache.add_preprocessed_file(file, imports, data);
}

//-------------------------------------------------------------------------------------------------------------------

/// Consumes a cobweb asset file's data and loads it into [`CobwebAssetCache`].
pub(crate) fn parse_caf_file(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    file: LoadableFile,
    mut data: Map<String, Value>,
    // [ shortname : longname ]
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
    // [ path : [ terminal identifier : constant value ] ]
    constants: &mut HashMap<SmolStr, HashMap<SmolStr, Arc<Value>>>,
    // tracks specs
    specs: &mut SpecsMap,
)
{
    tracing::info!("parsing cobweb asset file {:?}", file.as_str());

    // Extract using section.
    extract_using_section(type_registry, &file, &data, name_shortcuts);

    // Extract constants section.
    extract_constants_section(&file, &mut data, constants);

    // Search and replace constants.
    search_and_replace_map_constants(&file, CONSTANT_MARKER, &mut data, constants);

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

use std::collections::HashMap;

use bevy::reflect::TypeRegistry;
use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Preprocesses a loadablesheet file and adds it to [`LoadableSheet`] for processing.
///
/// Only the imports section of the file is parsed here.
pub(crate) fn preprocess_loadablesheet_file(loadablesheet: &mut LoadableSheet, file: LoadableFile, data: Value)
{
    loadablesheet.initialize_file(&file);

    let Value::Object(data) = data else {
        tracing::error!("failed preprocessing loadablesheet {:?}, data base layer is not an Object", file);
        return;
    };

    // Extract imports.
    let mut imports: HashMap<LoadableFile, String> = HashMap::default();
    extract_import_section(&file, &data, &mut imports);

    // Save this file for processing once its import dependencies are ready.
    loadablesheet.add_preprocessed_file(file, imports, data);
}

//-------------------------------------------------------------------------------------------------------------------

/// Consumes a loadablesheet file's data and loads it into [`LoadableSheet`].
pub(crate) fn parse_loadablesheet_file(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: LoadableFile,
    mut data: Map<String, Value>,
    // [ path : [ terminal identifier : constant value ] ]
    constants: &mut HashMap<String, Map<String, Value>>,
    // [ shortname : longname ]
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    tracing::info!("parsing loadablesheet {:?}", file.file);

    // [ shortname : [ loadable value ] ]
    let mut loadable_stack: HashMap<&'static str, Vec<ReflectedLoadable>> = HashMap::default();
    // [ {shortname, top index into loadablestack when first stack added this frame} ]
    let mut stack_trackers: Vec<Vec<(&'static str, usize)>> = Vec::default();

    // Extract using section.
    extract_using_section(type_registry, &file, &data, name_shortcuts);

    // Extract constants section.
    extract_constants_section(&file, &mut data, constants);

    // Search and replace constants.
    search_and_replace_map_constants(&file, "$", &mut data, &constants);

    // Recursively consume the file contents.
    parse_branch(
        type_registry,
        loadablesheet,
        &file,
        &LoadablePath::new(""),
        data,
        name_shortcuts,
        &mut loadable_stack,
        &mut stack_trackers,
    );
}

//-------------------------------------------------------------------------------------------------------------------

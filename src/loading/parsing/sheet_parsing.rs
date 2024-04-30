use std::collections::HashMap;

use bevy::reflect::TypeRegistry;
use serde_json::{Map, Value};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Consumes a loadablesheet file's data and loads it into [`LoadableSheet`].
pub(crate) fn parse_loadablesheet_file(
    type_registry: &TypeRegistry,
    loadablesheet: &mut LoadableSheet,
    file: LoadableFile,
    data: Value,
) -> bool
{
    tracing::info!("parsing loadablesheet {:?}", file.file);
    loadablesheet.initialize_file(file.clone());

    let Value::Object(mut data) = data else {
        tracing::error!("failed parsing loadablesheet {:?}, data base layer is not an Object", file);
        return false;
    };

    // [ shortname : longname ]
    let mut name_shortcuts: HashMap<&'static str, &'static str> = HashMap::default();
    // [ shortname : [ loadable value ] ]
    let mut loadable_stack: HashMap<&'static str, Vec<ReflectedLoadable>> = HashMap::default();
    // [ {shortname, top index into loadablestack when first stack added this frame} ]
    let mut stack_trackers: Vec<Vec<(&'static str, usize)>> = Vec::default();

    // TODO: handle imports

    // - get imports section
    // - check if imports available, else cache for parsing later
    // - copy saved using and constants from imported

    // Extract using section.
    extract_using_section(type_registry, &file, &data, &mut name_shortcuts);

    // Extract constants section.
    // - build constants map and check for $$ constant references
    // [ path : [ terminal identifier : constant value ] ]
    let mut constants = HashMap::<String, Map<String, Value>>::default();
    extract_constants_section(&file, &mut data, &mut constants);

    // TODO: save using and constants in case this file is imported by another file

    // Search and replace constants.

    // Recursively consume the file contents.
    parse_branch(
        type_registry,
        loadablesheet,
        &file,
        &LoadablePath::new(""),
        data,
        &mut name_shortcuts,
        &mut loadable_stack,
        &mut stack_trackers,
    );

    // On return true, load any files that depend on it.
    // TODO: use a cfg_if on the file_watcher feature to decide whether to discard all file contents once all
    // registerd sheets are done loading
    true
}

//-------------------------------------------------------------------------------------------------------------------

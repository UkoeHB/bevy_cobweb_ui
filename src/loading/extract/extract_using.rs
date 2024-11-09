use std::collections::HashMap;

use bevy::reflect::TypeRegistry;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_using_section(
    type_registry: &TypeRegistry,
    file: &CobFile,
    section: &CobUsing,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    let mut fullpath = String::default();

    for entry in section.entries.iter() {
        fullpath = entry.identifier.to_canonical(Some(fullpath));

        let Some(registration) = type_registry.get_with_type_path(fullpath.as_str()) else {
            tracing::error!("longname {:?} in 'using' section of {:?} not found in type registry",
                fullpath, file);
            continue;
        };
        let short_name = registration.type_info().type_path_table().short_path();
        let long_name = registration.type_info().type_path_table().path(); //get static version

        // Note: This overwrites existing entries for a shortname, which allows files to overload type names
        // imported from other files.
        name_shortcuts.insert(short_name, long_name);
    }
}

//-------------------------------------------------------------------------------------------------------------------

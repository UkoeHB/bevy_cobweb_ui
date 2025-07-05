use bevy::reflect::TypeRegistry;

use super::*;
use crate::cob::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_commands_section(
    type_registry: &TypeRegistry,
    commands: &mut Vec<(&'static str, ErasedLoadable)>,
    file: &CobFile,
    section: &mut CobCommands,
    loadables: &LoadableRegistry,
    resolver: &CobResolver,
)
{
    if section.entries.is_empty() {
        return;
    }

    let mock_path = ScenePath::new("#commands");
    let mut shortname = String::default();
    let mut seen_shortnames = vec![];

    for entry in section.entries.iter_mut() {
        let CobCommandEntry(loadable) = entry;
        // Get the shortname.
        shortname = loadable.id.to_canonical(Some(shortname));

        // Get the loadable's longname.
        let Some((short_name, long_name, type_id, deserializer)) =
            get_loadable_meta(type_registry, file, &mock_path, shortname.as_str(), loadables)
        else {
            continue;
        };

        // Check for duplicate.
        if seen_shortnames.iter().any(|other| *other == short_name) {
            tracing::warn!("ignoring duplicate command {} in {:?}; use Multi<{}> instead",
                short_name, file, short_name);
            continue;
        }

        seen_shortnames.push(short_name);

        // Resolve defs.
        if let Err(err) = loadable.resolve(&resolver.loadables) {
            tracing::warn!("failed extracting command {:?} in {:?}; error resolving defs: {:?}",
                short_name, file, err.as_str());
            continue;
        }

        // Get the commands's value.
        let command_value = get_loadable_value(deserializer, loadable);

        // Save the command.
        commands.push((long_name, ErasedLoadable { type_id, loadable: command_value }));
    }
}

//-------------------------------------------------------------------------------------------------------------------

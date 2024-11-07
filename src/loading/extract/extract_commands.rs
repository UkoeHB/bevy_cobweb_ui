use std::collections::HashMap;

use bevy::reflect::TypeRegistry;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_commands_section(
    type_registry: &TypeRegistry,
    commands: &mut Vec<(&'static str, ErasedLoadable)>,
    file: &CafFile,
    section: &CafCommands,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    if section.entries.is_empty() {
        return;
    }

    let mock_path = ScenePath::new("#commands");
    let mut shortname = String::default();
    let mut seen_shortnames = vec![];

    for entry in section.entries.iter() {
        match entry {
            CafCommandEntry::Loadable(loadable) => {
                // Get the shortname.
                shortname = loadable.id.to_canonical(Some(shortname));

                // Get the loadable's longname.
                let Some((short_name, long_name, type_id, deserializer)) =
                    get_loadable_meta(type_registry, file, &mock_path, shortname.as_str(), name_shortcuts)
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

                // Get the commands's value.
                let command_value = get_loadable_value(deserializer, loadable);

                // Save the command.
                commands.push((long_name, ErasedLoadable { type_id, loadable: command_value }));
            }
            CafCommandEntry::LoadableMacroCall(_) => {
                tracing::error!("ignoring unresolved loadable macro in CAF file command section {:?}", file);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

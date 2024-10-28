use std::collections::HashMap;

use bevy::reflect::TypeRegistry;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn extract_commands_section(
    type_registry: &TypeRegistry,
    caf_cache: &mut CobwebAssetCache,
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

    for entry in section.entries.iter() {
        match entry {
            CafCommandEntry::Instruction(instruction) => {
                // Get the shortname.
                shortname = instruction.id.to_canonical(Some(shortname));

                // Get the loadable's longname.
                let Some((_short_name, long_name, type_id, deserializer)) =
                    get_loadable_meta(type_registry, file, &mock_path, shortname.as_str(), name_shortcuts)
                else {
                    continue;
                };

                // Get the loadable's value.
                let command_value = get_loadable_value(deserializer, instruction);

                // Save this command.
                caf_cache.insert_command(
                    &SceneRef { file: SceneFile::File(file.clone()), path: mock_path.clone() },
                    command_value,
                    type_id,
                    long_name,
                );
            }
            CafCommandEntry::InstructionMacroCall(_) => {
                tracing::error!("ignoring unresolved instruction macro in CAF file command section {:?}", file);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

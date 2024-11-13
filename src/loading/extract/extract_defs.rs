use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn extract_constant_entry(file: &CobFile, mut entry: CobConstantDef, constants_buffer: &mut ConstantsBuffer)
{
    // Resolve the def's internal value.
    if let Err(err) = entry.value.resolve(constants_buffer) {
        tracing::warn!("failed extracting constant entry {:?} in {:?}; error resolving internal defs: {:?}",
            entry.name.as_str(), file, err.as_str());
        return;
    }

    // Save the constant definition in the constants buffer.
    constants_buffer.insert(entry.name.name, entry.value);
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_data_macro_entry(_file: &CobFile, _entry: CobDataMacroDef, _constants_buffer: &ConstantsBuffer) {}

//-------------------------------------------------------------------------------------------------------------------

fn extract_loadable_macro_entry(_file: &CobFile, _entry: CobLoadableMacroDef, _constants_buffer: &ConstantsBuffer)
{
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_scene_macro_entry(_file: &CobFile, _entry: CobSceneMacroDef, _constants_buffer: &ConstantsBuffer) {}

//-------------------------------------------------------------------------------------------------------------------

/// Removes all definitions and caches them in appropriate buffers/maps.
pub(super) fn extract_defs_section(file: &CobFile, section: &mut CobDefs, constants_buffer: &mut ConstantsBuffer)
{
    for entry in section.entries.drain(..) {
        match entry {
            CobDefEntry::Constant(entry) => extract_constant_entry(file, entry, constants_buffer),
            CobDefEntry::DataMacro(entry) => extract_data_macro_entry(file, entry, constants_buffer),
            CobDefEntry::LoadableMacro(entry) => extract_loadable_macro_entry(file, entry, constants_buffer),
            CobDefEntry::SceneMacro(entry) => extract_scene_macro_entry(file, entry, constants_buffer),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

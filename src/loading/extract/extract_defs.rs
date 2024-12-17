use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn extract_constant_entry(file: &CobFile, mut entry: CobConstantDef, resolver: &mut CobLoadableResolver)
{
    // Resolve the def's internal value.
    if let Err(err) = entry.value.resolve(resolver) {
        tracing::warn!("failed extracting constant definition {:?} in {:?}; error resolving internal defs: {:?}",
            entry.name.as_str(), file, err.as_str());
        return;
    }

    // Save the constant definition in the constants buffer.
    resolver
        .constants
        .insert(file, entry.name.name, entry.value);
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_scene_macro_entry(file: &CobFile, mut entry: CobSceneMacroDef, resolver: &mut CobResolver)
{
    // Full-resolve the definition content.
    if let Err(err) = entry.value.resolve(resolver, SceneResolveMode::Full) {
        tracing::warn!("failed extracting scene macro definition {:?} in {:?}; error resolving internal defs: {:?}",
            entry.name.as_str(), file, err.as_str());
        return;
    }

    // Save the scene macro definition in the scene macro buffer.
    resolver
        .scenes
        .scene_macros
        .insert(file, entry.name.name, entry.value);
}

//-------------------------------------------------------------------------------------------------------------------

/// Removes all definitions and caches them in appropriate buffers/maps.
pub(super) fn extract_defs_section(file: &CobFile, section: &mut CobDefs, resolver: &mut CobResolver)
{
    for entry in section.entries.drain(..) {
        match entry {
            CobDefEntry::Constant(entry) => extract_constant_entry(file, entry, &mut resolver.loadables),
            CobDefEntry::SceneMacro(entry) => extract_scene_macro_entry(file, entry, resolver),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

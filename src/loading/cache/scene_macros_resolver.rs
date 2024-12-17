use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use smallvec::SmallVec;
use smol_str::SmolStr;

use super::*;
use crate::loading::{
    CobFile, CobFill, CobLoadableIdentifier, CobSceneLayer, CobSceneLayerEntry, CobSceneMacroCall,
    CobSceneMacroCommandType, CobSceneMacroValue, CobSceneNodeName,
};
use crate::prelude::CobImportAlias;

//-------------------------------------------------------------------------------------------------------------------

fn set_canonical_id(canonical: &str, id: &mut CobLoadableIdentifier)
{
    if id.name == canonical {
        return;
    }
    id.name = SmolStr::from(canonical);
    id.generics = None;
}

//-------------------------------------------------------------------------------------------------------------------

fn canonicalize_loadable_names(mut id_scratch: String, entries: &mut Vec<CobSceneLayerEntry>) -> String
{
    for entry in entries.iter_mut() {
        match entry {
            CobSceneLayerEntry::Loadable(loadable) => {
                id_scratch = loadable.id.to_canonical(Some(id_scratch));
                set_canonical_id(id_scratch.as_str(), &mut loadable.id);
            }
            CobSceneLayerEntry::Layer(layer) => {
                id_scratch = canonicalize_loadable_names(id_scratch, &mut layer.entries);
            }
            _ => (),
        }
    }

    id_scratch
}

//-------------------------------------------------------------------------------------------------------------------

fn expand_macro_recursive(
    mut id_scratch: String,
    result_entries: &mut Vec<CobSceneLayerEntry>,
    call_entries: &[CobSceneLayerEntry],
) -> String
{
    // Apply loadable adjustments.
    for entry in call_entries.iter() {
        match entry {
            CobSceneLayerEntry::Loadable(loadable) => {
                // Overwrite or insert the laodable.
                // Note: result_entries loadable names are canonical.
                id_scratch = loadable.id.to_canonical(Some(id_scratch));
                let mut new = loadable.clone();

                match result_entries.iter_mut().find_map(|layer| {
                    let CobSceneLayerEntry::Loadable(loadable) = layer else { return None };
                    if loadable.id.name == id_scratch {
                        Some(loadable)
                    } else {
                        None
                    }
                }) {
                    Some(existing) => {
                        std::mem::swap(existing, &mut new);
                        existing.id = new.id;
                    }
                    None => {
                        set_canonical_id(id_scratch.as_str(), &mut new.id);
                        result_entries.push(CobSceneLayerEntry::Loadable(new));
                    }
                }
            }
            CobSceneLayerEntry::SceneMacroCommand(command) => {
                // Find the targeted loadable.
                id_scratch = command.id.to_canonical(Some(id_scratch));

                let Some(pos) = result_entries.iter().position(|layer| {
                    let CobSceneLayerEntry::Loadable(loadable) = layer else { return false };
                    loadable.id.name == id_scratch
                }) else {
                    continue;
                };

                // Apply the command.
                let removed = result_entries.remove(pos);
                match command.command_type {
                    CobSceneMacroCommandType::MoveToTop => {
                        result_entries.insert(0, removed);
                    }
                    CobSceneMacroCommandType::MoveToBottom => {
                        result_entries.insert(result_entries.len(), removed);
                    }
                    CobSceneMacroCommandType::Remove => (), // Already removed
                }
            }
            _ => (),
        }
    }

    // Update the scene layer.
    // - Apply scene node rearrangements and additions at this level.
    // - Recurse into each layer to update its contents.
    let mut prev = None;
    for entry in call_entries.iter() {
        let CobSceneLayerEntry::Layer(layer) = entry else { continue };
        let layer_id = layer.name.as_str();

        match result_entries.iter().position(|layer| {
            let CobSceneLayerEntry::Layer(layer) = layer else { return false };
            layer.name.as_str() == layer_id
        }) {
            Some(existing) => {
                let prev_prev = prev;
                prev = Some(existing);

                // If this entry is above the previous layer entry in result_entries, then we need to move it
                // immediately after the previous entry.
                if let Some(prev_idx) = prev_prev {
                    if existing < prev_idx {
                        let to_relocate = result_entries.remove(existing);
                        result_entries.insert(prev_idx, to_relocate); // prev entry idx gets bumped down on remove
                        prev = Some(prev_idx);
                    }
                }
            }
            None => {
                // We make a new layer instead of cloning since the new layer needs to be built up
                // from macro call contents.
                let new_layer = CobSceneLayer {
                    name_fill: CobFill::default(),
                    name: CobSceneNodeName(SmolStr::from(layer_id)),
                    entries: vec![],
                };
                result_entries.push(CobSceneLayerEntry::Layer(new_layer));
                prev = Some(result_entries.len() - 1);
            }
        }

        // Recurse into the layer.
        let result_idx = prev.expect("index was just set");
        let CobSceneLayerEntry::Layer(result_layer) = &mut result_entries[result_idx] else { unreachable!() };
        id_scratch = expand_macro_recursive(id_scratch, &mut result_layer.entries, &layer.entries);
    }

    id_scratch
}

//-------------------------------------------------------------------------------------------------------------------

// [ identifier : macro value ]
type SceneMacrosMap = HashMap<SmolStr, CobSceneMacroValue>;

//-------------------------------------------------------------------------------------------------------------------

/// Records a stack of scene macro maps.
///
/// Used to efficiently merge scene macros when importing them into new files.
#[derive(Default, Debug)]
pub struct SceneMacrosResolver
{
    stack: SmallVec<[(SmolStr, Arc<SceneMacrosMap>); 5]>,
    new_file: SceneMacrosMap,
    id_scratch: String,
}

impl SceneMacrosResolver
{
    pub(crate) fn start_new_file(&mut self)
    {
        self.new_file = HashMap::default();
    }

    pub(crate) fn end_new_file(&mut self)
    {
        let map = std::mem::take(&mut self.new_file);
        self.stack.push((SmolStr::default(), Arc::new(map)));
    }

    /// Adds an entry to the new file being collected.
    pub(crate) fn insert(&mut self, file: &CobFile, name: SmolStr, mut value: CobSceneMacroValue)
    {
        // Canonicalize all loadable names in the macro value.
        self.id_scratch = canonicalize_loadable_names(std::mem::take(&mut self.id_scratch), &mut value.entries);

        match self.new_file.entry(name) {
            Entry::Vacant(vacant) => {
                vacant.insert(value);
            }
            Entry::Occupied(mut occupied) => {
                tracing::warn!("overwriting scene macro definition +{} in {:?}", occupied.key().as_str(), file);
                occupied.insert(value);
            }
        }
    }

    /// Searches backward through the stack until a match is found.
    pub fn get(&self, path: impl AsRef<str>) -> Option<&CobSceneMacroValue>
    {
        let path = path.as_ref();
        self.new_file.get(path).or_else(|| {
            self.stack.iter().rev().find_map(|(prefix, m)| {
                let stripped = path.strip_prefix(prefix.as_str())?;
                let cleaned = stripped.strip_prefix(DEFS_SEPARATOR).unwrap_or(stripped);
                m.get(cleaned)
            })
        })
    }

    /// Expands a scene macro invocation into scene layer entries.
    pub fn expand(&mut self, call: &CobSceneMacroCall) -> Result<Vec<CobSceneLayerEntry>, String>
    {
        let path = call.path.as_str();
        let mut result_entries = self
            .get(path)
            .ok_or_else(|| format!("no scene macro definition at '{path}'"))?
            .entries
            .clone();

        self.id_scratch = expand_macro_recursive(
            std::mem::take(&mut self.id_scratch),
            &mut result_entries,
            &call.container.entries,
        );

        Ok(result_entries)
    }

    pub(crate) fn append(&mut self, alias: &CobImportAlias, to_append: &Self)
    {
        let alias = alias.as_str();

        // Remove duplicate maps in self.
        for (to_append_prefix, to_append) in to_append.stack.iter() {
            let new_to_append_prefix = path_to_string(DEFS_SEPARATOR, &[alias, &*to_append_prefix]);
            let Some(existing) = self.stack.iter().position(|(prefix, m)| {
                *prefix == new_to_append_prefix && Arc::as_ptr(m) == Arc::as_ptr(to_append)
            }) else {
                continue;
            };
            self.stack.remove(existing);
        }

        // Append.
        self.stack.reserve(to_append.stack.len());
        self.stack
            .extend(to_append.stack.iter().map(|(old_prefix, map)| {
                let new_prefix = path_to_string(DEFS_SEPARATOR, &[alias, &*old_prefix]);
                (new_prefix, map.clone())
            }));
    }
}

//-------------------------------------------------------------------------------------------------------------------

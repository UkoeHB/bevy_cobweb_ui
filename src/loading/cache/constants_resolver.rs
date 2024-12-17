use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use smallvec::SmallVec;
use smol_str::SmolStr;

use super::*;
use crate::loading::{CobConstantValue, CobFile};
use crate::prelude::CobImportAlias;

//-------------------------------------------------------------------------------------------------------------------

// [ identifier : constant value ]
type ConstantsMap = HashMap<SmolStr, CobConstantValue>;

//-------------------------------------------------------------------------------------------------------------------

/// Records a stack of constant maps.
///
/// Used to efficiently merge constants when importing them into new files.
#[derive(Default, Debug)]
pub struct ConstantsResolver
{
    stack: SmallVec<[(SmolStr, Arc<ConstantsMap>); 5]>,
    new_file: ConstantsMap,
}

impl ConstantsResolver
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
    pub(crate) fn insert(&mut self, file: &CobFile, name: SmolStr, value: CobConstantValue)
    {
        match self.new_file.entry(name) {
            Entry::Vacant(vacant) => {
                vacant.insert(value);
            }
            Entry::Occupied(mut occupied) => {
                tracing::warn!("overwriting constant definition ${} in {:?}", occupied.key().as_str(), file);
                occupied.insert(value);
            }
        }
    }

    /// Searches backward through the stack until a match is found.
    pub fn get(&self, path: impl AsRef<str>) -> Option<&CobConstantValue>
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

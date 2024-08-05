use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

// [ path : [ terminal identifier : constant value ] ]
type ConstantsMap = HashMap<SmolStr, HashMap<SmolStr, Value>>;

//-------------------------------------------------------------------------------------------------------------------

/// Records a stack of constant maps.
///
/// Used to efficiently merge constants when importing them into new files.
#[derive(Default, Debug)]
pub(crate) struct ConstantsBuffer
{
    stack: Vec<(SmolStr, Arc<ConstantsMap>)>,
    new_file: ConstantsMap,
}

impl ConstantsBuffer
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

    /// Adds an entry to the new file being constructed.
    pub(crate) fn add_entry(&mut self, path: SmolStr, map: HashMap<SmolStr, Value>)
    {
        self.new_file.insert(path, map);
    }

    /// Gets an already-inserted entry in the new file being constructed.
    pub(crate) fn get_entry_mut(&mut self, path: impl AsRef<str>) -> Option<&mut HashMap<SmolStr, Value>>
    {
        let path = path.as_ref();
        self.new_file.get_mut(path)
    }

    /// Searches backward through the stack until a match is found.
    pub(crate) fn get_path(&self, path: impl AsRef<str>) -> Option<&HashMap<SmolStr, Value>>
    {
        let path = path.as_ref();
        self.new_file.get(path).or_else(|| {
            self.stack.iter().rev().find_map(|(prefix, m)| {
                let stripped = path.strip_prefix(prefix.as_str())?;
                let cleaned = stripped
                    .strip_prefix(CONSTANT_SEPARATOR)
                    .unwrap_or(stripped);
                m.get(cleaned)
            })
        })
    }

    pub(crate) fn append(&mut self, alias: impl AsRef<str>, to_append: &Self)
    {
        let alias = alias.as_ref();

        // Remove duplicates in self.
        for (to_append_prefix, to_append) in to_append.stack.iter() {
            let new_to_append_prefix = path_to_string(CONSTANT_SEPARATOR, &[alias, &*to_append_prefix]);
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
                let new_prefix = path_to_string(CONSTANT_SEPARATOR, &[alias, &*old_prefix]);
                (new_prefix, map.clone())
            }));
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::collections::HashMap;

use serde_json::{Map, Value};
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone)]
struct SpecData
{
    /// [ param key : { saved value, cached temp override value } ]
    params: HashMap<SmolStr, (Value, Value)>,
    /// Cache for collecting temp inserts that need to be applied to the content.
    inserts_cache: SmallVec<[(SmolStr, Value); 2]>,
    /// The unresolved value of this spec.
    content: Value,
}

impl SpecData
{
    fn new(file: &LoadableFile, spec_invocation: &str, map: &mut Map<String, Value>) -> Self
    {
        let mut spec = Self::default();
        spec.update_from_specs_override(file, spec_invocation, map);
        if spec.content == Value::Null {
            tracing::warn!("new spec {} defined in {:?} has no content", spec_invocation, file);
        }
        spec
    }

    fn clear_cached_edits(&mut self)
    {
        self.params.retain(|_, (saved, temp)| {
            *temp = Value::Null;
            // Clean up map entries for temporary parameters that did not override pre-existing params.
            if let (Value::Null, Value::Null) = (saved, temp) {
                return false;
            }
            true
        });
        self.inserts_cache.clear();
    }

    /// Extracts spec edits from a map of params and inserts.
    fn extract_specs(
        &mut self,
        file: &LoadableFile,
        spec_invocation: &str,
        map: &mut Map<String, Value>,
        merge_params: bool,
    ) -> bool
    {
        for (key, value) in map.iter_mut() {
            if key == SPEC_CONTENT_SYMBOL {
                if self.content != Value::Null {
                    tracing::warn!("ignoring content in {:?} for spec {} that already has content",
                        file, spec_invocation);
                    continue;
                }
                self.content = value.take();
            } else if key.starts_with(SPEC_PARAMETER_MARKER) {
                match self.params.contains_key(key.as_str()) {
                    true => {
                        let (saved_param, temp_param) = self.params.get_mut(key.as_str()).unwrap();
                        match merge_params {
                            true => {
                                // This overwrites the previous value.
                                *saved_param = value.take();
                            }
                            false => {
                                *temp_param = value.take();
                            }
                        }
                    }
                    false => {
                        //todo: check that newly-added params are present in the content after insertions are
                        // applied so users can be warned if they made a typo or forgot to
                        // use a param/have extra params
                        match merge_params {
                            true => {
                                self.params.insert(key.into(), (value.take(), Value::Null));
                            }
                            false => {
                                self.params.insert(key.into(), (Value::Null, value.take()));
                            }
                        }
                    }
                }
            } else if key.starts_with(SPEC_INSERTION_MARKER) {
                self.inserts_cache.push((key.as_str().into(), value.take()));
            } else if key.starts_with(COMMENT_KEYWORD) {
                continue;
            } else {
                tracing::warn!("ignoring invalid spec key {} in spec invocation {} in {:?}",
                    key, spec_invocation, file);
            }
        }

        map.len() > 0
    }

    fn apply_insertions_to_spec_content(&self, file: &LoadableFile, value: &mut Value)
    {
        let mut insertion_cache = SmallVec::<[usize; 4]>::default();
        match value {
            Value::Object(map) => {
                // Iterate map entries.
                for (key, value) in map.iter_mut() {
                    // Insertion key: look in temp storage for value to insert.
                    if key.starts_with(SPEC_INSERTION_MARKER) {
                        if let Some(to_insert) = self
                            .inserts_cache
                            .iter()
                            .position(|(cached_key, _)| cached_key == key)
                        {
                            insertion_cache.push(to_insert);
                        }
                        continue;
                    }

                    // Normal entry: recurse into value.
                    self.apply_insertions_to_spec_content(file, value);
                }

                // Insert cached insertions (must be maps).
                for insert_idx in insertion_cache.drain(..) {
                    let (key, value) = &self.inserts_cache[insert_idx];
                    let Value::Object(insertion_map) = value else {
                        tracing::warn!("ignoring spec insertion {} for key {} in {:?}, value to insert is not a map but \
                            the insertion point is a map key", value, key, file);
                        continue;
                    };
                    for (key, value) in insertion_map.iter() {
                        map.insert(key.clone(), value.clone());
                    }
                }
            }
            Value::Array(arr) => {
                // Iterate array entries.
                for value in arr.iter_mut() {
                    // Non-string: recurse into it.
                    let Value::String(value_str) = value else {
                        self.apply_insertions_to_spec_content(file, value);
                        continue;
                    };

                    // Insertion key: look in temp storage for value to insert.
                    if !value_str.starts_with(SPEC_INSERTION_MARKER) {
                        continue;
                    }
                    let Some(to_insert) = self
                        .inserts_cache
                        .iter()
                        .position(|(cached_key, _)| cached_key == value_str)
                    else {
                        continue;
                    };
                    insertion_cache.push(to_insert);
                }

                // Insert cached insertions (must be arrays).
                for insert_idx in insertion_cache.drain(..) {
                    let (key, value) = &self.inserts_cache[insert_idx];
                    let Value::Array(insertion_vec) = value else {
                        tracing::warn!("ignoring spec insertion {} for key {} in {:?}, value to insert is not an array but \
                            the insertion point is an array entry", value, key, file);
                        continue;
                    };
                    let vec_idx = arr
                        .iter()
                        .position(|v| {
                            let Value::String(v_str) = v else { return false };
                            v_str == key
                        })
                        .unwrap();
                    arr.reserve(insertion_vec.len());
                    // Use a reverse iterator so final order matches order within the insertion vec.
                    for value in insertion_vec.iter().rev() {
                        arr.insert(vec_idx, value.clone());
                    }
                }
            }
            _ => (),
        }
    }

    fn recursively_resolve_spec_content(
        &self,
        file: &LoadableFile,
        value: &mut Value,
        is_resolving_insertion: bool,
    )
    {
        match value {
            Value::Object(map) => {
                let mut insertion_cache = SmallVec::<[usize; 4]>::default();

                // Iterate map entries.
                map.retain(|key, value| {
                    // Param key: remove and warn.
                    if key.starts_with(SPEC_PARAMETER_MARKER) {
                        tracing::warn!("ignoring spec param {} found in map key in {:?}, map key params not allowed",
                            key, file);
                        return false;
                    }

                    // Insertion key: remove and save for insertion.
                    if key.starts_with(SPEC_INSERTION_MARKER) {
                        if let Some(to_insert) = self.inserts_cache.iter().position(|(cached_key, _)| cached_key == key) {
                            if !is_resolving_insertion {
                                insertion_cache.push(to_insert);
                            } else {
                                tracing::warn!("removing nested spec insertion for key {} in {:?}, currently recursing over \
                                    a previous insertion and nested insertions are not supported", key, file);
                            }
                        }
                        return false;
                    }

                    // Spec invocation: ignore (recursive specs are handled in the external recursion, and spec params
                    // cannot be parameterized so there is no need to inspect the value)
                    if let Ok(Some(_)) = try_parse_spec_invocation(key) {
                        return true;
                    }

                    // Normal entry: recurse into value.
                    self.recursively_resolve_spec_content(file, value, is_resolving_insertion);
                    true
                });

                // Insert cached insertions (must be maps).
                for insert_idx in insertion_cache.drain(..) {
                    let (key, value) = &self.inserts_cache[insert_idx];
                    let Value::Object(insertion_map) = value else {
                        tracing::warn!("ignoring spec insertion {} for key {} in {:?}, value to insert is not a map but \
                            the insertion point is a map key", value, key, file);
                        continue;
                    };
                    for (key, value) in insertion_map.iter() {
                        // Recurse into the inserted value in case it needs any spec params.
                        let mut value = value.clone();
                        self.recursively_resolve_spec_content(file, &mut value, true);
                        map.insert(key.clone(), value);
                    }
                }
            }
            Value::Array(arr) => {
                // { (index of insertion value, index in array where to insert it) }
                let mut insertion_cache = SmallVec::<[(usize, usize); 4]>::default();

                // Iterate array entries.
                let mut index_head = 0;
                arr.retain_mut(|value| {
                    index_head += 1;

                    // Non-string: recurse into it.
                    let Value::String(value_str) = value else {
                        self.recursively_resolve_spec_content(file, value, is_resolving_insertion);
                        return true;
                    };

                    // Param key: recurse to set it.
                    if value_str.starts_with(SPEC_PARAMETER_MARKER) {
                        self.recursively_resolve_spec_content(file, value, is_resolving_insertion);
                        return true;
                    }

                    // Insertion key: remove and look in temp storage for value to insert
                    if value_str.starts_with(SPEC_INSERTION_MARKER) {
                        index_head -= 1;
                        if let Some(to_insert) = self.inserts_cache.iter().position(|(cached_key, _)| cached_key == value_str) {
                            if !is_resolving_insertion {
                                insertion_cache.push((to_insert, index_head));
                            } else {
                                tracing::warn!("removing nested spec insertion for key {} in {:?}, currently recursing over \
                                    a previous insertion and nested insertions are not supported", value_str, file);
                            }
                        }
                        return false;
                    }

                    // Normal string: ignore it.
                    true
                });

                // Insert cached insertions (must be arrays).
                // - Insertions are expanded 'in-place' at the spot where the insertion marker was removed.
                let mut num_insertions = 0;
                for (insert_idx, location_idx) in insertion_cache.drain(..) {
                    let (key, value) = &self.inserts_cache[insert_idx];
                    let Value::Array(insertion_vec) = value else {
                        tracing::warn!("ignoring spec insertion {} for key {} in {:?}, value to insert is not an array but \
                            the insertion point is an array entry", value, key, file);
                        continue;
                    };
                    let vec_idx = num_insertions + location_idx;
                    num_insertions += insertion_vec.len();
                    arr.reserve(insertion_vec.len());
                    // Use a reverse iterator so final order matches order within the insertion vec.
                    for value in insertion_vec.iter().rev() {
                        // Recurse into the inserted value in case it needs any spec params.
                        let mut value = value.clone();
                        self.recursively_resolve_spec_content(file, &mut value, true);
                        arr.insert(vec_idx, value);
                    }
                }
            }
            Value::String(string) => {
                // Insertion key: warn and ignore.
                if string.starts_with(SPEC_INSERTION_MARKER) {
                    tracing::warn!("ignoring spec insertion {} in {:?}, marker found in map element but only map keys and \
                        array elements are supported", string, file);
                    return;
                }

                // Other non-param keys: ignore.
                if !string.starts_with(SPEC_PARAMETER_MARKER) {
                    return;
                }

                // Param key: replace value with param (favor temp value over saved value).
                let Some((saved, temp)) = self.params.get(string.as_str()) else {
                    tracing::warn!("failed setting param {} in {:?} while resolving a spec, the spec doesn't contain any \
                        values for this param", value, file);
                    return;
                };

                let next_value = match temp {
                    Value::Null => saved.clone(),
                    _ => temp.clone(),
                };

                *value = next_value;
            }
            _ => (),
        }
    }

    /// Overwrites params, adds new params, applies insertions.
    fn update_from_specs_override(
        &mut self,
        file: &LoadableFile,
        spec_invocation: &str,
        overrides: &mut Map<String, Value>,
    )
    {
        // Extract override edits.
        self.extract_specs(file, spec_invocation, overrides, true);

        // Insert cached insertions to the spec.
        let mut content = self.content.take();
        self.apply_insertions_to_spec_content(file, &mut content);
        self.content = content;
        self.inserts_cache.clear();
    }

    /// Extracts spec edits from a value, then overwrites the value with the spec content.
    ///
    /// Spec content is resolved by applying params and inserting new sections.
    fn write_to_value(&mut self, file: &LoadableFile, spec_invocation: &str, value: &mut Value)
    {
        // Extract local edits specified in the value.
        let has_edits = match value {
            Value::Object(map) => self.extract_specs(file, spec_invocation, map, false),
            _ => false,
        };

        // Write the spec to the value and apply params/inserts.
        *value = self.content.clone();
        self.recursively_resolve_spec_content(file, value, false);

        // Cleanup
        if has_edits {
            self.clear_cached_edits();
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
pub(crate) struct SpecsMap
{
    map: HashMap<SmolStr, SpecData>,
}

impl SpecsMap
{
    /// Copies an imported spec map into this spec map.
    ///
    /// Imported specs are inserted directly, which means there is no way to disambiguate which file a spec came
    /// from. This is necessary to support nested specs, where the internal spec name cannot be overridden (so spec
    /// names cannot be contextual).
    pub(crate) fn import_specs(&mut self, import_file: &LoadableFile, file: &LoadableFile, imported: &SpecsMap)
    {
        for (spec_key, data) in imported.map.iter() {
            if let Some(_) = self.map.insert(spec_key.clone(), data.clone()) {
                tracing::warn!("overwriting spec definition {} in {:?} with version imported from {:?}",
                    spec_key, file, import_file);
            }
        }
    }

    /// Updates an existing spec or inserts a new one.
    /// - If `key` is a non-spec-invocation, then a new spec is inserted.
    /// - If `key` is a spec-invocation with key name equal to spec name, then the existing spec with that name is
    ///   overwritten.
    /// - If `key` is a spec-invocation with key name different from spec name, then a new spec is inserted that
    ///   clones and updates the exisitng spec with that spec name.
    fn update_or_insert_spec(&mut self, file: &LoadableFile, key: &str, value: &mut Value)
    {
        let Value::Object(map) = value else {
            tracing::warn!("failed evaluating spec {} in {:?}, value is not a map", key, file);
            return;
        };

        match try_parse_spec_invocation(key) {
            Ok(None) => {
                // Insert a new spec.
                if let Some(_) = self.map.insert(key.into(), SpecData::new(file, key, map)) {
                    tracing::warn!("overwriting existing spec definition {} in {:?}", key, file);
                }
            }
            Ok(Some((new_key, spec_key))) => {
                let Some(data) = self.map.get_mut(spec_key) else {
                    tracing::warn!("ignoring specification override {} with unknown spec {} in 'specs' section of {:?}",
                        key, spec_key, file);
                    return;
                };

                if new_key != spec_key {
                    // New key = add new spec
                    let mut data = data.clone();
                    data.update_from_specs_override(file, key, map);
                    if let Some(_) = self.map.insert(new_key.into(), data) {
                        tracing::warn!("overwriting existing spec definition {} in {:?}", new_key, file);
                    }
                } else {
                    // Same key = override existing spec
                    data.update_from_specs_override(file, key, map);
                }
            }
            Err(_) => {
                tracing::warn!("ignoring specification definition {} with invalid key format in 'specs' section of {:?}",
                    key, file);
                return;
            }
        }
    }

    /// Extracts a spec into a (non-spec-section) spec invocation (which may override parts of the spec).
    fn try_extract_spec_entry(&mut self, file: &LoadableFile, key: &str, value: &mut Value) -> bool
    {
        let (_invocation_id, spec_key) = match try_parse_spec_invocation(key) {
            Ok(Some(parsed_keys)) => parsed_keys,
            Ok(None) => return false,
            Err(_) => {
                tracing::warn!("ignoring suspected specification request {} with invalid format in {:?}", key, file);
                return false;
            }
        };
        let Some(data) = self.map.get_mut(spec_key) else {
            tracing::warn!("ignoring specification request {} with unknown key {} in {:?}", key, spec_key, file);
            return false;
        };

        // Set the spec value.
        data.write_to_value(file, key, value);
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Extracts a specs section from a file and inserts its contents to the file's specs map (which was initialized by
/// imports).
pub(crate) fn extract_specs_section(file: &LoadableFile, map: &mut Map<String, Value>, specs: &mut SpecsMap)
{
    let Some(specs_section) = map.get_mut(SPECS_KEYWORD) else {
        return;
    };

    let Value::Object(specs_section) = specs_section else {
        tracing::error!("failed parsing 'specs' section in {:?}, it is not an Object", file);
        return;
    };

    for (key, value) in specs_section.iter_mut() {
        specs.update_or_insert_spec(file, key, value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Iterates through an entire `data` map to insert specs where requested.
///
/// Insertion is recursive, which means specs within specs are allowed.
pub(crate) fn insert_specs(file: &LoadableFile, data: &mut Map<String, Value>, specs: &mut SpecsMap)
{
    // Iterate through the map to insert specs where requested.
    for (key, value) in data.iter_mut() {
        // Skip irrelevant keywords.
        if is_keyword_for_non_spec_editable_section(key.as_str()) {
            continue;
        }

        // Try to extract into the value.
        specs.try_extract_spec_entry(file, key, value);

        // Don't recurse into loadables.
        // Note: Don't need to strip the spec invocation from the key, since this only checks the first character.
        if is_loadable_entry(key.as_str()) {
            continue;
        }

        // Insertion failed, so recurse.
        // Note: if recursion isn't possible due to invalid content, errors will be thoroughly logged in the data
        // parser.
        let Value::Object(map) = value else { continue };
        insert_specs(file, map, specs);
    }
}

//-------------------------------------------------------------------------------------------------------------------

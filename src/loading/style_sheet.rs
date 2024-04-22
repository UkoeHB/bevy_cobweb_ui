use std::any::{type_name, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bevy::prelude::*;
use bevy::utils::warn_once;
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn setup_stylesheet(sheet_list: Res<StyleSheetList>, mut stylesheet: ReactResMut<StyleSheet>)
{
    // begin tracking expected stylesheet files
    for file in sheet_list.iter_files() {
        stylesheet.get_noreact().prepare_file(StyleFile::new(file.as_str()));
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn load_style_changes(
    mut c: Commands,
    mut events: EventReader<AssetEvent<StyleSheetAsset>>,
    sheet_list: Res<StyleSheetList>,
    mut assets: ResMut<Assets<StyleSheetAsset>>,
    mut stylesheet: ReactResMut<StyleSheet>,
    types: Res<AppTypeRegistry>,
)
{
    if events.is_empty() {
        return;
    }

    let type_registry = types.read();
    let mut need_reactions = false;

    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => id,
            _ => {
                tracing::debug!("ignoring stylesheet asset event {:?}", event);
                continue;
            }
        };

        let Some(handle) = sheet_list.get_handle(*id) else {
            tracing::warn!("encountered stylesheet asset event {:?} for an untracked asset", id);
            continue;
        };

        let Some(asset) = assets.remove(handle) else {
            tracing::error!("failed to remove stylesheet asset {:?}", handle);
            continue;
        };

        let stylesheet = stylesheet.get_noreact();
        parse_stylesheet_file(&type_registry, stylesheet, asset.file, asset.data);
        need_reactions = true;
    }

    if need_reactions {
        stylesheet.get_mut(&mut c);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn cleanup_stylesheet(mut stylesheet: ReactResMut<StyleSheet>, mut removed: RemovedComponents<LoadedStyles>)
{
    for removed in removed.read() {
        stylesheet.get_noreact().remove_entity(removed);
    }

    stylesheet.get_noreact().cleanup_pending();
}

//-------------------------------------------------------------------------------------------------------------------

struct ErasedStyle
{
    type_id: TypeId,
    style: ReflectedStyle,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) enum ReflectedStyle
{
    Value(Arc<Box<dyn Reflect + 'static>>),
    DeserializationFailed(Arc<serde_json::Error>),
}

impl ReflectedStyle
{
    pub(crate) fn equals(&self, other: &ReflectedStyle) -> Option<bool>
    {
        let (Self::Value(this), Self::Value(other)) = (self, other) else {
            return Some(false);
        };

        this.reflect_partial_eq(other.as_reflect())
    }

    pub(crate) fn get_value<T: LoadableStyle>(&self, style_ref: &StyleRef) -> Option<T>
    {
        match self {
            ReflectedStyle::Value(style) => {
                let Some(new_value) = T::from_reflect(style.as_reflect()) else {
                    let temp = T::default();
                    let mut hint = serde_json::to_string_pretty(&temp).unwrap();
                    if hint.len() > 250 {
                        hint = serde_json::to_string(&temp).unwrap();
                    }
                    tracing::error!("failed reflecting style {:?} at path {:?} in file {:?}\n\
                        serialization hint: {}",
                        type_name::<T>(), style_ref.path.path, style_ref.file, hint.as_str());
                    return None;
                };
                Some(new_value)
            }
            ReflectedStyle::DeserializationFailed(err) => {
                let temp = T::default();
                let mut hint = serde_json::to_string_pretty(&temp).unwrap();
                if hint.len() > 250 {
                    hint = serde_json::to_string(&temp).unwrap();
                }
                tracing::error!("failed deserializing style {:?} at path {:?} in file {:?}, {:?}\n\
                    serialization hint: {}",
                    type_name::<T>(), style_ref.path.path, style_ref.file, **err, hint.as_str());
                None
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive resource for managing styles loaded from stylesheet assets.
///
/**
### Stylesheet asset format

Stylesheets are written as JSON files with the extension `.style.json`. You must register stylesheets in your app with
[`StyleSheetListAppExt::add_style_sheet`].

The stylesheet format has a short list of rules.

- Each file must have one map at the base layer.
```json
{

}
```
- If the first map entry's key is `"using"`, then the value should be an array of full type names. This array
    should contain full type names for any [`Style`] that has an ambiguous short name (this will happen if there are
    multiple `Reflect` types with the same short name). Note that currently we only support one version of a shortname
    per file.
```json
{
    "using": [
        "crate::my_module::Color",
        "bevy_cobweb_ui::layout::Layout"
    ]
}
```
- All other map keys may either be [`CobwebStyle`] short type names or node path references.
    A style short name is a marker for a style, and is followed by a map containing the serialized value of that style.
    Node path references are used to locate specific styles in the map, and each node should be a map of styles and
    other nodes. The leaf nodes of the overall structure will be styles.
```json
{
    "using": [ "bevy_cobweb_ui::layout::Dims" ],

    "node1": {
        "Dims": {"Percent": [50.0, 50.0]},

        "node2": {
            "Dims": {"Percent": [50.0, 50.0]}
        }
    }
}
```
- A style name may be followed by the keyword `"inherited"`, which means the style value will be inherited from the most
    recent instance of that style below it in the tree. Inheritance is ordering-dependent, so if you don't want a style
    to be inherited, insert it below any child nodes.
```json
{
    "using": [ "bevy_cobweb_ui::layout::Dims" ],

    "node1": {
        "Dims": {"Percent": [50.0, 50.0]},

        "node2": {
            "Dims": "inherited"
        }
    }
}
```
- Node path references may be combined into path segments, which can be used to reduce indentation. If a style is inherited
    in an abbreviated path, it will inherit from the current scope, not its path-parent.
```json
{
    "using": [ "bevy_cobweb_ui::layout::Dims" ],

    "Dims": {"Percent": [25.0, 25.0]},

    "node1": {
        "Dims": {"Percent": [50.0, 50.0]}
    },

    "node1::node2": {
        // This inherits {25.0, 25.0}.
        "Dims": "inherited"
    }
}
```
*/
//TODO: add "MY_CONSTANT_X" references with "constants" section
//TODO: add "imports" section that brings "using" and "constants" sections from other files (track dependencies in
// StyleSheet)
// - warn if there are unresolved dependencies after all initial files have been loaded and handled
#[derive(ReactResource)]
pub struct StyleSheet
{
    /// Tracks styles in all style files.
    styles: HashMap<StyleRef, SmallVec<[ErasedStyle; 4]>>,
    /// Tracks which files have not initialized yet.
    pending: HashSet<StyleFile>,
    /// Tracks the total number of style sheets that should load.
    ///
    /// Used for progress tracking on initial load.
    total_expected_sheets: usize,

    /// Tracks subscriptions to style paths.
    subscriptions: HashMap<StyleRef, SmallVec<[Entity; 1]>>,
    /// Tracks entities for cleanup.
    subscriptions_rev: HashMap<Entity, StyleRef>,

    /// Records entities that need style updates.
    /// - We clear this at the end of every tick, so there should not be stale `ReflectedStyle` values.
    needs_updates: HashMap<TypeId, SmallVec<[(ReflectedStyle, StyleRef, SmallVec<[Entity; 1]>); 1]>>,
}

impl StyleSheet
{
    /// Prepares a stylesheet file.
    fn prepare_file(&mut self, file: StyleFile)
    {
        let _ = self.pending.insert(file.clone());
        self.total_expected_sheets += 1;
    }

    /// Initializes a stylesheet file.
    pub(crate) fn initialize_file(&mut self, file: StyleFile)
    {
        let _ = self.pending.remove(&file);
    }

    /// Gets the stylesheet's loading progress on startup.
    ///
    /// Returns `(num uninitialized files, num total files)`.
    pub fn loading_progress(&self) -> (usize, usize)
    {
        (self.pending.len(), self.total_expected_sheets)
    }

    /// Inserts a style at the specified path if its value will change.
    ///
    /// Returns `true` if this method added any pending subscriber updates.
    pub(crate) fn insert(
        &mut self,
        style_ref: &StyleRef,
        style: ReflectedStyle,
        type_id: TypeId,
        full_type_name: &str,
    ) -> bool
    {
        match self.styles.entry(style_ref.clone()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut vec = SmallVec::default();
                vec.push(ErasedStyle { type_id, style: style.clone() });
                entry.insert(vec);
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                // Insert if the style value changed.
                if let Some(erased_style) = entry.get_mut().iter_mut().find(|e| e.type_id == type_id) {
                    match erased_style.style.equals(&style) {
                        Some(true) => return false,
                        Some(false) => {
                            // Replace the existing value.
                            *erased_style = ErasedStyle { type_id, style: style.clone() };
                        }
                        None => {
                            tracing::error!("failed updating style {:?} at {:?}, its reflected value doesn't implement \
                                PartialEq", full_type_name, style_ref);
                            return false;
                        }
                    }
                } else {
                    entry.get_mut().push(ErasedStyle { type_id, style: style.clone() });
                }
            }
        }

        // Identify entites that should update.
        let Some(subscriptions) = self.subscriptions.get(&style_ref) else { return false };
        if subscriptions.len() == 0 {
            return false;
        }
        let entry = self.needs_updates.entry(type_id).or_default();
        entry.push((style, style_ref.clone(), subscriptions.clone()));

        true
    }

    /// Adds an entity to the tracking context.
    ///
    /// Schedules callbacks that will run to handle pending updates for the entity.
    pub(crate) fn track_entity(
        &mut self,
        entity: Entity,
        style_ref: StyleRef,
        c: &mut Commands,
        callbacks: &StyleLoaderCallbacks,
    )
    {
        // Add to subscriptions.
        // - Note: don't check for duplicates for max efficiency.
        self.subscriptions.entry(style_ref.clone()).or_default().push(entity);
        self.subscriptions_rev.insert(entity, style_ref.clone());

        // Get already-loaded styles that the entity is subscribed to.
        let Some(styles) = self.styles.get(&style_ref) else { return };

        // Schedule updates for each style.
        for style in styles.iter() {
            let type_id = style.type_id;
            self.needs_updates.entry(type_id).or_default().push((
                style.style.clone(),
                style_ref.clone(),
                SmallVec::from_elem(entity, 1),
            ));

            let Some(syscommand) = callbacks.get(type_id) else {
                tracing::warn!("found style at {:?} that wasn't registered as a loadable style", style_ref);
                continue;
            };

            c.add(syscommand);
        }

        // Notify the entity that some of its styles have loaded.
        if styles.len() > 0 {
            c.react().entity_event::<StylesLoaded>(entity, StylesLoaded);
        }
    }

    /// Cleans up despawned entities.
    fn remove_entity(&mut self, dead_entity: Entity)
    {
        let Some(style_ref) = self.subscriptions_rev.remove(&dead_entity) else { return };
        let Some(subscribed) = self.subscriptions.get_mut(&style_ref) else { return };
        let Some(dead) = subscribed.iter().position(|s| *s == dead_entity) else { return };
        subscribed.swap_remove(dead);
    }

    /// Cleans up pending updates that failed to be processed.
    fn cleanup_pending(&mut self)
    {
        if self.needs_updates.len() > 0 {
            // Note: This can technically print spuriously if the user spawns loaded entities in Last and doesn't
            // call `apply_deferred` before the cleanup system runs.
            warn_once!("The style sheet contains pending updates for types that weren't registered. This warning only \
                prints once, and may print spuriously if you spawn loaded entities in Last.");
        }
        self.needs_updates.clear();
    }

    /// Updates entities that subscribed to `T` found at recently-updated style paths.
    pub(crate) fn update_styles<T: LoadableStyle>(
        &mut self,
        mut callback: impl FnMut(Entity, &StyleRef, &ReflectedStyle),
    )
    {
        let Some(mut needs_updates) = self.needs_updates.remove(&TypeId::of::<T>()) else { return };

        for (style, styleref, mut entities) in needs_updates.drain(..) {
            for entity in entities.drain(..) {
                (callback)(entity, &styleref, &style);
            }
        }
    }
}

impl Default for StyleSheet
{
    fn default() -> Self
    {
        Self {
            styles: HashMap::default(),
            pending: HashSet::default(),
            total_expected_sheets: 0,
            subscriptions: HashMap::default(),
            subscriptions_rev: HashMap::default(),
            needs_updates: HashMap::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that enables style loading.
pub(crate) struct StyleSheetPlugin;

impl Plugin for StyleSheetPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_react_resource::<StyleSheet>()
            .add_systems(PreStartup, setup_stylesheet)
            .add_systems(First, load_style_changes)
            .add_systems(Last, cleanup_stylesheet);
    }
}

//-------------------------------------------------------------------------------------------------------------------

//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde_json::Value;

//standard shortcuts
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_stylesheet(sheet_list: Res<StyleSheetList>, mut stylesheet: ReactResMut<StyleSheet>)
{
    // begin tracking expected stylesheet files
    for file in sheet_list.iter_files()
    {
        stylesheet.get_mut_noreact().prepare_file(StyleFile::new(file.as_str()));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn initialize_stylesheet_for_tick(mut stylesheet: ReactResMut<StyleSheet>)
{
    stylesheet.get_mut_noreact().refresh_changed_tracker();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn load_style_changes(
    mut rc         : ReactCommands,
    mut events     : EventReader<AssetEvent<StyleSheetAsset>>,
    sheet_list     : Res<StyleSheetList>,
    mut assets     : ResMut<Assets<StyleSheetAsset>>,
    mut stylesheet : ReactResMut<StyleSheet>,
    types          : Res<AppTypeRegistry>,
){
    if events.is_empty() { return; }

    let mut stylesheet_ref: Option<&mut StyleSheet> = None;
    let type_registry = types.read();

    for event in events.read()
    {
        let id = match event
        {
            AssetEvent::Added{ id } |
            AssetEvent::Modified{ id } => id,
            _ =>
            {
                tracing::debug!("ignoring stylesheet asset event {:?}", event);
                continue;
            },
        };

        let Some(handle) = sheet_list.get_handle(*id)
        else { tracing::warn!("encountered stylesheet asset event {:?} for an untracked asset", id); continue; };

        let Some(asset) = assets.remove(handle)
        else { tracing::error!("failed to remove stylesheet asset {:?}", handle); continue; };

        if stylesheet_ref.is_none() { stylesheet_ref = Some(stylesheet.get_mut(&mut rc)); }
        let stylesheet = stylesheet_ref.unwrap();
        parse_stylesheet_file(&type_registry, stylesheet, asset.file, asset.data);
        stylesheet_ref = Some(stylesheet);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct ErasedStyle
{
    style: Arc<Value>,
    changed: Arc<AtomicBool>,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Reactive resource for managing styles loaded from stylesheet assets.
///
/**
### Stylesheet asset format

Stylesheets are written as JSON files with the extension `.style.json`. You can configure [`StyleSheetPlugin`]
to control how stylesheet files are discovered in the asset directory.

The stylesheet format has a short list of rules.

- Each file must have one map at the base layer.
```json
{
    
}
```
- If the first map entry's key is `"using"`, then the value should be an array of full type names. This array
    should contain full type names for any [`Style`] that has an ambiguous short name (i.e. there are multiple `Reflect`
    types with the same short name). Note that currently we only support one version of a shortname per file.
```json
{
    "using": [
        "crate::my_module::Color",
        "bevy_cobweb_ui::layout::Layout"
    ]
}
```
- All other map keys may either be [`Style`] short type names or node path references. A style short name is a marker for
    a style, and is followed by a map containing the serialized value of that style. Node path references are used to
    locate specific styles in the map, and each node should be a map of styles and other nodes. The leaf nodes of the overall
    structure will be styles.
```json
{
    "using": [ "bevy_cobweb_ui::layout::RelativeLayout" ],

    "node1": {
        "RelativeLayout": {"Center": {"Relative": [50.0, 50.0]}},

        "node2": {
            "RelativeLayout": {"Center": {"Relative": [50.0, 50.0]}}
        }
    }
}
```
- A style name may be followed by the keyword `"inherited"`, which means the style value will be inherited from the most
    recent instance of that style below it in the tree. Inheritance is ordering-dependent, so if you don't want a style
    to be inherited, insert it below any child nodes.
```json
{
    "using": [ "bevy_cobweb_ui::layout::RelativeLayout" ],

    "node1": {
        "RelativeLayout": {"Center": {"Relative": [50.0, 50.0]}},

        "node2": {
            "RelativeLayout": "inherited"
        }
    }
}
```
- Node path references may be combined into path segments, which can be used to reduce indentation. If a style is inherited
    in an abbreviated path, it will inherit from the current scope, not its path-parent.
```json
{
    "using": [ "bevy_cobweb_ui::layout::RelativeLayout" ],

    "RelativeLayout": {"Center": {"Relative": [25.0, 25.0]}}

    "node1": {
        "RelativeLayout": {"Center": {"Relative": [50.0, 50.0]}}
    },

    "node1::node2": {
        // This inherits {25.0, 25.0}.
        "RelativeLayout": "inherited"
    }
}
```
*/
#[derive(ReactResource)]
pub struct StyleSheet
{
    /// Tracks styles in all style files.
    styles: HashMap<StyleFile, HashMap<FullStylePath, ErasedStyle>>,
    /// Tracks which files have not initialized yet.
    pending: HashSet<StyleFile>,
    /// Flag pointing to styles changed in this tick.
    ///
    /// It is set to `false` at the end of the tick and replaced if `Self::change_count > 0`.
    changed: Arc<AtomicBool>,
    /// Counts the number of styles changed in this tick.
    change_count: usize,
}

impl StyleSheet
{
    /// Refreshes the changed files tracker.
    fn refresh_changed_tracker(&mut self)
    {
        if self.change_count == 0 { return; }
        self.changed.store(false, Ordering::Relaxed);
        self.changed = Arc::new(AtomicBool::new(true));
        self.change_count = 0;
    }

    /// Prepares a stylesheet file.
    fn prepare_file(&mut self, file: StyleFile)
    {
        let _ = self.pending.insert(file.clone());
        let _ = self.styles.entry(file).or_insert_with(|| HashMap::default());
    }

    /// Initializes a stylesheet file.
    pub(crate) fn initialize_file(&mut self, file: StyleFile)
    {
        let _ = self.pending.remove(&file);
        let _ = self.styles.entry(file).or_insert_with(|| HashMap::default());
    }

    /// Inserts a style at the specified path if its value will change.
    pub(crate) fn insert(&mut self, file: &StyleFile, path: FullStylePath, style: Arc<Value>)
    {
        let file = self.styles.get_mut(&file).expect("file should have been initialized");

        let mut inserter = ||
        {
            self.change_count += 1;
            ErasedStyle{ style: style.clone(), changed: self.changed.clone() }
        };

        match file.entry(path)
        {
            std::collections::hash_map::Entry::Vacant(entry) =>
            {
                entry.insert(inserter());
            }
            std::collections::hash_map::Entry::Occupied(mut entry) =>
            {
                // Check if the style changed.
                if *entry.get().style == *style { return; }
                entry.insert(inserter());
            }
        }
    }

    /// Gets the stylesheet's loading progress on startup.
    ///
    /// Returns `(num uninitialized files, num total files)`.
    pub fn loading_progress(&self) -> (usize, usize)
    {
        (self.pending.len(), self.styles.len())
    }

    /// Gets a style.
    ///
    /// If `ignore_unchanged` is set to `true` then if the style did not change this tick, it won't be deserialized,
    /// and `None` will be returned.
    pub fn get<S>(&self, style_ref: &FullStyleRef, ignore_unchanged: bool) -> Option<S>
    where
        S: CobwebStyle
    {
        // Access the style.
        let Some(style_file) = self.styles.get(&style_ref.file)
        else
        {
            tracing::error!("could not load style {:?} at path {:?}, file {:?} was not found",
                style_ref.path.full_type_name, style_ref.path.path, style_ref.file);
            return None;
        };
        let Some(erased_style) = style_file.get(&style_ref.path)
        else
        {
            // Don't error if the reference file is waiting to be initialized.
            if self.pending.contains(&style_ref.file)
            {
                tracing::trace!("ignored style load request for style {:?} at path {:?}, file {:?} still loading",
                    style_ref.path.full_type_name, style_ref.path.path, style_ref.file);
            }
            else
            {
                tracing::error!("could not load style {:?}, it was not found at path {:?} in file {:?}; \
                    maybe the path is wrong",
                    style_ref.path.full_type_name, style_ref.path.path, style_ref.file);
            }
            return None;
        };

        // Check if this style was changed this tick.
        // - The caller can skip this check if they need to get an initial value for this style.
        if ignore_unchanged
        {
            if !erased_style.changed.load(Ordering::Relaxed) { return None; }
        }

        // Deserialize the style.
        match serde_json::from_value((*erased_style.style).clone())
        {
            Ok(style) => Some(style),
            Err(err) =>
            {
                let temp = S::default();
                let hint = serde_json::to_string(&temp).unwrap();
                tracing::error!("failed deserializing style {:?} at path {:?} in file {:?}, {:?}\n\
                    serialization hint: {:?}",
                    style_ref.path.full_type_name, style_ref.path.path, style_ref.file, err, hint);
                None
            }
        }
    }
}

impl Default for StyleSheet
{
    fn default() -> Self
    {
        Self{
            styles: HashMap::default(),
            pending: HashSet::default(),
            changed: Arc::new(AtomicBool::new(true)),
            change_count: 0,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin that enables style loading.
pub struct StyleSheetPlugin;

impl Plugin for StyleSheetPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(StyleAssetLoaderPlugin)
            .init_react_resource::<StyleSheet>()
            .add_systems(PreStartup, setup_stylesheet)
            .add_systems(First,
                (
                    initialize_stylesheet_for_tick,
                    load_style_changes,
                )
                    .chain()
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

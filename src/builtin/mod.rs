#[cfg(feature = "assets")]
pub mod assets;

#[cfg(feature = "colors")]
pub mod colors;

#[cfg(feature = "widgets")]
pub mod widgets;

mod plugin;

pub(crate) use plugin::*;

//-------------------------------------------------------------------------------------------------------------------

/// Loads an embedded widget.
///
/// Example:
/*
```rust
// Macro call:
load_embedded_asset!(app, "bevy_cobweb_ui", "src/builtin/widgets/radio_button", "radio_button.caf.json");

// Expands to:
embedded_asset!(app, "src/builtin/widgets/radio_button", "radio_button.caf.json");
app.load(concat!("embedded://", "bevy_cobweb_ui", "/", "radio_button.caf.json"));
```
*/
#[macro_export]
macro_rules! load_embedded_scene_file {
    ($app: ident, $crate_name: expr, $source_path: expr, $widget_file: expr) => {{
        use crate::prelude::LoadedCobwebAssetFilesAppExt;
        bevy::asset::embedded_asset!($app, $source_path, $widget_file);
        $app.load(concat!("embedded://", $crate_name, "/", $widget_file));
    }};
}

//-------------------------------------------------------------------------------------------------------------------

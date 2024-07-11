//! Built-in widgets
//!
//! Currently implemented:
//! - `radio_button`: Includes [`RadioButtonBuilder`](radio_button::RadioButtonBuilder) for making radio buttons,
//!   and [`RadioButtonManager`](radio_button::RadioButtonManager) for coordinating button selection.

pub mod radio_button;

mod plugin;
pub(crate) use plugin::*;

//-------------------------------------------------------------------------------------------------------------------

/// Loads an embedded widget.
///
/// Example:
/*
```rust
// Macro call:
load_embedded_asset!(app, "bevy_cobweb_ui", "src/widgets/radio_button", "radio_button.caf.json");

// Expands to:
embedded_asset!(app, "src/widgets/radio_button", "radio_button.caf.json");
app.load(concat!("embedded://", "bevy_cobweb_ui", "/", "radio_button.caf.json"));
```
*/
#[macro_export]
macro_rules! load_embedded_widget {
    ($app: ident, $crate_name: expr, $source_path: expr, $widget_file: expr) => {{
        bevy::asset::embedded_asset!($app, $source_path, $widget_file);
        $app.load(concat!("embedded://", $crate_name, "/", $widget_file));
    }};
}

//-------------------------------------------------------------------------------------------------------------------

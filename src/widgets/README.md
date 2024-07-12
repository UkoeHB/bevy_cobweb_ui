Instructions for adding a new embedded widget:

- Add a `WIDGET_NAME.caf.json` file to the widget directory.
- Add `"#manifest": { "", "builtin.widgets.WIDGET_NAME" }` to the `WIDGET_NAME.caf.json` file.
- Add `load_embedded_widget!(app, "bevy_cobweb_ui", "src/widgets/WIDGET_NAME", "WIDGET_NAME.caf.json");` to the plugin.
- Add a docs entry to `src/widgets/mod.rs` for the new widget.

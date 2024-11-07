# Bevy Cobweb UI

A framework for building UI and managing assets in a `bevy` app. Built on [bevy_cobweb](https://github.com/UkoeHB/bevy_cobweb) and `bevy_ui`/`bevy_assets`/etc.


## Features

- [Custom asset format](bevy_cobweb_ui::loading) for specifying scenes and loading commands to be applied on startup, with seamless fine-grained hot reloading and thorough error handling.
- Powerful extension-based reactivity API for adding logic to UI scenes (e.g. `.on_pressed(your_system)`).
- Standardized API for accessing fonts with families, weights, styles, and widths (e.g. `Fira Sans + Bold + Italic + Condensed`).
- Robust [localization](bevy_cobweb_ui::localization) support for text, fonts, images, and audio, with extensibility to other assets.
- [Asset manager resources](bevy_cobweb_ui::assets_ext) that keep track of asset handles, take care of localization automatically, and are easily populated using asset manifests specified in cobweb asset files.
- [Wrappers](bevy_cobweb_ui::ui_bevy) around `bevy_ui` for loading UI into scenes via cobweb asset files.
- [Built-in](bevy_cobweb_ui::builtin) UI widgets, color palettes, and assets (e.g. fonts). Note that the `widgets` and `colors` features are enabled by default.


## Getting Started

1. (Optional) Install syntax highlighting for the CAF asset format.
    - [VSCode](https://github.com/UkoeHB/vscode-caf/)
    - [SublimeText](https://github.com/UkoeHB/sublime-caf/)
1. Add [`CobwebUiPlugin`](bevy_cobweb_ui::prelude::CobwebUiPlugin).
1. Load a CAF file if you have one. Usually these are stored in your assets directory.
1. Wait until in state `LoadState::Done` before loading UI. This avoids jank while loading CAF files and other assets. You can build UI in-code before then without a problem, as long as you don't reference not-yet-loaded assets.

```rust
app
    .add_plugins(bevy::DefaultPlugins)
    .add_plugins(CobwebUiPlugin)
    .load("main.caf")
    .add_systems(OnEnter(LoadState::Done), build_ui);
```

Check the [`loading`](bevy_cobweb_ui::loading) module for how to write CAF files.

Check the repository examples for how to build different kinds of UI.


## Examples

**NOTICE**: Many examples are not yet migrated to use CAF, which is still in development to reach feature parity with the previous JSON format.

- [`hello_world`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/hello_world): Bare-bones hello world.
- [`counter`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter): Simple counter button. Shows how [`ControlRoot`](bevy_cobweb_ui::prelude::ControlRoot) and [`ControlLabel`](bevy_cobweb_ui::prelude::ControlLabel) can be used to transfer interactions within a widget. Also demonstrates updating text dynamically on the code side.
- [`counter_widget`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter_widget): Widget-ified counter that can be configured. Uses scene 'specs' to make the widget scene data parameterized, enabling customization within asset files.
- [`help_text`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/help_text): Help text that appears on hover. Showcases [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity), which allows controlling (and animating) the opacity of entire node trees, and even layering multiple [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity) within a single tree.
- [`radio_buttons`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/radio_buttons): A set of buttons where only one is selected at a time. Uses the built-in radio button widget.
- [`localization`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/localization): Showcases localized text and font.
- [`calculator`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/calculator): A minimalistic code-only calculator. Shows how to mix normal `sickle_ui` UI construction with `bevy_cobweb_ui` convenience tools for interactions.
- [`game_menu`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/game_menu): A simple game menu with settings page. Showcases multiple uses of built-in radio buttons, localization, non-interactive animations, integration with `sickle_ui` built-in widgets (a slider and drop-down), and how to manage localized image assets using CAF files as manifests.


## `bevy` compatability

| `bevy` | `bevy_cobweb_ui` |
|-------|-------------------|
| 0.14  | 0.1.0 - main      |

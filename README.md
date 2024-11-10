# Bevy Cobweb UI

A framework for building UI and managing assets in a `bevy` app.

Depends on [bevy_cobweb](https://github.com/UkoeHB/bevy_cobweb), `bevy_ui`. and `bevy_assets`.


## Features

- [Custom scene format](bevy_cobweb_ui::loading) called COB
- [Font family](bevy_cobweb_ui::prelude::FontRequest) API
- [Localization](bevy_cobweb_ui::localization) framework (text, fonts, images, audio)
- [Asset management](bevy_cobweb_ui::assets_ext) tools
- [Built-in](bevy_cobweb_ui::builtin) UI widgets and color palettes
- And many small quality of life features supporting UI development.


## Getting Started

1. *(Optional)* Install syntax highlighting for the COB asset format.
    - [VSCode](https://github.com/UkoeHB/vscode-cob/)
    - [vim](https://github.com/UkoeHB/vim-cob/)
    - [SublimeText](https://github.com/UkoeHB/sublime-cob/)
1. Add [`CobwebUiPlugin`](bevy_cobweb_ui::prelude::CobwebUiPlugin).
1. Load a COB file if you have one. Usually these are stored in your assets directory.
1. Wait until in state `LoadState::Done` before loading UI. This avoids jank while loading COB files and other assets. You can build UI in-code before then without a problem, as long as you don't reference not-yet-loaded assets.

```rust
app
    .add_plugins(bevy::DefaultPlugins)
    .add_plugins(CobwebUiPlugin)
    .load("main.cob")
    .add_systems(OnEnter(LoadState::Done), build_ui);
```

Check the [`loading`](bevy_cobweb_ui::loading) module for how to write COB files.

Check the repository examples for how to build different kinds of UI.


## Examples

**NOTICE**: Many examples are not yet migrated to use COB, which is still in development to reach feature parity with the previous JSON format.

- [`hello_world`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/hello_world): Bare-bones hello world.
- [`counter`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter): Simple counter button. Shows how [`ControlRoot`](bevy_cobweb_ui::prelude::ControlRoot) and [`ControlLabel`](bevy_cobweb_ui::prelude::ControlLabel) can be used to transfer interactions within a widget. Also demonstrates updating text dynamically on the code side.
- [`counter_widget`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter_widget): Widget-ified counter that can be configured. Uses scene 'specs' to make the widget scene data parameterized, enabling customization within asset files.
- [`cursors`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/cursors): Set custom cursors that respond to interactions with UI elements.
- [`help_text`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/help_text): Help text that appears on hover. Showcases [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity), which allows controlling (and animating) the opacity of entire node trees, and even layering multiple [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity) within a single tree.
- [`radio_buttons`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/radio_buttons): A set of buttons where only one is selected at a time. Uses the built-in radio button widget.
- [`localization`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/localization): Showcases localized text and font.
- [`calculator`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/calculator): A minimalistic code-only calculator. Shows how to mix normal `sickle_ui` UI construction with `bevy_cobweb_ui` convenience tools for interactions.
- [`game_menu`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/game_menu): A simple game menu with settings page. Showcases multiple uses of built-in radio buttons, localization, non-interactive animations, integration with `sickle_ui` built-in widgets (a slider and drop-down), and how to manage localized image assets using COB files as manifests.


## `bevy` compatability

| `bevy` | `bevy_cobweb_ui` |
|-------|-------------------|
| 0.14  | 0.1.0 - main      |

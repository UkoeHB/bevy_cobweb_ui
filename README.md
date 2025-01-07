# Bevy Cobweb UI

A UI and asset-management framework for the `bevy` game engine.

Depends on `bevy` and [bevy_cobweb](https://github.com/UkoeHB/bevy_cobweb).


## Features

- [Custom scene format](bevy_cobweb_ui::loading) called COB
- [Localization](bevy_cobweb_ui::localization) framework (text, fonts, images, audio)
- [Font family](bevy_cobweb_ui::prelude::FontRequest) API
- [Built-in](bevy_cobweb_ui::builtin) UI widgets and color palettes
- [Asset management](bevy_cobweb_ui::assets_ext) tools
- And many quality of life features.


## Getting Started

Check out the `bevy_cobweb_ui` [book](https://ukoehb.github.io/cobweb_book/), which is a guide tailored to new users.

*(Optional)* Install syntax highlighting for the COB asset format.
- [VSCode](https://github.com/UkoeHB/vscode-cob/)
- [vim](https://github.com/UkoeHB/vim-cob/)
- [SublimeText](https://github.com/UkoeHB/sublime-cob/)

Check the loading module [docs](bevy_cobweb_ui::loading) for how to write COB files. COB files can be hot reloaded with the `hot_reload` feature. Hot-reloaded changes will cause affected scene nodes to be refreshed (or cause commands to be re-applied). Hot-reloading is minimally destructive. Entities are only despawned when you delete scene nodes from a COB file.

Check the repository examples for how to build different kinds of UI.


## Examples

- [`hello_world`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/hello_world): Bare-bones hello world.
- Built-in widgets:
    - [`checkbox`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/checkbox)
    - [`radio_buttons`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/radio_buttons)
    - [`slider`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/slider)
    - [`scroll`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/scroll)
- [`cursors`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/cursors): Set custom cursors that respond to interactions with UI elements.
- [`fonts`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/fonts): Register new fonts and use them to set text.
- [`help_text`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/help_text): Help text that appears on hover. Showcases [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity), which allows controlling (and animating) the opacity of entire node trees, and even layering multiple [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity) within a single tree.
- [`localization`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/localization): Showcases localized text and font.
- [`calculator`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/calculator): A basic calculator. Shows how to use grid layout and scene macros.
- [`counter`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter): Simple counter button. Shows how [`ControlRoot`](bevy_cobweb_ui::prelude::ControlRoot) and [`ControlMember`](bevy_cobweb_ui::prelude::ControlMember) can be used to transfer interactions within a widget. Also demonstrates updating text dynamically on the code side.
- [`game_menu`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/game_menu): A simple game menu with settings page. Showcases multiple uses of built-in radio buttons, sliders, and drop-downs, localization, non-interactive animations, and how to manage localized image assets using COB files as asset manifests.
    - Not yet migrated to use COB. It is waiting for a dropdown widget to be implemented.
- [`editor_demo`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/editor_demo): Showcases the editor with custom editor widgets.


## Editor

There is an editor, enabled by the `editor` feature. It is currently a very basic proof of concept, and may or may not be developed further. See the `editor_demo` example.


## `bevy` compatability

| `bevy` | `bevy_cobweb_ui` |
|-------|-------------------|
| 0.15  | 0.5.0 - main      |
| 0.14  | 0.1.0 - 0.4.1     |

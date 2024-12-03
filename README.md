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

1. *(Optional)* Install syntax highlighting for the COB asset format.
    - [VSCode](https://github.com/UkoeHB/vscode-cob/)
    - [vim](https://github.com/UkoeHB/vim-cob/)
    - [SublimeText](https://github.com/UkoeHB/sublime-cob/)

2. Add [`CobwebUiPlugin`](bevy_cobweb_ui::prelude::CobwebUiPlugin) to your app.

```rust
app
    .add_plugins(bevy::prelude::DefaultPlugins)
    .add_plugins(CobwebUiPlugin);
```

3. Add a COB file to your `assets` directory. Use the `.cob` file extension.

```rust
#scenes
"hello"
    TextLine{ text: "Hello, World!" }
```

4. Load the COB file to your app in a plugin.

```rust
app.load("main.cob");
```

You can load other COB files recursively using `#manifest` sections (see the loading module [docs](bevy_cobweb_ui::loading)).

5. Add a system for spawning a scene.

```rust
fn build_ui(mut commands: Commands, mut s: ResMut<SceneLoader>)
{
    // Spawns entities
    commands.ui_root().load_scene(("main.cob", "hello"), &mut s);
}
```

6. Add the system to your app.

```rust
app.add_systems(OnEnter(LoadState::Done), build_ui);
```

We put the system in `OnEnter(LoadState::Done)` so it runs after all COB files and assets loaded into this crate's [asset managers](bevy_cobweb_ui::assets_ext) have been loaded.

Check the loading module [docs](bevy_cobweb_ui::loading) for how to write COB files. COB files can be hot reloaded with the `hot_reload` feature. Hot-reloaded changes will cause affected scene nodes to be refreshed (or cause commands to be re-applied). Hot-reloading is minimally destructive. Entities are only despawned when you delete scene nodes from a COB file.

Check the repository examples for how to build different kinds of UI.


## Examples

**NOTICE**: Some examples are not yet migrated to use COB, which is still in development to reach feature parity with the previous JSON format.

- [`hello_world`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/hello_world): Bare-bones hello world.
- [`counter`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter): Simple counter button. Shows how [`ControlRoot`](bevy_cobweb_ui::prelude::ControlRoot) and [`ControlMember`](bevy_cobweb_ui::prelude::ControlMember) can be used to transfer interactions within a widget. Also demonstrates updating text dynamically on the code side.
- [`counter_widget`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter_widget) (*not migrated*): Widget-ified counter that can be configured. Uses scene 'specs' to make the widget scene data parameterized, enabling customization within asset files.
- [`cursors`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/cursors): Set custom cursors that respond to interactions with UI elements.
- [`help_text`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/help_text): Help text that appears on hover. Showcases [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity), which allows controlling (and animating) the opacity of entire node trees, and even layering multiple [`PropagateOpacity`](bevy_cobweb_ui::prelude::PropagateOpacity) within a single tree.
- [`radio_buttons`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/radio_buttons): A set of buttons where only one is selected at a time. Uses the built-in radio button widget.
- [`slider`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/slider): Uses the built-in slider widget.
- [`localization`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/localization) (*not migrated*): Showcases localized text and font.
- [`calculator`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/calculator): A minimalistic code-only calculator. Shows how to mix builder-pattern-based UI construction with `bevy_cobweb_ui` convenience tools for interactions.
- [`game_menu`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/game_menu) (*not migrated*): A simple game menu with settings page. Showcases multiple uses of built-in radio buttons, sliders, and drop-downs, localization, non-interactive animations, and how to manage localized image assets using COB files as asset manifests.
- [`editor_demo`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/editor_demo): Showcases the editor with custom editor widgets.


## Editor

There is an editor, enabled by the `editor` feature. It is currently a very basic proof of concept, and may or may not be developed further. See the `editor_demo` example.


## `bevy` compatability

| `bevy` | `bevy_cobweb_ui` |
|-------|-------------------|
| 0.15  | 0.5.0 - main      |
| 0.14  | 0.1.0 - 0.4.1     |

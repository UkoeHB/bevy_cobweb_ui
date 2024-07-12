# Bevy Cobweb UI

**This crate is under development (v0.1.0 release SOON).**

Provides a framework for building UI and managing assets in a Bevy app. Built on [bevy_cobweb](https://github.com/UkoeHB/bevy_cobweb), [sickle_ui](https://github.com/UmbraLuminosa/sickle_ui), and standard `bevy_ui`/`bevy_assets`/etc.


## Features

- Custom asset format for specifying scenes and loading commands to be applied on startup, with seamless fine-grained hot reloading and thorough error handling. See the [`loading`](bevy_cobweb_ui::loading) module.
- Integration with [sickle_ui](https://github.com/UmbraLuminosa/sickle_ui) so widgets and themes can be specified in cobweb asset files then easily overridden/customized. Also includes various reactivity extensions for `UiBuilder`, including UI interactions (e.g. `.on_pressed(your_system)`). See the [`sickle_ext`](bevy_cobweb_ui::sickle_ext) module.
- Robust localization support for text, fonts, images, and audio, with extensibility to other assets. See the [`localization`](bevy_cobweb_ui::localization) module.
- Asset manager resources that keep track of asset handles, take care of localization automatically, and are easily populated using asset manifests specified in cobweb asset files. See the [`assets_ext`](bevy_cobweb_ui::assets_ext) module.
- Wrappers around `bevy_ui` for loading UI into scenes via cobweb asset files. See the [ui_bevy](bevy_cobweb_ui::ui_bevy) module.
- Built-in UI widgets. See the [widgets](bevy_cobweb_ui::widgets) module. Note that the `widgets` feature is enabled by default.


## Getting Started

Check out the [`hello_world`](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/hello_world) example in the repository.


## `bevy` compatability

| `bevy` | `bevy_cobweb_ui` |
|-------|-------------------|
| 0.14  | 0.1.0 - master    |

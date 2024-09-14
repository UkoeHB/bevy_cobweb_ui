# Changelog

## WIP


## 0.4.1

- Automatically add `SickleUiPlugin` and `ReactPlugin` if missing.

## 0.4.0

- Add importing via manifest key to cobweb asset file syntax.
- Move built-in widgets and assets to `builtin` module.
- Add built-in colors mimicking Bevy's color palettes (CSS1 'basic', CSS4 'css', Tailwind CSS 'tailwind').
- Rework widget theming and interactivity to use simpler control scheme. Now you only need `ControlRoot` and `ControlLabel` loadables to set up multi-entity interactive and pseudo-state-sensitive structures.
- Add font families API for accessing fonts.
- Implement `Add` for `SceneFile` and `SceneRef`.
- Add `PropagateOpacity` for controlling opacity of hierarchies. Added `hover_text` example to showcase it.

## 0.3.0

- Update to `bevy_cobweb` v0.11.
- Rename `LoadableFile`/`LoadablePath`/`LoadableRef` to `SceneFile`/`ScenePath`/`SceneRef`.
- Fix bug with hot-reloading scene nodes not taking into account non-loaded sibling entities in the entity hierarchy.
- Optimize importing constants in cobweb asset files.
- Optimize importing specs in cobweb asset files.

## 0.2.0

- Register `DisplayControl` for reflection.
- Refactor `ApplyLoadable` to take `Entity` and `&mut World` instead of `EntityCommands`. This should be a small optimization.
- Split `LoadFonts`, `LoadAudio`, and `LoadImages` into `LoadX`/`LoadLocalizedX` pattern. These asset maps will no longer attempt to load localized assets until the `LocalizationManifest` has negotiated a language list.

## 0.1.1

- Add `JustifyText` and `BreakLineOn` options to `TextLine`.

## 0.1.0

- Initial release.

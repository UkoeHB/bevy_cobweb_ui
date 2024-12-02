# Changelog

## WIP

## 0.5.1

- Add `SliderValue::single` and `SliderValue::planar` helpers.
- Rename `AnimatableAttribute` to `AnimatedAttribute`.
- Add derives for setting up animated components and newtype components: `StaticComponent`, `ResponsiveComponent`, `AnimatedComponent`, `StaticReactComponent`, `ResponsiveReactComponent`, `AnimatedReactComponent`, `StaticNewtype`, `ResponsiveNewtype`, `AnimatedNewtype`, `StaticReactNewtype`, `ResponsiveReactNewtype`, `AnimatedReactNewtype`.
- Bugfix: internal panic when there is an invalid loadable in a scene node.
- Refactor control attributes so they are stored on target entities. You can use the `NodeAttributes` component to add/remove/modify attributes at runtime.
- Add `AnyClone` trait that mirrors `Any` but allows cloning the underlying type. Also bounded by `Debug + Send + Sync + 'static` for convenience.

## 0.5.0

- `LoadedImageNode::image` is now optional. If no image is specified, a default handle will be inserted.
- Add radio button widget. Use the `RadioButtonGroup` and `RadioButton` loadables to set up radio buttons. See the `radio_buttons` example.
- Remove `#using` section from COB files. All loadable shortnames must be uniquely registered.
- Tighten trait bounds on `Loadable`. It now requires `Reflectable` instead of `Reflect`.
- Fix control group attribute resolution so attributes of the same type (but associated with different pseudo states) won't be layered on top of each other.
- Refactor the `StaticAttribute`/`ResponsiveAttribute`/`AnimatedAttribute` traits so value extraction can be customized.
- Adjust field names of `AnimatedVals` and `AnimationSettings` to match the `Animated` instruction.
- Add `UpdateId` system input type (like `In` or `Trigger`). This is used in `.update()` and `.update_on()` now instead of layered closures.
- Add 'anonymous control groups' which enables using PseudoStates without ControlRoot/ControlLabel if you only need to target one entity.
- Add `NodeShadow` instruction loadable that inserts bevy's `BoxShadow` component. The shadow can be animated.
- Rename `LoadedUiImage`/`UiImageColor`/`UiImageIndex` to `LoadedImageNode`/`ImageNodeColor`/`ImageNodeIndex`.
- Add slider widget. Use the `Slider` and `SliderHandle` loadables to set up a slider. See the `slider` example.
- Update to `bevy` v0.15.0.

## 0.5.0-rc.3

- Replace JSON-based asset format with custom Cobweb Asset Format (COB).
- Many updates and improvements throughout the crate.
- Remove `sickle_ui` dependency. We now vendor a subset of the sickle functionality as subcrates.
- Update to Bevy 0.15.
- Add editor proof of concept.

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

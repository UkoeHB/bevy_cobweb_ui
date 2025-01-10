# Changelog

## WIP

### Highlights
- Auto-apply `Interactive` when adding interactive callbacks like `.on_pressed`. Users no longer need to add `Interactive` unless they have a custom use for it.

### Updates
- `SceneBuilder` is now a system param instead of resource. `ResMut<SceneBuilder>` -> `SceneBuilder`
- Add `.reactor()` extension method for when `.update_on` is overkill.
- Rename `UpdateId` to `TargetId`.

## 0.9.0

### Highlights
- Rename `load` semantics to `spawn` and `build`.
    - `load_scene` -> `spawn_scene`
    - `load_scene_and_edit` -> `spawn_scene_and_edit`
    - `SceneLoader` -> `SceneBuilder`
    - `LoadedScene` -> `SceneHandle`
- Add `fonts` example.

### Updates
- Add `UiSceneHandle` and `EcsSceneHandle` type aliases to simplify passing `SceneHandle<UiBuilder<Entity>>` and `SceneHandle<EntityCommands>` as function parameters.
- Rename `ControlBuilderExt::edit_child` -> `ControlBuilderExt::edit_control_child`


## 0.8.0

### Hightlights
- Add full support for grid layouts.
    - Added `GridNode` and `AbsoluteGridNode` loadables.
    - Added `fr` builtin type for `GridVal::Fraction`.
    - Added deserialization magic so `RepeatedGridVal` can be obtained from a single `GridVal`. This lets you do `GridNode{ grid_template_rows:[10px 20fr] }` for example.

### Updates
- Rework calculator example to use a COB file with the new grid layout support.
- Implement `Instruction` for `GlobalZIndex`.


## 0.7.0

### Highlights
- Add scene macros to COB files. Scene macros are re-usable scene fragments whose contents can be overridden.
- Add checkbox widget with new `checkbox` example.

### Updates
- Improve deserialization errors for COB loadables.

### Fixes
- Treat carriage returns `\r` as plain whitespace.


## 0.6.0

### Highlights
- Update README to point to the new cobweb ui book!
- Integrate `ReactorResult` from `bevy_cobweb`. All built-in callbacks like `.on_pressed` now let you early-out with `?`, so long as you return `OK` (warns on error) or `DONE` (drops errors).
- Add scroll view widget with new `scroll` example.

### Updates
- Rename `ControlLabel` to `ControlMember`.
- Rename `RadioButtonGroup` to `RadioGroup`.
- Update `ControlRoot` and `ControlMember` to create anonymous labels if an empty string is set for the ID.
- Update `ControlRoot` and `ControlMember` to be structs instead of newtypes. This allows eliding the container when using anonymous ids.
- Remove `Copy` derive from `AnimationSettings`, `AnimationConfig`, and `LoopedAnimationConfig` to avoid silent copies causing issues when editing values.
- Expand API of `NodeAttributes` component to make editing attributes easier.
- Validate loadable names on registration. Only named structs that start uppercase are allowed.
- Simplify `AnimationConfig` and `LoopedAnimationConfig` definitions. Allow the `duration` field to be ignored (e.g. if you only want the delay).
- Add auto-value-extraction for enter animations.
    - Add `AnimatedAttribute::get_value` trait method.
    - Add `Splattable::splat_value` trait method.
    - Rename `Animated::enter_ref` to `Animated::enter_ref_override`. If the override is not set, then `AnimatedAttribute::get_value` will be used to extract the current value for an animation when entering a new state.
- Remove `AnyClone` trait.
- Simplify `FlexNode`/`AbsoluteNode`/`DisplayControl`.
    - These are no longer `ReactComponents`.
    - `DisplayControl` is now a normal component.
    - Removed `WithAbsoluteNode` and `WithFlexNode`.
- Add `SetClipMargin` loadable.
- Re-enable `remove_with_requires` in instruction reversion now that the bevy bug is fixed.
- Update localization example to the COB format.
- Add `.update_text()` extension method for UiBuilder to simplify updating a TextLine with static text.
- Add `Picking` instruction loadable for inserting the `PickingBehavior` component.
- Add `Visibility` component loadable.
- Add hierarchy traversal tools.
- Remove extra lifetime parameter in `LoadedScene`.
- Add `BoxShadow` to `PropagateOpacity`.
- Rename `App` extension method `register_themed` -> `register_static`.
- Rename `DisplayControl::Display` -> `DisplayControl::Show`.
- Improve node cleanup on hot-reloaded removal.
- Remove `id` field from `ControlRoot`. The `responds_to` field in `Responsive` and `Animated` is only for responding to non-root entities in the control group.
- Adjust `PseudoStateParam` API for consistency with other APIs.
- `Enable` and `Disable` entity events now add/remove `FluxInteraction::Disabled`.

### Fixes
- Cursor compile error on WASM.
- `on_pointer_enter` now works properly instead of being an alias for `on_pressed`
- Avoid panicking when accessing UiBuilder if the entity doesn't exist.


## 0.5.1

### Highlights
- Refactor control attributes so they are stored on target entities. You can use the `NodeAttributes` component to add/remove/modify attributes at runtime.

### Updates
- Add `SliderValue::single` and `SliderValue::planar` helpers.
- Rename `AnimatableAttribute` to `AnimatedAttribute`.
- Add derives for setting up animated components and newtype components: `StaticComponent`, `ResponsiveComponent`, `AnimatedComponent`, `StaticReactComponent`, `ResponsiveReactComponent`, `AnimatedReactComponent`, `StaticNewtype`, `ResponsiveNewtype`, `AnimatedNewtype`, `StaticReactNewtype`, `ResponsiveReactNewtype`, `AnimatedReactNewtype`.
- Add `AnyClone` trait that mirrors `Any` but allows cloning the underlying type. Also bounded by `Debug + Send + Sync + 'static` for convenience.

### Fixes
- Internal panic when there is an invalid loadable in a scene node.

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

# Changelog

## WIP

- Update to `bevy_cobweb` v0.11.

## 0.2.0

- Register `DisplayControl` for reflection.
- Refactor `ApplyLoadable` to take `Entity` and `&mut World` instead of `EntityCommands`. This should be a small optimization.
- Split `LoadFonts`, `LoadAudio`, and `LoadImages` into `LoadX`/`LoadLocalizedX` pattern. These asset maps will no longer attempt to load localized assets until the `LocalizationManifest` has negotiated a language list.

## 0.1.1

- Add `JustifyText` and `BreakLineOn` options to `TextLine`.

## 0.1.0

- Initial release.

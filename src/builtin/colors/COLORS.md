Manifest keys of embedded COB files:

- `builtin.colors.basic`: See [`bevy::color::palettes::basic`](bevy::color::palettes::basic). Example use: `$AQUA`.
- `builtin.colors.css`: See [`bevy::color::palettes::css`](bevy::color::palettes::css). Example use: `$ALICE_BLUE`.
- `builtin.colors.tailwind`: See [`bevy::color::palettes::tailwind`](bevy::color::palettes::tailwind). Example use: `$AMBER_500`.
- `builtin.colors`: Re-exports color palettes with appropriate aliases (`basic`, `css`, and `tailwind`). Examples: `$basic::AQUA`, `$css::ALICE_BLUE`, `$tailwind::AMBER_500`.

Example:

```rust
// my_project/assets/main.cob
#import
builtin.colors as colors

#scenes
"hello"
    TextLine{ text: "Hello, World!" }
    BackgroundColor($colors::tailwind::EMERALD_600)
```

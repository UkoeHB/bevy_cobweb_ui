Attempt at a semi-formal specification for COB - WIP

Sub-crate of [`bevy_cobweb_ui`](https://github.com/UkoeHB/bevy_cobweb_ui).

## Global

Comments, whitespace, and filler characters
- Comments
    - Line comments: `// ... \n`
    - Block comments: `\* ... *\`
- Whitespace: ` `, `\n`
- Filler characters: `,` `;`

Banned characters outside string values and comments
- non-basic ASCII whitespace: `\b`, `\f`, `\r`, `\t`
- any unicode character


## `manifest`

self as {manifest key}
{file} as {manifest key}

### Manifest key

{id}
{id}.{id}.{id}


## `import`

{manifest key} as _
{manifest key} as {alias}

implicit: std as _
    - can override, e.g. 'std as std'

### Alias

{id}


## `defs`

Definitions
    Value constants
        `${name} = {value}`
        `${name} = \ .. values .. \`
    Value macros
        - 
    Loadable macros
    Scene macros

Invocations
    Value constants
        `${name}`
        `${import::alias::path::to::}{name}`
    Value macros
        - 
    Loadable macros
    Scene macros


## `commands`

Loadables


## `scenes`

Scene layers
- Layer name
    - string value
- Layer stacking
    - add layer stack if encounter layer name >= 2 spaces deeper than current layer
    - if encounter layer name >= 2 spaces shallower than current layer, pop layers until find nearest parent layer at same level
- Layer contents
    - Loadables
    - Loadable macros
    - Scene macros
    - New layers


## Value model

Loadable values appear in COB files very similar to how they appear in Rust. Since COB is minimalist, there are several simplifications and details to note.

### Comments

Single-line and block comments are both supported, the same as rust.

```rust
// Single-line
/*
Multi-line
*/
```

### Spacing instead of punctuation

Commas and semicolons are treated as whitespace. They are completely optional, even inside type generics.

Since we don't use commas to separate items, we have one important whitespace rule.

**Important whitespace rule**: All loadables and enum variants with data must have no whitespace between their name (or generics) and their data container (`{}`, `[]`, or `()`).

Example (rust):
```rust
let s = MyStruct::<A, B<C, D>> { a: 10.0, b: true };
```

Example (COB):
```rust
MyStruct<A B<C D>>{ a: 10 b: true }
// No space here..^..
```

This rule lets us differentiate between a list of 'unit struct/variant, map/array/tuple' and a single 'struct/variant with data'.

### Keywords banned from structs

Built-in keywords cannot be used as struct field names.
- `true`/`false`: Boolean true/false.
- `inf`/`-inf`/`nan`: Float values.
- `none`: Corresponds to rust `None`.
- `auto`: Corresponds to enum variant `Val::Auto` (used in UI, see built-in types below).

**`Option<T>`**

If a rust type contains `Option<T>`, then `Some(T)` is elided to `T`, and `None` is represented by the `none` keyword.

Example (rust):
```rust
#[derive(Reflect, Default, PartialEq)]
struct MyStruct
{
    a: Option<u32>,
    b: Option<bool>
}

let s = MyStruct{ a: Some(10), b: None };
```

Example (COB):
```rust
MyStruct{ a:10 b:none }
```

### Type name ellision

Type names inside loadable values are elided.

Example (rust):
```rust
#[derive(Reflect, Default, PartialEq)]
struct OtherStruct
{
    a: u32
}

#[derive(Reflect, Default, PartialEq)]
enum OtherEnum
{
    A,
    B
}

#[derive(Reflect, Default, PartialEq)]
struct MyStruct
{
    a: OtherStruct,
    b: OtherEnum,
}

let s = MyStruct{ a: OtherStruct{ a: 10 }, b: OtherEnum::B };
```

Example (COB):
```rust
MyStruct{ a:{ a:10 } b:B }
```

Loadable type names are always required, since they are used to figure out how to deserialize values.

Example loadables (COB):
```rust
OtherStruct{ a:10 }
OtherEnum::A
```

### Newtype ellision

When a newtype struct appears inside a loadable, the newtype is 'peeled' to the innermost non-newtype type.

Example (rust):
```rust
#[derive(Reflect, Default, PartialEq)]
struct MyNewtype(u32);

#[derive(Reflect, Default, PartialEq)]
struct MyStruct
{
    a: MyNewtype(u32),
}

let s = MyStruct{ a: MyNewtype(10) };
```

Example (COB):
```rust
MyStruct{ a:10 }
//          ^
// MyNewtype(10) simplified to 10
```

### Newtype collapsing

Instead of peeling, loadable newtypes and newtype enum variants use *newtype collapsing*. Newtypes are collapsed by discarding 'outer layers'.

Example (rust):
```rust
#[derive(Reflect, Default, PartialEq)]
struct MyStruct
{
    a: u32,
}

#[derive(Reflect, Default, PartialEq)]
struct MyNewtype(MyStruct);

let s = MyNewtype(MyStruct{ a: 10 });
```

Example (COB):
```rust
MyNewtype{ a:10 }
// Un-collapsed: MyNewtype({ a:10 })
```

An important use-case for collapsing is bevy's `Color` type.

Rust:
```rust
let c = Color::Srgba(Srgba{ red: 1.0, blue: 1.0, green: 1.0, alpha: 1.0 });
```

COB:
```rust
Srgba{ red:1 blue:1 green:1 alpha:1 }
```

Here we have `Srgba` for the `Color::Srgba` variant, and `{ ... }` for the `Srgba{ ... }` inner struct's value.

### Floats

Floats are written similar to how they are written in rust.

- Scientific notation: `1.2e3` or `1.2E3`.
- Integer-to-float conversion: `1` can be written instead of `1.0`.
- Keywords `inf`/`-inf`/`nan`: infinity, negative infinity, `NaN`.

### String parsing

Strings are handled similar to how rust string literals are handled.

- Enclosed by double quotes (e.g. `"Hello, World!"`).
- Escape sequences: standard ASCII escape sequences are supported (`\n`, `\t`, `\r`, `\f`, `\"`, `\\`), in addition to Unicode code points (`\u{..1-6 digit hex..}`).
- Multi-line strings: a string segment that ends in `\` followed by a newline will be concatenated with the next non-space character on the next line.
- Can contain raw Unicode characters.

### Built-in types

Since COB is part of `bevy_cobweb_ui`, we include special support for two common UI types.

- [`Val`](bevy::prelude::Val): `Val` variants can be written with special units (`px`, `%`, `vw`, `vh`, `vmin`, `vmax`, `fr`) and the keyword `auto`. For example, `10px` is equivalent to `Px(10)`.
    - The `fr` variant corresponds to [`GridVal::Fraction`](bevy_cobweb_ui::prelude::GridVal::Fraction).
- [`Color`](bevy::prelude::Color): The `Color::Srgba` variant can be written with color-hex in the format `#FFFFFF` (for `alpha = 1.0`) or `#AAFFFFFF` (for custom `alpha`).

### Limitations

- Struct field names must be snake-case.
- Loadable names and enum variant names must be camel-case.
- Loadables must be structs/enums (no tuples, arrays, or rust primitives).

### Lossy conversions (COB file to rust value back to COB file):

- Scientific notation: only floats >= 1e16 or <= 1e-7 will be formatted with scientific notation when serializing to raw COB
- Trailing zeroes after decimal in floats: if float can be coerced to int, it will be; otherwise trailing zeroes will be removed
- Multiline strings: multi-line strings are concatenated
- In-line string formatting (newlines/tabs/etc.) and unicode characters will be replaced with escape sequences
- Unicode with leading zeros: leading zeroes removed
- Unicode escape sequences will be lower-cased
- Hex color sequences will be upper-cased
- Manual builtin values (e.g. `Val::Px(1.0)`) will be converted to auto-builtin (e.g. `1.0px`)
- Reflect-defaulted fields: all serializable fields will be serialized
    - Workaround: manually filter default values somehow ??
- Whitespace/comments/filler characters can often, but not always, be recovered using `recover_fill`

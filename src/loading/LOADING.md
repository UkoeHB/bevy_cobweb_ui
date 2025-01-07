## Cobweb asset format (COB)

COB is a minimalist custom asset format with the `.cob` file extension. This crate automatically loads, extracts, and manages hot-reloading for COB files.


### Loading files

COB files are assets and need to be [loaded](bevy_cobweb_ui::prelude::LoadedCobAssetFilesAppExt::load) into your app.

```rust
app.load("path/to/file.cob");
```

You always need to load at least one `.cob` file directly. The `#manifest` keyword can be used to transitively load other files (see [below](#Manifest-section) for details).


### Sections

All COB files are composed of *sections*.

For example, here's a file with one **`#scenes`** section:

```rust
// my_project/assets/main.cob
#scenes
"ex"
    FlexNode{width:10px height:10px}
    BackgroundColor{#229944}
```

There are six section types, all of which are optional and can be written in any order in a file:

- **`#manifest`**: Requests other COB files to be loaded, assigns *manifest keys*, and controls the global order that commands are applied.
- **`#import`**: Pulls **`#defs`** sections from other files into the current file using their manifest keys, with an optional import alias.
- **`#defs`**: Definitions of re-usable constants and scene macros.
- **`#commands`**: Bevy commands that are applied when a COB file is initially loaded. COB commands are globally ordered based on the file load order specified in **`#manifest`** sections.
- **`#scenes`**: Specifies scene hierarchies that can be spawned in-code as entity hierarchies. Scene nodes are composed of loadables (components and instructions).

File extraction uses the following overall algorithm.

1. First, **`#manifest`** and **`#import`** sections are extracted. Manifest files are loaded, and import entries are cached until the files they point to are loaded.
1. Once all imports are available, **`#defs`** sections are extracted in the order the appear in-file. When extracting **`#defs`**, each definition that internally requests other defs is 'resolved' using definitions available up to that point (including imports and previous definitions from the file).
    - After defs are extracted, the extracted values (stacked on top of the file's own imports) can be imported to other files.
1. Then all **`#commands`** sections are extracted in the order they appear in-file. Command values are immediately resolved using available **`#defs`** values (including both imports and defs from the file). Commands are buffered in order to apply them in the correct order (see [below](#Commands-section)).
1. Finally, all **`#scenes`** sections are extracted in the order they appear in-file. Similar to commands, all scene node values are immediately resolved using available **`#defs`** values.


### Manifest section

A manifest section is a sequence of 'file path : manifest key' pairs.

For example:

```rust
// my_project/assets/main.cob
#manifest
"menu/home_menu.cob" as home_menu
"widgets/slider.cob" as widgets.slider
```

Files loaded directly from the app aren't in any manifest sections, so you can also specify the manifest key of `self` in root files:

```rust
// my_project/assets/main.cob
self as main
```

The manifest key is used by import sections, and is also a shortcut that can be used when loading scenes.


### Import section

An import section is a sequence of 'manifest key : import alias' pairs.

For example:

```rust
// my_project/assets/main.cob
#import
home_menu as _
widgets.slider as slider
```

In this example, `home_menu` is given `_`, which means no import alias. Import aliases are prepended to all imported definitions, allowing you to 'namespace' definitions.

For example, this crate has built-in constants, including the `builtin.colors.tailwind` file. Tailwind has a constant `$AMBER_500` that is imported to `builtin.colors` with the `tailwind` import alias. If you import `builtin.colors as colors` to your project, then the constant will be available with `$colors::tailwind::AMBER_500`.


### Defs section

A definition allows data and pattern re-use within COB files. There are four kinds of definitions: constants, data macros, loadable macros, and scene macros.

**Constants**

Constants are a 'copy paste' mechanism for data, and use the symbol `$`. They need a definition in a **`#defs`** section.

Example (COB):
```rust
#defs
$text = "Hello, World!"

#scenes
"hello"
    TextLine{ text: $text }
```

A definition takes the form `${constant id} = {constant value}`.

You 'request' a constant with `${alias path}{constand id}`. The alias path comes from importing constants from other files.

Example (COB constants file):
```rust
// my_project/assets/constants.cob
#defs
$text = "Hello, World!"
```

Example (COB main file):
```rust
// my_project/assets/main.cob

// First load in "constants.cob"
#manifest
"constants.cob" as constants

// Then import constants from "constants.cob"
#import
constants as consts

#scenes
"hello"
    // Now we need the 'consts::' alias path.
    TextLine{ text: $consts::text }
```

A constant can point to a single value or a *value group*. Value groups look like `\ ..entries .. \` and can contain either values or key-value pairs. Value groups will be flattened into parent structures - arrays, tuples, or maps.

Example (COB):
```rust
#defs
$elements = \ 10 11 12 \
$entries = \ a:10 b:11 c:12 \

#commands
// Flattens to: MyNumbers[ 10 11 12 ]
MyNumbers[$elements]
// Flattens to: MyStruct{ a:10 b:11 c:12 }
MyStruct{$entries}
```


**Scene macros**

Scene macros allow 'scene fragements' to be copy-pasted into scenes. Scene fragments can be modified when inserting them to a scene.

Example (COB):
```rust
#defs
+hello_world = \
    TextLine{ text: "Hello, World!" }
\

#scenes
"hello"
    +hello_world{}
```

In this example, `+hello_world = \ ... \` defines a scene fragment with one scene layer. Then invoking it with `+hello_world{}` pastes it into the `"hello"` scene.

When you invoke a scene macro, you can make several kinds of changes to the macro content before it gets pasted.
1. Overwrite existing loadables.
1. Add new loadables.
1. Adjust an existing loadable using scene macro commands:
    1. Move it to the top: `^LoadableName`
    1. Move it to the bottom: `!LoadableName`
    1. Remove it: `-LoadableName`
1. Add new scene nodes.
1. Rearrange scene nodes.

Let's look at an example to illustrate these changes.

```rust
#defs
+base = \
    FlexNode{width:100px height:100px}
    BackgroundColor(#009900)

    "a"
        Width(50px)
        Height(75px)
        BackgroundColor(#990000)
    "b"
        FlexNode{width:50px height:50px}
        BackgroundColor(#222222)
\
+wrapped = \
    +base{
        // Overrides BackgroundColor in base layer
        BackgroundColor(#00FFFF)

        // Moves "b" before "a"
        "b"
        "a"
            // Overrides BackgroundColor in "a"
            BackgroundColor(#FF00FF)
    }
\

#scenes
"scene"
    ""
        +base{}

    ""
        +wrapped{}

    ""
        +wrapped{
            // Overrides FlexNode in base layer
            FlexNode{width:150px height:150px}

            "b"
                // Overrides FlexNode in node "b"
                FlexNode{width:100px height:100px}
            "a"
                // Adds FlexNode to node "a" at the end of the loadable list.
                FlexNode{width:100px}
                // Removes the Width loadable
                -Width
                // Moves the Height loadable below the new FlexNode loadable.
                !Height
        }
```

When wrapping a macro in another macro, it is recommended (but not required) to reproduce the entire node structure of the inner macro (i.e. the node names without loadables (unless you need to modify them)). This way you can see the entire macro structure without needing to trace out nested macro calls.


### Commands section

A command section is a sequence of *command loadables*. Command loadables are rust types that implement [`Command`](bevy::ecs::world::Command).

For example:

```rust
// my_project/assets/main.cob
#commands
Example
```

```rust
// my_project/src/main.rs
#[derive(Reflect, Default, PartialEq)]
struct Example;

impl Command for Example
{
    fn apply(self, _: &mut World)
    {
        println!("Example command");
    }
}
```

All loadables need to implement `Reflect`, `Default` and `PartialEq`.

Finally, command loadables must be registered in your app.

```rust
app.register_command_type::<Example>();
```

For details about COB value serialization, see [below](Value-serialization).

**Command ordering**

Commands are applied in a consistent global order, which has four rules:

1. Files loaded to the app with `app.load("{file}.cob")` are ordered by registration order.
1. Manifest entries are ordered from top to bottom.
1. Manifest entries' commands are ordered before a file's own commands.
1. Commands within a file are ordered from top to bottom.

The overall structure is 'leaf-first', which is how imports tend to flow (imports have no strict ordering requirements).


### Scenes section

A scenes section is a sequence of scenes. Each scene is a tree of scene nodes.

For example:

```rust
#scenes
"a"
    FlexNode{width:100px height:100px}

"b"
    FlexNode{width:100px height:100px}

    "bb"
        FlexNode{width:50px height:50px}
```

Scene `"a"` has one node: a root node `"a"`. Scene `"b"` has two nodes: a root node `"b"` and child node `"bb"`.

Node children are created by indenting them relative to their parents.

Only the root node is required to have a name. Other nodes can be anonymous using `""`.

For details about COB value serialization, see [below](Value-serialization).

**Spawning a scene**

Scenes can be spawned all-at-once, or you can build individual scene nodes into pre-spawned entities. The latter is useful when designing widgets with configurable structures.

For example:
```rust
let file = &SceneFile::new("example.cob");

// Spawns and builds an entire scene (entities: b, b::bb).
commands.ui_root().spawn_scene(file + "b");

// Builds an individual root node into the spawned entity.
commands.spawn_empty().build(file + "a");

// Builds an individual inner node into the spawned entity.
commands.spawn_empty().build(file + "b::bb");
```

Each node in a scene may have any number of [`Loadable`](bevy_cobweb_ui::prelude::Loadable) values, which are applied to entities.

**Loadable values**

A [`Loadable`](bevy_cobweb_ui::prelude::Loadable) value is a Rust type that is registered with one of the methods in [`CobLoadableRegistrationAppExt`](bevy_cobweb_ui::prelude::CobLoadableRegistrationAppExt).

For example, with the [`BackgroundColor`](bevy::prelude::BackgroundColor) component from `bevy`:

```rust
#scenes
"a"
    BackgroundColor(#F50A80)
```

When the scene node `"a"` is loaded to an entity, the [`BackgroundColor`](bevy::prelude::BackgroundColor) component will be inserted to the entity.

You can define three kinds of loadables:
- **Components**: Inserted as components.
- **Reactive**: Inserted as `bevy_cobweb` reactive components.
- **Instruction**: Applied to an entity via the [`Instruction`](bevy_cobweb_ui::prelude::Instruction) trait. The [`BrRadius`](bevy_cobweb_ui::prelude::BrRadius) loadable is an instruction that inserts the `BorderRadius` component.

For example:

```rust
#[derive(Reflect, Default, PartialEq)]
struct MyLoadable(usize);

// Use this if you want MyLoadable to be inserted as a `Component`.
// The type must implement `Component`.
app.register_component_type::<MyLoadable>();

// Use this if you want MyLoadable to be inserted as a `React` component.
// The type must implement `ReactComponent`.
app.register_reactive_type::<MyLoadable>();

// Use this if you want MyLoadable to mutate the entity.
// The type must implement `Instruction`.
app.register_instruction_type::<MyLoadable>();

impl Instruction for MyLoadable
{
    fn apply(self, entity: Entity, _: &mut World)
    {
        println!("MyLoadable({}) applied to entity {:?}", self.0, entity);
    }

    fn revert(entity: Entity, _: &mut World)
    {
        println!("MyLoadable reverted on entity {:?}", entity);
    }
}
```

The `revert` method on `Instruction` is used when hot-reloading an instruction. When a loadable is changed or removed from a node, then it will be reverted. After that, all of the nodes' loadables are re-applied in order. This two-step process allows best-effort state repair when complex mutations are hot reloaded.

**Warning**: If a loadable contains `NaN`, then it will *always* appear changed when a file reloads, since we use `reflect_partial_eq` to detect changes.

To load a full scene and edit it, you can use [`SpawnSceneExt::spawn_scene_and_edit`](bevy_cobweb_ui::prelude::SpawnSceneExt::spawn_scene_and_edit). This will spawn a hierarchy of nodes to match the hierarchy found in the specified scene tree. You can then edit those nodes with the [`SceneRef`](bevy_cobweb_ui::prelude::SceneRef) struct accessible in the `spawn_scene_and_edit` callback.

```rust
fn setup(mut c: Commands, mut s: ResMut<SceneBuilder>)
{
    let file = &SceneFile::new("main"); // Using a manifest key

    c.spawn_scene_and_edit(file + "game_menu_scene", &mut s, |loaded_scene: &mut SceneHandle<EntityCommands>| {
        // Do something with loaded_scene, which points to the root node...
        // - SceneRef derefs to the internal scene node builder (EntityCommands in this case).
        loaded_scene.insert(MyComponent);

        // Edit a child of the root node directly.
        loaded_scene.get("header")
            .do_something()
            .do_something_else();

        // Edit a more deeply nested child.
        loaded_scene.edit("footer::content", |loaded_scene| {
            // ...

            // Insert another scene as a child of this node.
            loaded_scene.spawn_scene_and_edit(file + "footer_scene", |loaded_scene| {
                // ...
            });
        });
    });
}
```


### Value serialization

Loadable values appear in COB files very similar to how they appear in Rust. Since COB is minimalist, there are several simplifications and details to note.

**Comments**

Single-line and block comments are both supported, the same as rust.

```rust
// Single-line
/*
Multi-line
*/
```

**Spacing instead of punctuation**

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

**Keywords banned from structs**

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

**Type name ellision**

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

**Newtype ellision**

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

**Newtype collapsing**

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

**Floats**

Floats are written similar to how they are written in rust.

- Scientific notation: `1.2e3` or `1.2E3`.
- Integer-to-float conversion: `1` can be written instead of `1.0`.
- Keywords `inf`/`-inf`/`nan`: infinity, negative infinity, `NaN`.

**String parsing**

Strings are handled similar to how rust string literals are handled.

- Enclosed by double quotes (e.g. `"Hello, World!"`).
- Escape sequences: standard ASCII escape sequences are supported (`\n`, `\t`, `\r`, `\f`, `\"`, `\\`), in addition to Unicode code points (`\u{..1-6 digit hex..}`).
- Multi-line strings: a string segment that ends in `\` followed by a newline will be concatenated with the next non-space character on the next line.
- Can contain raw Unicode characters.

**Built-in types**

Since COB is part of `bevy_cobweb_ui`, we include special support for two common UI types.

- [`Val`](bevy::prelude::Val): `Val` variants can be written with special units (`px`, `%`, `vw`, `vh`, `vmin`, `vmax`, `fr`) and the keyword `auto`. For example, `10px` is equivalent to `Px(10)`.
    - The `fr` variant corresponds to [`GridVal::Fraction`](bevy_cobweb_ui::prelude::GridVal::Fraction).
- [`Color`](bevy::prelude::Color): The `Color::Srgba` variant can be written with color-hex in the format `#FFFFFF` (for `alpha = 1.0`) or `#AAFFFFFF` (for custom `alpha`).





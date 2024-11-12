## Cobweb asset format (COB)

COB is a minimalist custom asset format with the `.cob` file extension. This crate has a built-in framework for loading, extracting, and managing hot-reloading for COB files.


### Loading files

In order for a COB file's contents to be available, you need to [load](bevy_cobweb_ui::prelude::LoadedCobAssetFilesAppExt::load) the file into your app.

```rust
app.load("path/to/file.cob");
```

You always need to load at least one `.cob` file directly. The `#manifest` keyword can be used to transitively load other files (see [below](#Manifest-section) for details).


### Sections

All COB files are composed of *sections*.

For example, here's a file with one **`#scenes`** section:

```rust
#scenes
"ex"
    FlexNode{width:10px height:10px}
    BackgroundColor{#229944}
```

There are six section types, all of which are optional and can be written in any order in a file:

- **`#manifest`**: Requests other COB files to be loaded, assigns *manifest keys*, and controls the global order that commands are applied.
- **`#import`**: Pulls **`#using`** and **`#defs`** sections from other files into the current file using their manifest keys, with an optional import alias.
- **`#using`**: Allows specifying the full typename of a loadable, to disambiguate it when multiple types with the same short name are registered for reflection. Very rarely needed.
- **`#defs`**: Definitions of re-usable constants and macros. **NOT YET IMPLEMENTED**
- **`#commands`**: Bevy commands that are applied when a COB file is initially loaded. COB commands are globally ordered based on the file load order specified in **`#manifest`** sections.
- **`#scenes`**: Specifies scene hierarchies that can be spawned in-code as entity hierarchies. Scene nodes are composed of loadables (components and instructions).

File extraction uses the following overall algorithm.

1. First, **`#manifest`** and **`#import`** sections are extracted. Manifest files are loaded, and import entries are cached until the files they point to are loaded.
1. Once all imports are available, **`#using`** and **`#defs`** sections are extracted in the order the appear in-file. When extracting **`#defs`**, each definition is simplified using other definitions available up to that point (including imports and previous definitions from the file).
    - After using/defs are extracted, the extracted values (stacked on top of the file's own imports) can be imported to other files.
1. Then all **`#commands`** sections are extracted in the order they appear in-file. Command values are immediately resolved using available **`#using`** and **`#defs`** values (including both imports and values from the file). Commands are buffered internally in order to apply them in the correct order (see [below](#Commands-section)).
1. Finally, all **`#scenes`** sections are extracted in the order they appear in-file. Similar to commands, all scene node values are immediately resolved using available **`#using`** and **`#defs`** values.


### Manifest section

A manifest section is a simple sequence of 'file path : manifest key' pairs.

For example:

```rust
// my_project/assets/main.cob
#manifest
"menu/home_menu.cob" as home_menu
"widgets/slider.cob" as widgets.slider
```

Files loaded directly from the app aren't in any manifest sections, so you can also specify the manifest key of `self`:

```rust
// my_project/assets/main.cob
self as main
```

The manifest key is used by import sections, and is also a shortcut that can be used when loading scenes.


### Import section

An import section is a simple sequence of 'manifest key : import alias' pairs.

For example:

```rust
// my_project/assets/main.cob
#manifest
home_menu as _
widgets.slider as slider
```

In this example, `home_menu` is given `_`, which means no import alias. Import aliases are prepended to all imported definitions, allowing you to 'namespace' definitions.

For example, in this repository the `builtin.colors.tailwind` file has a constant `$AMBER_500` that is imported to the `builtin.colors` file with the `tailwind` import alias. If you import `builtin.colors as colors`, then the constant will be available with `$colors::tailwind::AMBER_500`. (NOTICE: constants are not yet implemented)


### Using section

A using section is a simple sequence of 'long type name : short type name' pairs.

For example:

```rust
#using
bevy_cobweb_ui::ui_bevy::ui_ext::style_wrappers::FlexNode as FlexNode
```

If any command or scene in the file, or any file that imports the file, contains a `FlexNode`, then the type path specified here will be used to reflect the raw data into a concrete rust type.


### Defs section

TODO: definitions are not yet implemented


### Commands section

A command section is a sequence of *command loadables*. Command loadables are rust types that implement [`Command`](bevy::ecs::world::Command).

For example:

```rust
#commands
Example
```

All loadables need to implement `Reflect`, `Default` and `PartialEq`, in addition to traits like `Command` for specific loadable types.

```rust
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

Finally, command loadables must be registered in your app.

```rust
app.register_command_type::<Example>();
```

For details about COB value serialization, see [the section below](Value-serialization).

**Command ordering**

Commands are applied in a consistent global order, which has four rules:

1. Files loaded to the app with `app.load("{file}.cob")` are ordered by registration order.
1. Commands found in a file's **`#manifest`** section are ordered before the file's own commands.
1. Manifest entries are ordered from top to bottom.
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

For details about COB value serialization, see [the section below](Value-serialization).

**Spawning a scene**

Scenes can be spawned all-at-once, or you can load individual scene nodes into pre-spawned entities. The latter is useful when designing widgets with configurable structures (e.g. the `radio_button` widget lets you configure the indicator dot's location).

For example:
```rust
let file = SceneFile::new("example.cob");

// Spawns and loads an entire scene (entities: b, b::bb).
commands.ui_root().load_scene(file + "b");

// Loads an individual root node into the spawned entity.
commands.spawn_empty().load(file + "a");

// Loads an individual inner node into the spawned entity.
commands.spawn_empty().load(file + "b::bb");
```

Each node in a scene may have any number of [`Loadable`](bevy_cobweb_ui::prelude::Loadable) values, which are applied to entities.

**Loadable values**

A [`Loadable`](bevy_cobweb_ui::prelude::Loadable) value is a Rust type that is registered with one of the methods in [`CobAssetRegistrationAppExt`](bevy_cobweb_ui::prelude::CobAssetRegistrationAppExt).

For example, with the [`BackgroundColor`](bevy::prelude::BackgroundColor) component from `bevy`:

```rust
#scenes
"a"
    BackgroundColor(#F50A80)
```

When the scene node `"a"` is loaded to an entity, the [`BackgroundColor`](bevy::prelude::BackgroundColor) component will be inserted to the entity.

You can define three kinds of loadables:
- **Bundles**: Inserted as bundles.
- **Reactive**: Inserted as `bevy_cobweb` reactive components.
- **Instruction**: Applied to an entity via the [`Instruction`](bevy_cobweb_ui::prelude::Instruction) trait. The [`BrRadius`](bevy_cobweb_ui::prelude::BrRadius) loadable is an instruction that inserts the `BorderRadius` component.

For example:

```rust
#[derive(Reflect, Default, PartialEq)]
struct MyInstruction(usize);

// Use this if you want MyInstruction to be inserted as a `Bundle`.
// The type must implement `Bundle` (or `Component`).
app.register_bundle_type::<MyInstruction>();

// Use this if you want MyInstruction to be inserted as a `React` component.
// The type must implement `ReactComponent`.
app.register_reactive_type::<MyInstruction>();

// Use this if you want MyInstruction to mutate the entity.
// The type must implement `Instruction`.
app.register_instruction_type::<MyInstruction>();

impl Instruction for MyInstruction
{
    fn apply(self, entity: Entity, _: &mut World)
    {
        println!("MyInstruction({}) applied to entity {:?}", self.0, entity);
    }

    fn revert(entity: Entity, _: &mut World)
    {
        println!("MyInstruction reverted on entity {:?}", entity);
    }
}
```

The `revert` method on `Instruction` is used when hot-reloading an instruction. When a loadable is changed or removed from a node, then it will be reverted. Components are removed, and instructions call `revert`. After that, all of the nodes loadables are re-applied in order. This two-step process allows best-effort state repair when complex mutations are hot reloaded.

To load a full scene and edit it, you can use [`LoadSceneExt::load_scene_and_edit`](bevy_cobweb_ui::prelude::LoadSceneExt::load_scene_and_edit). This will spawn a hierarchy of nodes to match the hierarchy found in the specified scene tree. You can then edit those nodes with the [`LoadedScene`](bevy_cobweb_ui::prelude::LoadedScene) struct accessible in the `load_scene_and_edit` callback.

```rust
fn setup(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let file = &SceneFile::new("main"); // Using a manifest key

    c.load_scene_and_edit(file + "game_menu_scene", &mut s, |loaded_scene: &mut LoadedScene<EntityCommands>| {
        // Do something with loaded_scene, which points to the root node...
        // - LoadedScene derefs to the internal scene node builder (EntityCommands in this case).
        loaded_scene.insert(MyComponent);

        // Edit a child of the root node directly.
        loaded_scene.get("header")
            .do_something()
            .do_another_thing();

        // Edit a more deeply nested child.
        loaded_scene.edit("footer::content", |loaded_scene| {
            // ...

            // Insert another scene as a child of this node.
            loaded_scene.load_scene_and_edit(file + "footer_scene", |loaded_scene| {
                // ...
            });
        });
    });
}
```


### Value serialization

TODO

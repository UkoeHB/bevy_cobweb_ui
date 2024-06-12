## Loadablesheet asset format

Loadablesheets are written as JSON files with the extension `.load.json`. You must register loadablesheets in your
app with [`LoadableSheetListAppExt::load_sheet`]. The `#manifest` keyword can be used to transitively load files (see below for details).


### Base layer

Each file must have one map at the base layer.
```json
{

}
```


### Loadable paths

In the base layer, you can construct path trees as nested maps. These are used to access [`Loadable`] values from in your app.

Each section of a path tree can contain deeper paths, and values. Path segment names must be lower-case.

```json
{
    "root": {
        "a": {
            // Values

            "inner": {
                // More values
            }
        },
        "b": {
            // Other values
        }
    }
}
```

Path references may be combined into path segments, which can be used to reduce indentation.

```json
{
    "a": {
        // Values
    },

    "a::inner": {
        // More values
    }
}
```


### Loadable values

A [`Loadable`] value is a Rust type that is registered for reflection with [`App::register_type`]. It can be loaded from file by writing its short type name in a path tree, followed by the value that will be deserialized in your app.

For example, with the [`BgColor`] loadable defined in this crate:

```json
{
    "root": {
        "a": {
            "BgColor": {"Hsla": {"hue": 274.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},

            "inner": {
                // More values
            }
        },
        "b": {
            // Other values
        }
    }
}
```

To access loadables in your app, do the following:

1. Register the file to be loaded (using its path in the asset directory)
```rust
app.load_sheet("path/to/file.load.json")
```
2. Make a reference to the file. You can use the file path or its manifest key (see the `#manifest` keyword for details).
```rust
let file = LoadableRef::from_file("path/to/file.load.json");
```
3. Extend the file with the path to access.
```rust
let a = file.e("root").e("a");
```
4. Load the value onto your entity. All values stored at the path `a` will be loaded onto the entity.

```rust
commands.spawn_empty().load(a);
```

When it is inserted (or reinserted due to its value changing on file, if you have the `hot_reload` feature enabled), a [`Loaded`] entity event will be emitted. See [`ReactCommands::entity_event`](bevy_cobweb::prelude::ReactCommands::entity_event).


### Keywords

Several keywords are supported in loadable files.

#### Comments: `#c:`

Comments can be added as map entries throughout loadable files. They can't be added inside loadable values.

```json
{
    "#c: My comment":1
}
```

We need to add `:1` here because the comment is a map entry, which means it needs *some* value (any value is fine). We write the comment in the key since map keys need to be unique.

#### Commands: `#commands`

Normal loadables implement [`ApplyLoadable`](bevy_cobweb_ui::prelude::ApplyLoadable) and must be loaded onto specific entities. If you want a 'world-scoped' loadable, i.e. data that is applied automatically when loaded in, then you can use a `#commands` section with types that implement [`ApplyCommand`](bevy_cobweb_ui::prelude::ApplyCommand).

```rust
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MyCommand(usize);

impl ApplyCommand for MyCommand
{
    fn apply(self, _c: &mut Commands)
    {
        println!("MyCommand applied: {}", self.0);
    }
}
```

```json
{
    "#commands": {
        "MyCommand": [10],
    }
}
```

#### Name shortcuts: `#using`

In each file we reference types using their 'short names' (e.g. `Color`). If there is a type conflict (which will happen if multiple registered [`Reflect`] types have the same short name), then we need to clarify it in the file so values can be reflected correctly.

To solve that you can add a `#using` section to the base map in a file. The using section must be an array of full type names.
```json
{
    "#using": [
        "crate::my_module::Color",
        "bevy_cobweb_ui::ui_bevy::ui_ext::component_wrappers::BgColor"
    ]
}
```

#### Inheritance: `#inherited`

If you want values to propagate down a tree, you can use `#inherited` to pull-in the nearest value.

Inheritance is ordering-dependent, so if you don't want a loadable to be inherited, insert it below any child nodes.
```json
{
    "a": {
        "Width": {"Px": 25.0},

        "inner": {
            "Width": "inherited"
        }
    }
}
```

If a loadable is inherited in an abbreviated path, it will inherit from the current scope, not its path-parent.
```json
{
    "root": {
        "Width": {"Px": 25.0},

        "a": {
            "Width": {"Px": 50.0},
        },

        "a::inner": {
            // This inherits 50.0.
            "Width": "inherited"
        }
    }
}
```

#### Constants: `#constants`

It is often useful to have the same value in multiple places throughout a file. Constants let you 'paste' sections of JSON to different places.

The `#constants` section is a tree in the base layer where you define constants. Path segments in the tree must start with `$`. You can access other constants within the constants tree using `$$path::to::constant`.
```json
{
    "#constants": {
        "$start":{
            "$inner": 10.0
        },
        "$outer": "$$start::inner"
    }
}
```

There are two ways to reference a constant, either as a value or a map key.

When accessing a constant as a value (an array entry or a value in a map), the data pointed-to by the constant path is pasted in place of the constant.

This example shows inserting a constant in the middle of a value. We use `$path::to::constant` when referencing a constant in a normal value tree.
```json
{
    "#constants": {
        "$standard":{
            "$hue": 250.0
        }
    },

    "background": {
        "BgColor": {"Hsla": {"hue": "$standard::hue", "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
    }
}
```

When accessing a constant as a map key, the 'terminal segment' of the constant is inserted in place of the constant, and the value pointed-to by the constant is inserted as the map value.

In this example, the `BgColor` constant is inserted as a value to the `background` path.
```json
{
    "#constants": {
        "$standard": {
            "$BgColor": {"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
        }
    },

    "background": {
        "$standard::BgColor": {},
    }
}
```

When expanded, the result will be
```rust
{
    "background": {
        "BgColor": {"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
    }
}
```

To support this, you can end a constant with `::*` to 'paste all' when it's used as a map key.

```json
{
    "#constants": {
        "$standard":{
            "$BgColor": {"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
            "$AbsoluteStyle": {
                "dims": {"width": {"Px": 100.0}, "height": {"Px": 100.0}}
            }
        }
    },

    "my_node": {
        "$standard::*": {},
    }
}
```

#### Imports: `#import`

You can import `#using` and `#constants` sections from other files with the `#import` keyword.

Add the `#import` section to the base map in a file. It should be a map between file names and file aliases. The aliases can be used to access constants imported from each file.

```json
// my_constants.load.json
{
    "#constants": {
        "$standard":{
            "$BgColor": {"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
            "$AbsoluteStyle": {
                "dims": {"width": {"Px": 100.0}, "height": {"Px": 100.0}}
            }
        }
    },
}

// my_app.load.json
{
    "#import": {
        "my_constants.load.json": "constants"
    },

    "my_node": {
        "$constants::standard::*": {},
    }
}
```

Imports will be implicitly treated as manifests (see the next section), but *without* manifest keys. You can have a file in multiple manifest and import sections.

#### Transitive loading: `#manifest`

Sheets can be transitively loaded by specifying them in a `#manifest` section.

Add the `#manifest` section to the base map in a file. It should be a map between file names and manifest keys. The manifest keys can be used in [`LoadableFile`] references in place of explicit file paths.

An empty map key `""` can be used to set a manifest key for the current file. This is mainly useful for the root-level sheet which must be loaded via [`LoadableSheetListAppExt::load_sheet`].

```json
// button_widget.load.json
{
    "widget": {
        // ...
    }
}

// app.load.json
{
    "my_node": {
        // ...
    }
}

// manifest.load.json
{
    "#manifest": {
        "": "manifest",
        "button_widget.load.json": "widgets.button",
        "app.load.json": "app"
    },

    "demo_node_in_manifest": {
        // ...
    }
}
```

## Cobweb asset format

**TODO: These docs are out of date! We now have a custom format CAF instead of JSON.**

Cobweb assets are written as JSON files with the extension `.caf.json`.


### Loading files

In order for a cobweb asset file's contents to be available, you need to [load](bevy_cobweb_ui::prelude::LoadedCobwebAssetFilesAppExt::load) the file into your app.

```rust
app.load("path/to/file.caf.json");
```

You always need to load at least one `caf` file directly. The `#manifest` keyword can be used to transitively load other files (see below for details).


### Base layer

Each file must have one map at the base layer.
```json
{

}
```


### Scenes

In the base layer, you can construct nested maps of path ids, which we call scenes. Path ids must be lower-case.

```json
{
    "root": {
        "a": {
            "inner": {}
        },
        "b": {}
    }
}
```

Every root node can be loaded as a scene (a hierarchy of entities is automatically spawned), and any node in a scene can be loaded as an individual scene node (only the target entity is modified).

A path can be written out by combining segments with "::", such as `root::a::inner`.

For example:
```rust
let file = SceneFile::new("example.caf.json");

// Loads an individual root node into the spawned entity.
commands.spawn_empty().load(file + "root");

// Loads an individual inner node into the spawned entity.
commands.spawn_empty().load(file + "root::a");

// Spawns and loads an entire scene (entities: root, root::a, root::a::inner, root::b).
commands.load_scene(file + "root");
```

Each node in a scene may have any number of [`Loadable`](bevy_cobweb_ui::prelude::Loadable) values, which are applied to entities.


### Loadable values

A [`Loadable`](bevy_cobweb_ui::prelude::Loadable) value is a Rust type that is registered with one of the methods in [`CobwebAssetRegistrationAppExt`](bevy_cobweb_ui::prelude::CobwebAssetRegistrationAppExt). It can be added to a scene node by writing its short type name in a path tree, followed by the value that will be deserialized in your app.

For example, with the [`BgColor`](bevy_cobweb_ui::prelude::BgColor) loadable defined in this crate:

```json
{
    "root": {
        "a": {
            "BgColor": [{"Hsla": {"hue": 274.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}}],

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

When the scene node `"root::a"` is loaded to an entity, the [`BgColor`](bevy_cobweb_ui::prelude::BgColor) loadable will be applied to the entity.

You can define three kinds of loadables:
- **Bundles**: Inserted as bundles.
- **Reactive**: Inserted as reactive components.
- **Instruction**: Applied to an entity via the [`Instruction`](bevy_cobweb_ui::prelude::Instruction) trait. The [`BgColor`](bevy_cobweb_ui::prelude::BgColor) loadable is an instruction that inserts the Bevy `BackgroundColor` component.

```rust
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MyInstruction(usize);

// Use this if you want MyInstruction to be inserted as a `Bundle`.
// The type must implement `Bundle` (or `Component`).
app.register_bundle::<MyInstruction>();

// Use this if you want MyInstruction to be inserted as a `React` component.
// The type must implement `ReactComponent`.
app.register_reactive::<MyInstruction>();

// Use this if you want MyInstruction to mutate the entity.
// The type must implement `Instruction`.
app.register_instruction_type::<MyInstruction>();

impl Instruction for MyInstruction
{
    fn apply(self, ec: &mut EntityCommands)
    {
        println!("MyInstruction({}) applied to entity {:?}", self.0, ec.id());
    }
}
```

If you have the `hot_reload` feature enabled, then whenever a loadable is changed in a file it will be re-applied to all entities that loaded the associated scene node. If the feature is *not* enabled, then only values already loaded when an entity requests a scene node will be applied. This means you should only load scenes/scene nodes when in [`LoadState::Done`](bevy_cobweb_ui::prelude::LoadState::Done) so all loadables will be available.

- **Warning**: Loadables are 'non-reversible' so if you delete a loadable from a scene node, entities subscribed to that node won't be updated to reflect the change. To see the effects, you need to either restart your app or spawn new entities with that node.

To load a full scene, you can use [`LoadSceneExt::load_scene_and_edit`](bevy_cobweb_ui::prelude::LoadSceneExt::load_scene_and_edit). This will spawn a hierarchy of nodes to match the hierarchy found in the specified scene tree. You can then edit those nodes with the [`LoadedScene`](bevy_cobweb_ui::prelude::LoadedScene) struct accessible in the `load_scene_and_edit` callback.

```rust
fn setup(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let file = &SceneFile::new("path/to/file.caf.json");

    c.load_scene_and_edit(&mut s, file + "game_menu_scene", |loaded_scene: &mut LoadedScene<EntityCommands>| {
        // Do something with loaded_scene, which points to the root node...
        // - LoadedScene derefs to the internal scene node builder (EntityCommands in this case).
        loaded_scene.insert(MyComponent);

        // Edit a child of the root node.
        loaded_scene.edit("header", |loaded_scene| {
            // ...
        });

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


### Note on serialization

Note that to serialize unit structs to JSON you need to wrap them in `[]` like an array. This is a common source of errors.

```rust
pub struct CustomInt(pub usize);
```

```json
{
    "CustomInt": [42]
}
```


### Keywords

Several keywords are supported in `caf` files.

#### Comments: `#c:`

Comments can be added as map entries throughout `caf` files (except inside loadable values).

```json
{
    "#c: My comment":0
}
```

We need to add `:0` here because the comment is a map entry, which means it needs *some* value (any value is fine). We write the comment in the key since map keys need to be unique (otherwise we couldn't have multiple comments in a single map).

#### Commands: `#commands`

Scene nodes must be loaded onto specific entities. If you want a 'world-scoped' loadable, i.e. data that is applied automatically when loaded in, then you can add a `#commands` section with types that implement [`Command`](bevy::ecs::world::Command).

Commands are globally ordered by:
1. Files manually registered to an app with [`LoadedCobwebAssetFilesAppExt::load`](bevy_cobweb_ui::prelude::LoadedCobwebAssetFilesAppExt::load).
2. Commands in a file's `#commands` section(s).
3. Files loaded recursively via CAF manifests. Commands in file A will be applied before any commands in 
manifest files in file A. All `self as xxx` manifest entries are ignored for command ordering.

```json
{
    "#commands": {
        "MyCommand": [10],
    }
}
```

Impementation of `MyCommand`. Note that `MyCommand` must be registered with the app:

```rust
use bevy::ecs::world::Command;

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MyCommand(usize);

impl Command for MyCommand
{
    fn apply(self, _w: &mut World)
    {
        println!("MyCommand applied: {}", self.0);
    }
}

// Commands must be registered.
app.register_command_type::<MyCommand>();
```

#### Name shortcuts: `#using`

In each file we reference types using their 'short names' (e.g. `Color`). If there is a type conflict (which will happen if multiple registered [`Reflect`](bevy::prelude::Reflect) types have the same short name), then we need to clarify it in the file so values can be reflected correctly.

To solve that you can add a `#using` section to the base map in a file. The using section must be an array of full type names.
```json
{
    "#using": [
        "my_color_crate::custom_colors::Color",
        "bevy_cobweb_ui::ui_bevy::ui_ext::component_wrappers::BgColor"
    ]
}
```

#### Constants: `#constants`

It is often useful to have the same value in multiple places throughout a file. Constants let you 'paste' sections of JSON to different locations.

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

When accessing a constant as a value (an array entry or a value in a map), the data pointed to by the constant path is pasted in place of the constant.

This example shows inserting a constant in the middle of a value. We use `$path::to::constant` when referencing a constant in a normal value tree.
```json
{
    "#constants": {
        "$standard":{
            "$hue": 250.0
        }
    },

    "background": {
        "BgColor": [{"Hsla": {"hue": "$standard::hue", "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}}],
    }
}
```

Which expands to:
```json
{
    "background": {
        "BgColor": [{"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}}],
    }
}
```

When accessing a constant as a map key, you must end it with `::*`, which means 'paste all contents'.

In this example, the [`BgColor`](bevy_cobweb_ui::prelude::BgColor) and [`AbsoluteStyle`](bevy_cobweb_ui::prelude::AbsoluteStyle) loadables are inserted to the `my_node` path.
```json
{
    "#constants": {
        "$standard":{
            "BgColor": {"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
            "AbsoluteStyle": {
                "dims": {"width": {"Px": 100.0}, "height": {"Px": 100.0}}
            }
        }
    },

    "my_node": {
        "$standard::*": {},
    }
}
```
When expanded, the result will be
```json
{
    "my_node": {
        "BgColor": {"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
        "AbsoluteStyle": {
            "dims": {"width": {"Px": 100.0}, "height": {"Px": 100.0}}
        }
    }
}
```

Future versions of this crate may add more features to 'constants in map keys'.

#### Specs: `#specs`

When designing a widget, it is useful to have a base implementation and styling, and then to customize it as needed. To support this we have the `#specs` section. Specifications (specs) are parameterized JSON data that can be pasted into commands or scene trees. Overriding a spec is a simple as redefining some of its parameters.

Spec definitions have three pieces.
1. **Parameters**: Parameters are written as `@my_param` and can be used to insert data anywhere within a spec's content.
2. **Insertion points**: Insertion points are written as `!my_insertion_point` and can be added to any map key or array within a spec's content. Overriding an insertion point lets you paste arbitrary values into the spec content. This can be used to add loadables to positions in scene trees, add entries to arrays, or add normally-defaulted fields to structs. They also allow you to expand a spec's definition by adding more areas that can be parameterized to the spec content.
3. **Content**: Marked with `*`, spec content is inserted when a spec is requested in the `#commands` section or the scene tree.

A spec can be requested anywhere in a file's `#specs` section, `#commands` section, or its scene tree by adding a 'spec request' with format `IDENTIFIER(#spec:requested_spec)`. Spec requests can set parameter values, add content to insertion points, and add new parameters that are referenced by content added to insertion points.

Note that constants are applied to the `#specs` section in a file before specs are imported from other files, and before the `#specs` section is evaluated to extract spec definitions.

Here is a spec definition for a trivial `text` widget:
```json
{
    "#specs": {
        "text": {
            "@size": 30.0,
            "*": {
                "FlexStyle": {},
                "TextLine": {
                    "size": "@size",
                    "!textline": ""
                },
                "!insert": ""
            }
        }
    }
}
```

The spec would be used like this:
```json
{
    "#specs": "..omitted..",

    "root": {
        "#c: Root entity sets up the UI.":0,
        "FlexStyle": {
            "dims": {"width": {"Vw": 100.0}, "height": {"Vh": 100.0}},
            "content": {"justify_main": "SpaceEvenly", "justify_cross": "Center"}
        },

        "#c: Invoke the text spec as a loadable section.":0,
        "hello_text(#spec:text)": {
            "@size": 50.0,
            "!textline": {
                "text": "Hello, World!"
            },
            "!insert": {
                "TextLineColor": {"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}
            }
        }
    }
}
```

**`#specs` definition overrides**

You can override an existing spec by adding a spec definition-override like `new_spec_name(#spec:spec_to_override)` as a key in the `#specs` map. If the spec names are different, then a new spec will be created by copying the requested spec. Otherwise the requested spec will be overridden (its params and content will be overridden with new values specified by the request) and the updated version will be available in the remainder of the file and when importing the file to another file.

Here is our trivial text spec again:
```json
// file_a.caf.json
{
    "#specs": {
        "text": {
            "@size": 30.0,
            "*": {
                "FlexStyle": {},
                "TextLine": {
                    "size": "@size",
                    "!textline": ""
                },
                "!insert": ""
            }
        }
    }
}
```

And here we first override the text spec and then add a new spec derived from our overridden value. Note that specs are processed from top to bottom, which means an override in the specs section will be used by all references below the override.
```json
// file_b.caf.json
{
    "#import": {
        "file_a.caf.json": ""
    },

    "#specs": {
        "text(#spec:text)": {
            "@size": 45.0
        },

        "colorful_text(#spec:text)": {
            "!insert": {
                "TextLineColor": {"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}
            }
        }
    }
}
```

The `colorful_text` spec will have text size `45.0` and also a [`TextLineColor`](bevy_cobweb_ui::prelude::TextLineColor) loadable.

**Spec request in scene node**

You can insert a spec to a scene node with `path_identifier(#spec:spec_to_insert)`. When the spec is inserted, all parameters saved in the spec will be inserted to their positions in the spec content. Any nested specs in the spec content will also be inserted and their params resolved.

Here is a shortened version of the 'hello world' example from above:
```json
{
    "#specs": "..omitted..",

    "root": {
        "..root entity omitted..":0,

        "hello_text(#spec:text)": {
            "@size": 50.0,
            "!textline": {
                "text": "Hello, World!"
            },
            "!insert": {
                "TextLineColor": [{"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}]
            }
        }
    }
}
```

When the text spec is expanded, the final scene node will look like:
```json
{
    "#specs": "..omitted..",

    "root": {
        "..root entity omitted..":0,

        "hello_text": {
            "FlexStyle": {},
            "TextLine": {
                "size": 50.0,
                "text": "Hello, World!"
            },
            "TextLineColor": [{"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}]
        }
    }
}
```

**Loadable spec**

You can insert a spec as a loadable to the scene tree or `#commands` section with `MyInstruction(#spec:spec_to_insert)`. As with path specs, spec content is inserted, params are resolved, and nested specs are handled.

We could rewrite the `text` spec like this:
```json
{
    "#specs": {
        "text": {
            "@size": 30.0,
            "*": {
                "size": "@size",
                "!textline": ""
            }
        }
    }
}
```

And then use the spec content to directly fill in the `TextLine` loadable:
```json
{
    "#specs": "..omitted..",

    "root": {
        "..root entity omitted..":0,

        "hello_text": {
            "FlexStyle": {},
            "TextLine(#spec:text)": {
                "@size": 50.0,
                "!textline": {
                    "text": "Hello, World!"
                },
            },
            "TextLineColor": [{"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}]
        }
    }
}
```

**Nested specs**

Specs can reference other specs internally. This allows making complex widget structures composed of smaller widgets when you want the small widgets to be externally customizable.

In this example we use the `text` spec as a component of a simple `button` spec:
```json
// file_a.caf.json
{
    "#specs": {
        "text": {
            "@size": 30.0,
            "*": {
                "FlexStyle": {},
                "TextLine": {
                    "size": "@size",
                    "!textline": ""
                },
                "!insert": ""
            }
        },

        "button_text(#spec:text)": {
            "@margin": {"top": {"Px": 5.0}, "bottom": {"Px": 5.0}, "left": {"Px": 8.0}, "right": {"Px": 8.0}},
            "!insert": {
                "Margin": "@margin"
            }
        },

        "button": {
            "*": {
                "core": {
                    "FlexStyle": {
                        "dims"    : { "!dims":"" },
                        "content" : { "!content":"" },
                        "flex"    : { "!flex":"" }
                    },

                    "text(#spec:button_text)": { }
                }
            }
        }
    }
}
```

If you provide an override definition for `button_text` when requesting the `button` spec, then the *override* definition will be used when the nested `button` is expanded.

```json
// file_b.caf.json
{
    "#import": {
        "file_a.caf.json": ""
    },

    "#specs": {
        "button_text(#spec::button_text)": {
            "@size": 100.0
        }
    },

    "my_big_button(#spec:button)": {}
}
```

The `my_big_button` scene will have an inner text entity with `100.0` size font.

#### Imports: `#import`

You can import `#using`, `#constants`, and `#specs` sections from other files with the `#import` keyword.

Add the `#import` section to the base map in a file. It should be a map between file names or manifest keys and file aliases. The aliases can be used to access constants imported from each file. Note that specs do *not* use the aliases, because specs can be nested and we want spec overrides to apply to spec requests that are inside spec content.

```json
// my_constants.caf.json
{
    "#constants": {
        "$standard":{
            "BgColor": {"Hsla": {"hue": 250.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
            "AbsoluteStyle": {
                "dims": {"width": {"Px": 100.0}, "height": {"Px": 100.0}}
            }
        }
    },
}

// my_app.caf.json
{
    "#import": {
        "my_constants.caf.json": "constants"
    },

    "my_node": {
        "$constants::standard::*": {},
    }
}
```

If you import a file name explicitly, then it will be treated as a 'manifest request' and the file will be automatically loaded (but the file *won't* have a manifest key assigned to it). You can have a file in multiple manifest and import sections.

#### Transitive loading: `#manifest`

Cobweb asset files can be transitively loaded by specifying them in a `#manifest` section.

Add the `#manifest` section to the base map in a file. It should be a map between file names and manifest keys. The manifest keys can be used in [`SceneFile`](bevy_cobweb_ui::prelude::SceneFile) references in place of explicit file paths.

An empty map key `""` can be used to set a manifest key for the current file. This is mainly useful for the root-level file which must be loaded via [`LoadedCobwebAssetFilesAppExt::load`](bevy_cobweb_ui::prelude::LoadedCobwebAssetFilesAppExt::load).

```json
// button_widget.caf.json
{
    "widget": {
        // ...
    }
}

// app.caf.json
{
    "my_scene": {
        // ...
    }
}

// manifest.caf.json
{
    "#manifest": {
        "": "manifest",
        "button_widget.caf.json": "widgets.button",
        "app.caf.json": "app"
    },

    "demo_scene_in_manifest_file": {
        // ...
    }
}
```

Then you only need to load the manifest to get the other files loaded automatically:
```rust
app.load("manifest.caf.json");
```

And now manifest keys can be used instead of file paths to reference files:

```rust
fn setup(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    // Load widget
    c.load_scene(&mut s, SceneRef::new("widgets.button", "widget"));

    // Load app scene
    c.load_scene(&mut s, SceneRef::new("app", "my_scene"));

    // Load demo scene
    c.load_scene(&mut s, SceneRef::new("manifest", "demo_scene_in_manifest_file"));
}
```


### Overriding imports

When using widgets defined in third-party libraries, it is useful to customize the global theming/styling of those widgets so individual widgets don't need to be manually overridden. Similarly, it is useful to customize the theming/styling of a *class* of widgets without overriding individual specs.

We enable global customization through import overrides. Since it is allowed to import via manifest key, we can 'swap' the file that a manifest key points to at runtime.

#### Example

**Widget crate**

Define a reference file for the global constants:

`embedded://my_widget_crate/global_constants_ref.caf.json`
```json
{
"#manifest": {
    "": "builtin.global_constants_ref",
},
"#constants": {
    "$constant_a": 10,
    "$constant_b": 20,
}
}
```

Add a re-export shim that will be used by default:

`embedded://my_widget_crate/global_constants.caf.json`
```json
{
"#manifest": {
    "": "builtin.global_constants",
},
"#import": {
    "builtin.global_constants_ref": ""
}
}
```

Write your built-in widget:

`embedded://my_widget_crate/example_widget.caf.json`
```json
{
"#manifest": {
    "": "builtin.widgets.example_widget"
},
"#import": {
    "builtin.global_constants": "glob"
},

"scene": {
    "DataA": "$glob::constant_a",
    "DataB": "$glob::constant_b"
}
}
```

Add a plugin for importing your widgets:
```rust
pub struct MyWidgetsPlugin
{
    pub with_default_constants: bool
}

impl Plugin for MyWidgetsPlugin
{
    fn build(&self, app: &mut App)
    {
        // Load constants reference.
        load_embedded_scene_file!(app, "my_widget_crate", "src/widgets", "global_constants_ref.caf.json");

        // Conditionally load re-export shim.
        if self.with_default_constants {
            load_embedded_scene_file!(app, "my_widget_crate", "src/widgets", "global_constants.caf.json");
        }

        // Load widget.
        load_embedded_scene_file!(app, "my_widget_crate", "src/widgets/example_widget", "example_widget.caf.json");
    }
}
```

**User-land crate**

Write your override for the external crate's constants. In this example we override only one of the original file's constants. Note that to override a constant you *can't* use an import alias for the file where the values to override originate.

`global_constants_override.caf.json`
```json
{
"#manifest": {
    "": "builtin.global_constants"
},
"#import": {
    "builtin.global_constants_ref": ""
},
"#constants": {
    "constant_a": 42
}
}
```

Setup your app, configuring `MyWidgetsPlugin` not to use default constants:

```rust
pub struct MyAppPlugin;

impl Plugin for MyAppPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .add_plugins(ReactPlugin)
            .add_plugins(CobwebUiPlugin)
            .add_plugins(my_widget_crate::MyWidgetsPlugin{ with_default_constants: false })
            .load("global_constants_override.caf.json")
            .add_systems(OnEnter(LoadState::Done), build_scene);
    }
}
```

Now when you load the built-in widget, it will use your override for `constant_a`, and the original value of `constant_b`.

```rust
fn build_scene(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = SceneRef::new("builtin.widgets.example_widget", "scene");
    c.load_scene(&mut s, scene);
}
```

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
app.load_sheet("path/to/file.load.json");
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

You can define three kinds of custom loadables: loadables inserted as a component, inserted as reactive component, or applied to an entity as a 'derived' effect via [`ApplyLoadable`](bevy_cobweb_ui::prelude::ApplyLoadable).

```rust
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MyLoadable(usize);

// Use this if you want MyLoadable to be inserted as a `Bundle`. The type must implement `Bundle`.
app.register_loadable::<MyLoadable>();

// Use this if you want MyLoadable to be inserted as a `React` component. The type must implement `ReactComponent`.
app.register_reactive::<MyLoadable>();

// Use this if you want MyLoadable to mutate the entity. The type must implement `ApplyLoadable`.
app.register_derived::<MyLoadable>();

impl ApplyLoadable for MyLoadable
{
    fn apply(self, ec: &mut EntityCommands)
    {
        println!("MyLoadable({}) applied to entity {:?}", self.0, ec.id());
    }
}
```


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

Normal loadables must be loaded onto specific entities. If you want a 'world-scoped' loadable, i.e. data that is applied automatically when loaded in, then you can use a `#commands` section with types that implement [`ApplyCommand`](bevy_cobweb_ui::prelude::ApplyCommand).

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

// Commands must be registered.
app.register_command::<MyCommand>();
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
```json
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

#### Specs: `#specs`

When designing a widget, it is useful to have a base implementation and styling, and then to customize it as needed. To support this we have the `#specs` section. Specifications (specs) are parameterized JSON data that can be pasted into commands or loadable trees. Overriding a spec is a simple as redefining some of its parameters.

Here is a spec for a trivial `text` widget:
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

Spec definitions have three pieces.
1. **Parameters**: Parameters are written as `@my_param` and can be used to insert data anywhere within a spec's content.
2. **Insertion points**: Insertion points are written as `!my_insertion_point` and can be added to any map key or array within a spec's content. Overriding an insertion point lets you paste arbitrary values into the spec content. This can be used to add loadables to positions in loadable trees, add entries to arrays, or add normally-defaulted fields to structs. They also allow you to expand a spec's definition by adding more paremeters to the spec content.
3. **Content**: Marked with `*`, spec content is inserted when a spec is invoked in the `#commnds` section or the loadable tree.

An existing spec can be invoked anywhere in a file's `#specs` section, `#commands` section, or its loadable tree by adding a 'spec invocation' with format `IDENTIFIER(#spec:spec_to_invoke)`. Spec invocations can define parameters or add content to insertion points.

Note that constants are applied to the `#specs` section before specs are imported from other files, and before the `#specs` section is evaluated.

**`#specs` override**

You can override an existing spec by adding a spec invocation like `new_spec_name(#spec:spec_to_override)` as a key in the `#specs` map. If the spec names are different, then a new spec will be created by copying the invoked spec. Otherwise the invoked spec will be overridden (its params and content will be overridden with new values specified by the invocation) and the updated version will be available in the remainder of the file.

Here is our trivial text spec again:
```json
// file_a.load.json
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
// file_b.load.json
{
    "#import": {
        "file_a.load.json": ""
    },

    "#specs": {
        "text(#spec:text)": {
            "@size": 30.0
        },

        "colorful_text(#spec:text)": {
            "!insert": {
                "TextLineColor": {"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}
            }
        }
    }
}
```

The `colorful_text` spec will have text size `30.0` and also a `TextLineColor` loadable.

**Path spec**

You can insert a spec to a path position in the loadable tree with `path_identifier(#spec:spec_to_insert)`. When the spec is inserted, all parameters saved in the spec will be inserted to their positions in the spec content. Any nested specs in the spec content will also be inserted and their params resolved.

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
                "TextLineColor": {"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}
            }
        }
    }
}
```

When the text spec is expanded, the final loadable will look like:
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
            "TextLineColor": {"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}
        }
    }
}
```

**Loadable spec**

You can insert a spec as a loadable to the loadable tree or `#commands` section with `MyLoadable(#spec:spec_to_insert)`. As with path specs, spec content is inserted, params are resolved, and nested specs are handled.

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
            "TextLineColor": {"Hsla": {"hue": 0.0, "saturation": 0.52, "lightness": 0.9, "alpha": 0.8}}
        }
    }
}
```

**Nested specs**

It is allowed for specs to reference other specs internally. This allows making complex widget structures composed of smaller widgets when you want the small widgets to be externally customizable.

In this example we use the `text` spec as a component of a simple `button` spec:
```json
// file_a.load.json
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

#### Imports: `#import`

You can import `#using`, `#constants`, and `#specs` sections from other files with the `#import` keyword.

Add the `#import` section to the base map in a file. It should be a map between file names and file aliases. The aliases can be used to access constants imported from each file. Note that specs do *not* use the aliases, because specs can be nested and we want spec overrides to apply to spec invocations that are inside spec content.

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

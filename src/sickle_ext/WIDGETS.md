TODO: THIS IS OUT OF DATE, REQUIRES COB SCENE MACROS



## Implementing widgets

This crate uses a custom widget framework inspired by `sickle_ui`'s theming framework. We continue to use `sickle_ui` attributes and `PseudoStates`, and collect them into `DynamicStyles`, but we don't use `Theme<C>` or `C` components.

Instead, we have a simple interface with two loadables: [`ControlRoot`](bevy_cobweb_ui::prelude::ControlRoot) and [`ControlLabel`](bevy_cobweb_ui::prelude::ControlLabel). You add a `ControlRoot` to the root entity of a widget, and `ControlLabels` to sub-entities. When those loadables are present in a UI node tree, the `Static`/`Responsive`/`Animated` control loadables will recognize them and adjust their behavior. By default, control loadables on an entity with `ControlLabel` will receive interactions from the root entity. You can also manually specify the `source` and `target` entities within those loadables. Control loadables will respond to `PseudoStates` on the root entity.

### Example widget

Below we showcase the [counter_widget](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter_widget) example from the repository (simplified, abbreviated, and annotated).

First we define a builder for our widget. Note that we don't add `ControlRoot` or `ControlLabel` here. Those are added in the spec file (shown below).

```rust
#[derive(Default)]
struct CounterWidgetBuilder
{
    custom: Option<SceneRef>,
}

impl CounterWidgetBuilder
{
    /// Sets the path where the widget specification override should be loaded from.
    fn customize(mut self, custom: SceneRef) -> Self
    {
        self.custom = custom.into();
        self
    }

    /// Returns a reference to the cobweb asset file where the widget is defined.
    fn widget_file() -> SceneFile
    {
        SceneFile::new("widgets.counter")
    }

    /// Builds the widget as a child of an entity.
    fn build<'a>(self, builder: &'a mut UiBuilder<Entity>) -> UiBuilder<'a, Entity>
    {
        // Override the button structure with a spec override.
        let button = self
            .custom
            .unwrap_or_else(|| Self::widget_file() + "counter_widget");

        let mut core_entity = Entity::PLACEHOLDER;

        // The base of the widget is loaded manually to illustrate (complex widgets may customize
        // the order of sub-entities, in which case scene-based loading would not be feasible).
        builder.load(button, |button, path| {
            core_entity = button.id();

            // Note: We don't show the `Counter` struct, which is just a wrapper around `usize` and
            // has some helper methods for setting up reactive systems.
            let button_id = button.id();
            button
                .insert_reactive(Counter(0))
                .on_released(Counter::increment(button_id));

            // Inner entities are loaded manually.
            button.load(
                path + "text",
                |text, _path| {
                    text.update_on(
                        entity_mutation::<Counter>(button_id),
                        |text_id| Counter::write("Counter: ", button_id, text_id),
                    );
                },
            );
        });

        builder.commands().ui_builder(core_entity)
    }
}
```

In this framework *structure* and *styling* are merged, and you customize widgets by overriding widget specs (see the [loading](bevy_cobweb_ui::loading) module for information about specs). The code-side widget adds all structure and attributes at once.

Now we construct a `spec` for the widget containing default structure and styling.

```rust
// assets/widgets/counter.cob.json
{
"#specs" : {
    "counter_widget": {
        "@text_margin": {"top": {"Px": 10.0}, "bottom": {"Px": 10.0}, "left": {"Px": 18.0}, "right": {"Px": 18.0}},
        "@text_size": 50.0,
        "@bg_idle": {"Hsla": {"hue": 274.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
        "@bg_hover": {"Hsla": {"hue": 274.0, "saturation": 0.32, "lightness": 0.46, "alpha": 0.8}},
        "@bg_press": {"Hsla": {"hue": 274.0, "saturation": 0.40, "lightness": 0.35, "alpha": 0.8}},

        "!button": {
            "FlexNode": {
                "content": {"justify_main": "Center", "justify_cross": "Center"},
                "dims": {"!button_dims": ""},
                "flex": {"!button_flex": ""}
            },
            "Animated<BackgroundColor>": {
                "values": {"idle": "@bg_idle", "hover": "@bg_hover", "press": "@bg_press"},
                "settings": {
                    "pointer_enter": {"duration": 0.15, "ease": "OutExpo"},
                    "pointer_leave": {"duration": 0.15, "ease": "OutExpo"},
                    "press": {"duration": 0.2, "ease": "OutExpo"},
                    "release": {"duration": 0.2, "ease": "OutExpo"}
                }
            },
            "Interactive": []
        },
        "!text": {
            "FlexNode": {"flex": {"margin": "@text_margin"}},
            "TextLine": {"size": "@text_size"}
        },

        "*": {
            "ControlRoot": ["CounterWidget"],
            "!button":0,

            "text": {
                "ControlLabel": ["CounterWidgetText"],
                "!text":0
            }
        }
    }
},

"#c: Make the default widget available at this location.":0,
"counter_widget(#spec:counter_widget)": {}
}
```

This example doesn't actually use any `PseudoStates` or interaction propagation. For more complex examples check this crate's repository.

Customizing the widget is as simple as redefining params or adding inserts:
```json
// assets/main.cob.json
{
"#import": {"widgets/counter.cob.json": ""},

"counter_widget_bigtext(#spec:counter_widget)": {"@text_size": 100.0}
}
```

To build the widget, we use the `UiBuilder` from `sickle_ui`:

```rust
fn build_ui(mut c: Commands)
{
    c.ui_root().container(|n| {
        // Add default widget.
        CounterWidgetBuilder::default().build(n);

        // Add customized widget.
        CounterWidgetBuilder::default()
            .customize(SceneRef::new("main.cob.json", "counter_widget_bigtext"))
            .build(n);
    });
}
```

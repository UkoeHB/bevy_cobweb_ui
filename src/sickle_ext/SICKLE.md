This module extends [sickle_ui](https://github.com/UmbraLuminosa/sickle_ui) by integrating its attributes framework with scene loading from cobweb asset files, and by adding `UiBuilder` extensions that bring in [bevy_cobweb](https://github.com/UkoeHB/bevy_cobweb) reactivity (e.g. for interactions, see [`UiInteractionExt`](bevy_cobweb_ui::prelude::UiInteractionExt)).


## `sickle_ui` attributes

`sickle_ui` theming revolves around attributes. Attributes are entity mutators applied under certain conditions.

Note that *theming* a loadable means the loadable will be part of a *theme*, which means it can be overloaded (more on that below).

There are three kinds of attributes:
- **Static**: A static value is applied directly to an entity. For example, a static attribute could set the text size of a widget. We have the [`Themed<T>`](bevy_cobweb_ui::prelude::Themed) loadable for inserting static attributes to entities, and the corresponding [`ThemedAttribute`]((bevy_cobweb_ui::prelude::ThemedAttribute)) trait that must be implemented on `T`. Loadables should only use `Themed` when you want overloadability.
- **Responsive**: A responsive value will change in response to `sickle_ui` flux interactions (you need the [`Interactive`](bevy_cobweb_ui::prelude::Interactive) loadable to enable this). For example, the background color of a widget may change in response to hovering or pressing it. We have the [`Responsive<T>`](bevy_cobweb_ui::prelude::Responsive) loadable for these attributes, and the corresponding [`ResponsiveAttribute`]((bevy_cobweb_ui::prelude::ResponsiveAttribute)) trait that must be implemented on `T`. Note that `sickle_ui` calls these 'interactive attributes', but we call them 'responsive' for clarity.
- **Animated**: An animated value will change fluidly between states in response to `sickle_ui` flux interactions (again you need the [`Interactive`](bevy_cobweb_ui::prelude::Interactive) loadable to enable this). We have the [`Animated<T>`](bevy_cobweb_ui::prelude::Animated) loadable for these attributes, and the corresponding [`AnimatableAttribute`]((bevy_cobweb_ui::prelude::AnimatableAttribute)) trait that must be implemented on `T`.

To illustrate, here is our implementation of the [`BgColor`](bevy_cobweb_ui::prelude::BgColor) loadable:
```rust
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgColor(pub Color);

impl ApplyLoadable for BgColor
{
    fn apply(self, ec: &mut EntityCommands)
    {
        ec.try_insert(BackgroundColor(self.0));
    }
}

impl ThemedAttribute for BgColor
{
    type Value = Color;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        // Make a BgColor and then call ApplyLoadable::apply.
        Self(value).apply(ec);
    }
}

impl ResponsiveAttribute for BgColor {}
impl AnimatableAttribute for BgColor {}
```

To animate it on an entity, your coweb asset file could look like this (it's a reactive square):
```json
{
"scene": {
    "FlexStyle": {
        "dims": {"width": {"Px": 100.0}, "height": {"Px": 100.0}}
    },
    "Animated<BgColor>": {
        "values": {
            "idle": {"Hsla": {"hue": 0.0, "saturation": 0.0, "lightness": 0.25, "alpha": 1.0}},
            "pressed": {"Hsla": {"hue": 0.0, "saturation": 0.0, "lightness": 0.5, "alpha": 1.0}}
        },
        "settings": {
            "press": {"duration": 0.1, "easing": "Linear"},
            "release": {"duration": 0.1, "easing": "Linear"}
        }
    },
    "Interactive": []
}
}
```


## `sickle_ui` themes

In `sickle_ui`, a theme is a `Theme<C>` component containing collections of attributes, for some 'theme target' component `C`. If you add component `C` to an entity, then theme attributes from that entity's `Theme<C>` component and all `Theme<C>` components on ancestors of the entity will be collected/collapsed into a single attribute set and applied to the entity.

Applying an attribute set means saving the attributes in a `DynamicStyle` component on the entity. When an entity's `DynamicStyle` is changed, all static attributes are immediately removed and applied to the entity. Responsive and animated attributes are retained in the component so they can be used to calculate and apply new values to the entity in response to flux interactions.

### Pseudo-states

Themes are actually multiple collections of attributes organized under different 'pseudo state sets'. A `PseudoState` is an entity state such as `Enabled`, `Disabled`, `Checked`, `Selected`, etc. An entity's current pseudo state set is stored in a `PseudoStates` component on the entity.

When an entity with theme target `C` gets an updated `Theme<C>`, `C`, or `PseudoStates` component, then their `DynamicStyle` will be reconstructed. The final dynamic style will contain the intersection between the attributes collected/collapsed from `Theme<C>` components on the entity and its ancestors, and the entity's `PseudoStates` component. If multiple instances of an attribute match against the entity's pseudo states, then the one with the most states will win.

### Multi-entity themes

If you are building a multi-entity themed widget, then you need to implement `UiContext` on `C`. When an instance of that widget is spawned, you must store all the sub-entities in `C` so they can be retrieved by `UiContext`.

If you want interactions on a specific entity in a widget to propagate to other entities in the widget, all responsive and animatable attributes for those other entities must be stored in the 'interactive' entity's `DynamicStyle` component. By default all attributes from sub-entities in a `sickle_ui` widget will be added to the base entity's `DynamicStyle` component.


## Implementing widgets

This crate uses a custom widget framework inspired by `sickle_ui`'s theming framework. We continue to use `sickle_ui` attributes and `PseudoStates`, and collect them into `DynamicStyles`, but we don't use `Theme<C>` or `C` components.

Instead, we have a simple interface with two loadables: [`ControlRoot`](bevy_cobweb_ui::prelude::ControlRoot) and [`ControlLabel`](bevy_cobweb_ui::prelude::ControlLabel). You add a `ControlRoot` to the root entity of a widget, and `ControlLabels` to sub-entities. When those loadables are present in a UI node tree, the `Themed`/`Responsive`/`Animated` control loadables will recognize them and adjust their behavior. By default, control loadables on an entity with `ControlLabel` will receive interactions from the root entity. You can also manually specify the `source` and `target` entities within those loadables. Control loadables will respond to `PseudoStates` on the root entity.

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
            .unwrap_or_else(|| Self::widget_file().e("counter_widget"));

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
                path.e("text"),
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
// assets/widgets/counter.caf.json
{
"#specs" : {
    "counter_widget": {
        "@text_margin": {"top": {"Px": 10.0}, "bottom": {"Px": 10.0}, "left": {"Px": 18.0}, "right": {"Px": 18.0}},
        "@text_size": 50.0,
        "@bg_idle": {"Hsla": {"hue": 274.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
        "@bg_hover": {"Hsla": {"hue": 274.0, "saturation": 0.32, "lightness": 0.46, "alpha": 0.8}},
        "@bg_press": {"Hsla": {"hue": 274.0, "saturation": 0.40, "lightness": 0.35, "alpha": 0.8}},

        "!button": {
            "FlexStyle": {
                "content": {"justify_main": "Center", "justify_cross": "Center"},
                "dims": {"!button_dims": ""},
                "flex": {"!button_flex": ""}
            },
            "Animated<BgColor>": {
                "values": {"idle": "@bg_idle", "hover": "@bg_hover", "press": "@bg_press"},
                "settings": {
                    "pointer_enter": {"duration": 0.15, "easing": "OutExpo"},
                    "pointer_leave": {"duration": 0.15, "easing": "OutExpo"},
                    "press": {"duration": 0.2, "easing": "OutExpo"},
                    "release": {"duration": 0.2, "easing": "OutExpo"}
                }
            },
            "Interactive": []
        },
        "!text": {
            "FlexStyle": {"flex": {"margin": "@text_margin"}},
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
// assets/main.caf.json
{
"#import": {"widgets/counter.caf.json": ""},

"counter_widget_bigtext(#spec:counter_widget)": {"@text_size": 100.0}
}
```

To build the widget, we use the `UiBuilder` from `sickle_ui`:

```rust
fn build_ui(mut c: Commands)
{
    c.ui_builder(UiRoot).container(|n| {
        // Add default widget.
        CounterWidgetBuilder::default().build(n);

        // Add customized widget.
        CounterWidgetBuilder::default()
            .customize(SceneRef::new("main.caf.json", "counter_widget_bigtext"))
            .build(n);
    });
}
```

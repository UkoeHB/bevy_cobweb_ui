This module extends [sickle_ui](https://github.com/UmbraLuminosa/sickle_ui) by integrating its theming framework with scene loading from cobweb asset files, and by adding `UiBuilder` extensions that bring in [bevy_cobweb](https://github.com/UkoeHB/bevy_cobweb) reactivity (e.g. for interactions, see [`UiInteractionExt`](bevy_cobweb_ui::prelude::UiInteractionExt)).


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

If you want interactions on a specific entity in a widget to propagate to other entities in the widget, all responsive and animatable attributes for those other entities must be stored in the 'interactive' entity's `DynamicStyle` component. By default all attributes from sub-entities in a widget will be added to the base entity's `DynamicStyle` component. You can instead place attributes on sub-entities with the `with_placement` and `and_placement` extension methods.


## Implementing themes (and widgets)

There are two different ways to build UI widgets in this crate. One uses `sickle_ui` theming for interaction propagation and state management, and `bevy_cobweb_ui` specs for overrides. The other uses standard `sickle_ui` theming and overrides.

### Commonalities

Both methods use `sickle_ui` themes, so there are in-code commonalities.

Below we show how the [counter_widget](https://github.com/UkoeHB/bevy_cobweb_ui/tree/master/examples/counter_widget) example from the repository is built (simplified, abbreviated, and annotated). First we define the widget's theme target component and add a marker type for its inner text entity.

```rust
/// Marker type for the counter widget's internal text entity.
///
/// Notice this derives `TypeName`, which is a simple trait that adds a
/// NAME constant to the type (`CounterWidgetText::NAME` in this case).
#[derive(TypeName)]
struct CounterWidgetText;

/// Theme target component for our counter widget.
///
/// Adding this to an entity means it should receive `Theme<CounterWidget>` attributes.
#[derive(Component, DefaultTheme, Copy, Clone, Debug)]
struct CounterWidget
{
    text_entity: Entity,
}

/// Implement `UiContext` so `sickle_ui` can properly apply styles to the inner
/// text entity.
impl UiContext for CounterWidget
{
    fn get(&self, target: &str) -> Result<Entity, String>
    {
        match target {
            CounterWidgetText::NAME => Ok(self.text_entity),
            _ => Err(format!("unknown UI context {target} for {}", type_name::<Self>())),
        }
    }
    fn contexts(&self) -> Vec<&'static str>
    {
        vec![CounterWidgetText::NAME]
    }
}
```

Now we define a builder for our widget.

```rust
#[derive(Default)]
struct CounterWidgetBuilder
{
    /// METHOD 1: Spec override.
    /// METHOD 2: Theme override.
    custom: Option<SceneRef>,
}

impl CounterWidgetBuilder
{
    /// Sets the path where the widget theme/specification override should be loaded from.
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
        // METHOD 1: Override the button structure with a spec override.
        let button = self
            .custom
            .unwrap_or_else(|| Self::widget_file().e("counter_widget"));

        // METHOD 2: The button structure is built-in.
        let button = Self::widget_file().e("counter_widget");

        let mut core_entity = Entity::PLACEHOLDER;
        let mut text_entity = Entity::PLACEHOLDER;

        // The base of the widget is loaded via `load_with_theme`.
        builder.load_with_theme::<CounterWidget>(button, &mut core_entity, |button, path| {
            // METHOD 2: Optionally override the theming.
            if let Some(theme) = self.custom {
                // Here, `load_theme` loads a scene node (with theme support) directly to the entity,
                // different from `load_with_theme` which spawns a child and loads a scene node with theming
                // to the child.
                button.load_theme::<CounterWidget>(theme.clone());
                // The sub-entity might also have theme override data. The attributes will be inserted to
                // the `button` entity regardless of what entity in the widget it's loaded to.
                button.load_subtheme::<CounterWidget, CounterWidgetText>(theme.e("text"));
            }

            // Note: We don't show the `Counter` struct, which is just a wrapper around `usize` and
            // has some helper methods for setting up reactive systems.
            let button_id = button.id();
            button
                .insert_reactive(Counter(0))
                .on_released(Counter::increment(button_id));

            // Inner entities are loaded with `load_with_subtheme`.
            button.load_with_subtheme::<CounterWidget, CounterWidgetText>(
                path.e("text"),
                &mut text_entity,
                |text, _path| {
                    text.update_on(
                        entity_mutation::<Counter>(button_id),
                        |text_id| Counter::write("Counter: ", button_id, text_id),
                    );
                },
            );

            // We insert `CounterWidget` to the base of the widget so theming will take hold
            // properly.
            button.insert(CounterWidget { text_entity });
        });

        builder.commands().ui_builder(core_entity)
    }
}
```

Finally, we register the theme in the app:
```rust
app.add_plugins(ComponentThemePlugin::<CounterWidget>::new());
```

### Method 1: `bevy_cobweb_ui` specs

In this method *structure* and *styling* are merged, and you customize widgets by overriding widget specs (see the [loading](bevy_cobweb_ui::loading) module for information about specs). The code-side widget will add all structure and attributes at once. Under the hood we use `sickle_ui` theming to store and manage the attributes, but there is no code-side overriding or style inheritence because all attributes are always inserted directly to the widget's `Theme<C>` component, so there is no room to inherit attributes from `Theme<C>` components on ancestors.

Continuing the example from above, in this method we construct a `spec` for the widget containing default structure and styling.

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
            "!button":0,

            "text": {
                "!text":0
            }
        }
    }
},

"#c: Make the default widget available at this location.":0,
"counter_widget(#spec:counter_widget)": {}
}
```

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


### Method 2: standard `sickle_ui`

In this method we separate *structure* from *styling*. The code-side widget will only assemble the widget's structure (spawn the entity hierarchy and add non-themable details like interactivity and reactive behavior). Widget styling will come from theme attributes loaded separately.

For our example we will define a *base theme* and load that to an ancestor entity, then override its attributes as needed.

Here we define the widget structure and a base theme in the widget's cobweb asset file:

```json
// assets/widgets/counter.caf.json
{
"#manifest": {"": "widgets.counter"},

"counter_widget": {
    "FlexStyle": {"content": {"justify_main": "Center", "justify_cross": "Center"}},
    "Interactive": [],

    "text": {
        "FlexStyle": {
            "flex": {"margin": {"top": {"Px": 10.0}, "bottom": {"Px": 10.0}, "left": {"Px": 18.0}, "right": {"Px": 18.0}}}
        },
        "TextLine": {}
    }
},

"default_theme": {
    "Animated<BgColor>": {
        "values": {
            "idle": {"Hsla": {"hue": 274.0, "saturation": 0.25, "lightness": 0.55, "alpha": 0.8}},
            "hover": {"Hsla": {"hue": 274.0, "saturation": 0.32, "lightness": 0.46, "alpha": 0.8}},
            "press": {"Hsla": {"hue": 274.0, "saturation": 0.40, "lightness": 0.35, "alpha": 0.8}}
        },
        "settings": {
            "pointer_enter": {"duration": 0.15, "easing": "OutExpo"},
            "pointer_leave": {"duration": 0.15, "easing": "OutExpo"},
            "press": {"duration": 0.2, "easing": "OutExpo"},
            "release": {"duration": 0.2, "easing": "OutExpo"}
        }
    },

    "text": {
        "Themed<TextLineSize>": {"value": 35.0}
    }
}
}
```

To customize the widget we need to add a theming override. Note that we must reproduce the entire theme structure here because our widget assumes theme overrides mirror the structure. You could also override individual nodes with a more fine-grained widget builder.
```json
// assets/main.caf.json
{
"counter_widget_bigtext": {
    "text": {
        "Themed<TextLineSize>": {"value": 100.0}
    }
}
}
```

In order to load the default theme, we add a helper method to the widget:
```rust
impl CounterWidget
{
    fn load_default_theme(ui: &mut UiBuilder<Entity>)
    {
        let theme = CounterWidget::widget_file().e("default_theme");
        ui.load_theme::<CounterWidget>(theme.clone());
        ui.load_subtheme::<CounterWidget, CounterWidgetText>(theme.e("text"));
    }
}
```

Now we are ready to build the widget:

```rust
fn build_ui(mut c: Commands)
{
    c.ui_builder(UiRoot).container(|n| {
        // Add default theme.
        CounterWidget::load_default_theme(n);

        // Add default widget.
        // - NOTE: If the default theme wasn't loaded anywhere, then nothing will display!
        CounterWidgetBuilder::default().build(n);

        // Add customized widget.
        // - NOTE: If the default theme wasn't loaded anywhere, then non-overridden
        //   attributes won't display!
        CounterWidgetBuilder::default()
            .customize(SceneFile::new("main.caf.json", "counter_widget_bigtext"))
            .build(n);
    });
}
```


## `sickle_ui` non-theme dynamic styles

Attributes can be added directly to entities without using themes, with two caveats. First, if an entity has a theme target component, then directly added attributes will be overwritten. Second, pseudo states are *only* usable via themes (this means all multi-state widgets need to use themes).

If you are building a multi-entity widget and want `Responsive` and `Animated` attributes on sub-entities to react to interactions on the base entity, then you need to add the [`PropagateControl`](bevy_cobweb_ui::prelude::PropagateControl) component to the base entity and set the `inherit_control` field to `true` in the `Responsive` and `Animated` loadables of the sub-entities.

Here we extend our earlier reactive square by adding an inner square that is also animated:
```json
{
"scene": {
    "FlexStyle": {
        "dims": {"width": {"Px": 100.0}, "height": {"Px": 100.0}},
        "flex": {"JustifyMain": "Center", "JustifyCross": "Center"}
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
    "Interactive": [],
    "PropagateControl": {},

    "inner": {
        "FlexStyle": {
            "dims": {"width": {"Px": 33.3}, "height": {"Px": 33.3}}
        },
        "Animated<BgColor>": {
            "values": {
                "idle": {"Hsla": {"hue": 0.0, "saturation": 0.0, "lightness": 0.75, "alpha": 1.0}},
                "pressed": {"Hsla": {"hue": 0.0, "saturation": 0.0, "lightness": 1.0, "alpha": 1.0}}
            },
            "settings": {
                "press": {"duration": 0.1, "easing": "Linear"},
                "release": {"duration": 0.1, "easing": "Linear"}
            },
            "inherit_control": true
        }
    }
}
}
```

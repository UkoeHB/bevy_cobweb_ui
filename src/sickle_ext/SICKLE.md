This module provides COB instructions to control scene node attributes. For example, it allows you to animate the [`BackgroundColor`](bevy::prelude::BackgroundColor) of a UI node, or to provide a different [`Width`](bevy_cobweb_ui::prelude::Width) if a node is [`Selected`](bevy_cobweb_ui::sickle::PseudoState::Selected) vs not selected, or allow [`TextLineColor`](bevy_cobweb_ui::prelude::TextLineColor) to change when a text node's parent is hovered.

It also provides `UiBuilder` extensions to facilitate reactivity using [bevy_cobweb](https://github.com/UkoeHB/bevy_cobweb) (e.g. for interactions, see [`UiInteractionExt`](bevy_cobweb_ui::prelude::UiInteractionExt), and state changes, see [`PseudoStateExt`](bevy_cobweb_ui::prelude::PseudoStateExt)).


## Attributes

Attributes are entity mutators applied under certain conditions.

There are three kinds of attributes:
- **Static**: A static value is applied directly to an entity. For example, a static attribute could set the text size of a widget. We have the [`Static<T>`](bevy_cobweb_ui::prelude::Static) instruction for inserting static attributes to entities, and the corresponding [`StaticAttribute`]((bevy_cobweb_ui::prelude::StaticAttribute)) trait that must be implemented on `T`. `Static` is only useful when you want values that vary according to an entity's [`PseudoStates`](bevy_cobweb_ui::sickle::PseudoStates). Otherwise, just use `T` directly.
- **Responsive**: A responsive value will change in response to flux interactions. For example, the background color of a widget may change in response to hovering or pressing it. We have the [`Responsive<T>`](bevy_cobweb_ui::prelude::Responsive) instruction for these attributes, and the corresponding [`ResponsiveAttribute`]((bevy_cobweb_ui::prelude::ResponsiveAttribute)) trait that must be implemented on `T`.
- **Animated**: An animated value will change fluidly between states in response to flux interactions. We have the [`Animated<T>`](bevy_cobweb_ui::prelude::Animated) instruction for these attributes, and the corresponding [`AnimatedAttribute`]((bevy_cobweb_ui::prelude::AnimatedAttribute)) trait that must be implemented on `T`.

To illustrate, here is `bevy`'s built-in [`BackgroundColor`](bevy::prelude::BackgroundColor) component:
```rust
impl Instruction for BackgroundColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<BackgroundColor>();
        });
    }
}

impl StaticAttribute for BackgroundColor
{
    type Value = Color;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl ResponsiveAttribute for BackgroundColor {}
impl AnimatedAttribute for BackgroundColor {}
```

We include an `Instruction` implementation because it is a trait bound on `StaticAttribute`. `Instruction::apply` is used to apply values whenever a static/responsive/animatable attribute is activated.

To animate `BackgroundColor` on an entity, your COB file could look like this:
```rust
#scenes
"scene"
    FlexNode{width:100px height:100px}
    Animated<BackgroundColor>{
        idle:#123456
        press:#123477
        press_with:{duration:0.1 ease:Linear}
        release_with:{duration:0.1 ease:Linear}
    }
```

We include derive macros for setting up animations on components:
- [`StaticComponent`](bevy_cobweb_ui::prelude::StaticComponent)
- [`ResponsiveComponent`](bevy_cobweb_ui::prelude::ResponsiveComponent)
- [`AnimatedComponent`](bevy_cobweb_ui::prelude::AnimatedComponent)
- [`StaticReactComponent`](bevy_cobweb_ui::prelude::StaticReactComponent)
- [`ResponsiveReactComponent`](bevy_cobweb_ui::prelude::ResponsiveReactComponent)
- [`AnimatedReactComponent`](bevy_cobweb_ui::prelude::AnimatedReactComponent)
- [`StaticNewtype`](bevy_cobweb_ui::prelude::StaticNewtype)
- [`ResponsiveNewtype`](bevy_cobweb_ui::prelude::ResponsiveNewtype)
- [`AnimatedNewtype`](bevy_cobweb_ui::prelude::AnimatedNewtype)
- [`StaticReactNewtype`](bevy_cobweb_ui::prelude::StaticReactNewtype)
- [`ResponsiveReactNewtype`](bevy_cobweb_ui::prelude::ResponsiveReactNewtype)
- [`AnimatedReactNewtype`](bevy_cobweb_ui::prelude::AnimatedReactNewtype)


## Control groups

A control group is a group of entities in a scene hierarchy that are linked together and allow interactions on certain entities in the group to activate other entities' attributes.

`PseudoStates` on the root entity of a control group will determine which attributes are activated for all entities in the group.

You can set up a control root by adding a [`ControlRoot`](bevy_cobweb_ui::prelude::ControlRoot) instruction to the root node of the group, and [`ControlLabel`](bevy_cobweb_ui::prelude::ControlLabel) to the other nodes in the group.

### Anonymous control groups

If an entity has `Static/Responsive/Animated` but no `ControlRoot`/`ControlLabel`, then the entity will have an *anonymous* control group that only applies to itself.

### DynamicStyle

Control groups use an internal [`DynamicStyle`](bevy_cobweb_ui::sickle::DynamicStyle) component to handle active attributes. Whenever `DynamicStyle` changes, all static attributes are immediately removed and applied to the entity. Responsive and animated attributes are retained in the component so they can be used to apply new values to the entity in response to flux interactions.

### Pseudo-states

A `PseudoState` is an entity state such as `Enabled`, `Disabled`, `Checked`, `Selected`, etc. An entity's current set of pseudo states is stored in a `PseudoStates` component on the entity.

Whenever `PseudoStates` changes on the root node of a control group, all the attributes that match the new `PseudoStates` will be collected into `DynamicStyle` components and inserted to members of the group. All attributes with the same interaction 'source' will be collected into a single `DynamicStyle` component and inserted to the group member with label that matches the 'source' string. Static attributes always end up in a `DynamicStyle` component on the targeted entity.

It is possible for multiple instances of an attribute to match against the root entity's pseudo states (attribute states only need to be a subset of the root entity's pseudo states). In that case, the one with the most states will be selected.

In a single-entity anonymous control group, all attributes are inserted to a `DynamicStyle` component on the entity.

### Action at a distance

By default, the `Responsive` and `Animated` attributes will respond to interactions on the root of a control group. For example, in this structure:

```rust
#scenes
"node_a"
    FlexNode{width:100px height:100px}
    ControlRoot("a")

    "node_b"
        FlexNode{width:50px height:50px}
        ControlLabel("b")
        Responsive<BackgroundColor>{idle:#002200 press:#004400}
```

Interactions on the node with label "a" will cause the "b"-labeled node's color to change. Interactions on "b" will do nothing.

For other behavior, you can manually specify the interaction source that attributes respond to:

```rust
#scenes
"node_a"
    FlexNode{width:100px height:100px}
    ControlRoot("a")
    BackgroundColor(#111111)

    "node_b"
        FlexNode{width:50px height:50px}
        ControlLabel("b")
        // Here the source is set to 'c'.
        Responsive<BackgroundColor>{respond_to:"c" idle:#002200 hover:#004400}

    "node_c"
        FlexNode{width:50px height:50px}
        ControlLabel("c")
        BackgroundColor(#888888)
```

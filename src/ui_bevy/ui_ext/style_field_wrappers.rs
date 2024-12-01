use std::any::type_name;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

trait ApplyToNode: Sized + Send + Sync + 'static
{
    fn apply_to_absolute(self, _node: &mut AbsoluteNode, entity: Entity)
    {
        tracing::warn!("tried to apply {} to {:?} that has AbsoluteNode; only FlexNode is supported",
            type_name::<Self>(), entity);
    }
    fn apply_to_flex(self, node: &mut FlexNode);
}

//-------------------------------------------------------------------------------------------------------------------

fn initialize_absolute_node(
    In(entity): In<Entity>,
    mut c: Commands,
    query: Query<(Has<React<AbsoluteNode>>, Has<React<FlexNode>>)>,
)
{
    let Ok((maybe_absolute, maybe_flex)) = query.get(entity) else { return };

    // Check absolute node.
    if maybe_absolute {
        return;
    }

    // Check flex node.
    if maybe_flex {
        tracing::warn!("tried initializing absolute node on entity {:?} that has flex node", entity);
        return;
    }

    // Insert absolute node.
    c.react().insert(entity, AbsoluteNode::default());
}

//-------------------------------------------------------------------------------------------------------------------

fn initialize_flex_node(
    In(entity): In<Entity>,
    mut c: Commands,
    query: Query<(Has<React<AbsoluteNode>>, Has<React<FlexNode>>)>,
)
{
    let Ok((maybe_absolute, maybe_flex)) = query.get(entity) else { return };

    // Check flex node.
    if maybe_flex {
        return;
    }

    // Check absolute node.
    if maybe_absolute {
        tracing::warn!("tried initializing flex node on entity {:?} that has absolute node", entity);
        return;
    }

    // Insert flex node.
    c.react().insert(entity, FlexNode::default());
}

//-------------------------------------------------------------------------------------------------------------------

fn remove_nodes(entity: Entity, world: &mut World)
{
    let _ = world.get_entity_mut(entity).map(|mut e| {
        //e.remove::<(React<AbsoluteNode>, React<FlexNode>, Node)>();
        // TODO: need https://github.com/bevyengine/bevy/pull/16288 to remove Node
        e.remove::<(React<AbsoluteNode>, React<FlexNode>)>();
        e.insert(Node::default());
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_to_node<T: ApplyToNode>(param: T, entity: Entity, world: &mut World)
{
    let Ok(mut emut) = world.get_entity_mut(entity) else { return };

    // Check flex node.
    if let Some(mut flex) = emut.get_mut::<React<FlexNode>>() {
        param.apply_to_flex(&mut flex.get_noreact());
        React::<FlexNode>::trigger_mutation(entity, world);
        return;
    }

    // Check absolute node.
    if let Some(mut absolute) = emut.get_mut::<React<AbsoluteNode>>() {
        param.apply_to_absolute(&mut absolute.get_noreact(), entity);
        React::<AbsoluteNode>::trigger_mutation(entity, world);
        return;
    }

    // Fall back to inserting flex node.
    let mut node = FlexNode::default();
    param.apply_to_flex(&mut node);
    world.react(|rc| rc.insert(entity, node));
}

//-------------------------------------------------------------------------------------------------------------------

/// Initializes [`AbsoluteNode`] on an entity.
///
/// This instruction should be inserted before all other node field wrappers.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct WithAbsoluteNode;

impl Instruction for WithAbsoluteNode
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(entity, initialize_absolute_node);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}
impl StaticAttribute for WithAbsoluteNode
{
    type Value = ();
    fn construct(_: Self::Value) -> Self
    {
        Self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Initializes [`FlexNode`] on an entity.
///
/// This instruction should be inserted before all other node field wrappers.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct WithFlexNode;

impl Instruction for WithFlexNode
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(entity, initialize_flex_node);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}
impl StaticAttribute for WithFlexNode
{
    type Value = ();
    fn construct(_: Self::Value) -> Self
    {
        Self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::width`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Width(pub Val);

impl ApplyToNode for Width
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.width = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.width = self.0;
    }
}

impl Instruction for Width
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for Width
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for Width {}
impl AnimatedAttribute for Width {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::height`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Height(pub Val);

impl ApplyToNode for Height
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.height = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.height = self.0;
    }
}

impl Instruction for Height
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for Height
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for Height {}
impl AnimatedAttribute for Height {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::min_width`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MinWidth(pub Val);

impl ApplyToNode for MinWidth
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.min_width = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.min_width = self.0;
    }
}

impl Instruction for MinWidth
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for MinWidth
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for MinWidth {}
impl AnimatedAttribute for MinWidth {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::min_height`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MinHeight(pub Val);

impl ApplyToNode for MinHeight
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.min_height = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.min_height = self.0;
    }
}

impl Instruction for MinHeight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for MinHeight
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for MinHeight {}
impl AnimatedAttribute for MinHeight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::max_width`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MaxWidth(pub Val);

impl ApplyToNode for MaxWidth
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.max_width = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.max_width = self.0;
    }
}

impl Instruction for MaxWidth
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for MaxWidth
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for MaxWidth {}
impl AnimatedAttribute for MaxWidth {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::max_height`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MaxHeight(pub Val);

impl ApplyToNode for MaxHeight
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.max_height = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.max_height = self.0;
    }
}

impl Instruction for MaxHeight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for MaxHeight
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for MaxHeight {}
impl AnimatedAttribute for MaxHeight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::aspect_ratio`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct AspectRatio(pub f32);

impl ApplyToNode for AspectRatio
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.aspect_ratio = Some(self.0);
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.aspect_ratio = Some(self.0);
    }
}

impl Instruction for AspectRatio
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for AspectRatio
{
    type Value = f32;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for AspectRatio {}
impl AnimatedAttribute for AspectRatio {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::border`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Border(pub StyleRect);

impl ApplyToNode for Border
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.border = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.border = self.0;
    }
}

impl Instruction for Border
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for Border
{
    type Value = StyleRect;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for Border {}
impl AnimatedAttribute for Border {}

impl Splattable for Border
{
    type Splat = Val;
    fn splat(single: Self::Splat) -> Self
    {
        Border(StyleRect::splat(single))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::top`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct DimsTop(pub Val);

impl ApplyToNode for DimsTop
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.top = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.top = self.0;
    }
}

impl Instruction for DimsTop
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for DimsTop
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for DimsTop {}
impl AnimatedAttribute for DimsTop {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::bottom`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct DimsBottom(pub Val);

impl ApplyToNode for DimsBottom
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.bottom = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.bottom = self.0;
    }
}

impl Instruction for DimsBottom
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for DimsBottom
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for DimsBottom {}
impl AnimatedAttribute for DimsBottom {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::left`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct DimsLeft(pub Val);

impl ApplyToNode for DimsLeft
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.left = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.left = self.0;
    }
}

impl Instruction for DimsLeft
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for DimsLeft
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for DimsLeft {}
impl AnimatedAttribute for DimsLeft {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::right`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct DimsRight(pub Val);

impl ApplyToNode for DimsRight
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.right = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.right = self.0;
    }
}

impl Instruction for DimsRight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for DimsRight
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for DimsRight {}
impl AnimatedAttribute for DimsRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::clipping`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetClipping(pub Clipping);

impl ApplyToNode for SetClipping
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.clipping = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.clipping = self.0;
    }
}

impl Instruction for SetClipping
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for SetClipping
{
    type Value = Clipping;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetClipping {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::padding`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Padding(pub StyleRect);

impl ApplyToNode for Padding
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.padding = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.padding = self.0;
    }
}

impl Instruction for Padding
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for Padding
{
    type Value = StyleRect;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for Padding {}
impl AnimatedAttribute for Padding {}

impl Splattable for Padding
{
    type Splat = Val;
    fn splat(single: Self::Splat) -> Self
    {
        Padding(StyleRect::splat(single))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::flex_direction`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetFlexDirection(pub FlexDirection);

impl ApplyToNode for SetFlexDirection
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.flex_direction = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.flex_direction = self.0;
    }
}

impl Instruction for SetFlexDirection
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for SetFlexDirection
{
    type Value = FlexDirection;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetFlexDirection {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::flex_wrap`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetFlexWrap(pub FlexWrap);

impl ApplyToNode for SetFlexWrap
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.flex_wrap = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.flex_wrap = self.0;
    }
}

impl Instruction for SetFlexWrap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for SetFlexWrap
{
    type Value = FlexWrap;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetFlexWrap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_lines`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifyLines(pub JustifyLines);

impl ApplyToNode for SetJustifyLines
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.justify_lines = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.justify_lines = self.0;
    }
}

impl Instruction for SetJustifyLines
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for SetJustifyLines
{
    type Value = JustifyLines;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetJustifyLines {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_main`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifyMain(pub JustifyMain);

impl ApplyToNode for SetJustifyMain
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.justify_main = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.justify_main = self.0;
    }
}

impl Instruction for SetJustifyMain
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for SetJustifyMain
{
    type Value = JustifyMain;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetJustifyMain {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_cross`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifyCross(pub JustifyCross);

impl ApplyToNode for SetJustifyCross
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.justify_cross = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.justify_cross = self.0;
    }
}

impl Instruction for SetJustifyCross
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for SetJustifyCross
{
    type Value = JustifyCross;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetJustifyCross {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::column_gap`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct ColumnGap(pub Val);

impl ApplyToNode for ColumnGap
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.column_gap = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.column_gap = self.0;
    }
}

impl Instruction for ColumnGap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for ColumnGap
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for ColumnGap {}
impl AnimatedAttribute for ColumnGap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::row_gap`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct RowGap(pub Val);

impl ApplyToNode for RowGap
{
    fn apply_to_absolute(self, node: &mut AbsoluteNode, _: Entity)
    {
        node.row_gap = self.0;
    }
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.row_gap = self.0;
    }
}

impl Instruction for RowGap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for RowGap
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for RowGap {}
impl AnimatedAttribute for RowGap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::margin`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Margin(pub StyleRect);

impl ApplyToNode for Margin
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.margin = self.0;
    }
}

impl Instruction for Margin
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for Margin
{
    type Value = StyleRect;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for Margin {}
impl AnimatedAttribute for Margin {}

impl Splattable for Margin
{
    type Splat = Val;
    fn splat(single: Self::Splat) -> Self
    {
        Margin(StyleRect::splat(single))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::flex_basis`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct FlexBasis(pub Val);

impl ApplyToNode for FlexBasis
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.flex_basis = self.0;
    }
}

impl Instruction for FlexBasis
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for FlexBasis
{
    type Value = Val;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for FlexBasis {}
impl AnimatedAttribute for FlexBasis {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::flex_grow`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct FlexGrow(pub f32);

impl ApplyToNode for FlexGrow
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.flex_grow = self.0;
    }
}

impl Instruction for FlexGrow
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for FlexGrow
{
    type Value = f32;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for FlexGrow {}
impl AnimatedAttribute for FlexGrow {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::flex_shrink`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct FlexShrink(pub f32);

impl ApplyToNode for FlexShrink
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.flex_shrink = self.0;
    }
}

impl Instruction for FlexShrink
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for FlexShrink
{
    type Value = f32;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for FlexShrink {}
impl AnimatedAttribute for FlexShrink {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::justify_self_cross`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifySelfCross(pub JustifySelfCross);

impl ApplyToNode for SetJustifySelfCross
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, node: &mut FlexNode)
    {
        node.justify_self_cross = self.0;
    }
}

impl Instruction for SetJustifySelfCross
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_nodes(entity, world);
    }
}

impl StaticAttribute for SetJustifySelfCross
{
    type Value = JustifySelfCross;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetJustifySelfCross {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiStyleFieldWrappersPlugin;

impl Plugin for UiStyleFieldWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
        // Base type
        app.register_themed::<WithAbsoluteNode>();
        app.register_themed::<WithFlexNode>();

        // Dims
        app.register_animatable::<Width>()
            .register_animatable::<Height>()
            .register_animatable::<MinWidth>()
            .register_animatable::<MinHeight>()
            .register_animatable::<MaxWidth>()
            .register_animatable::<MaxHeight>()
            .register_animatable::<AspectRatio>()
            .register_animatable::<Border>()
            .register_animatable::<Splat<Border>>()
            .register_animatable::<DimsTop>()
            .register_animatable::<DimsBottom>()
            .register_animatable::<DimsLeft>()
            .register_animatable::<DimsRight>();

        // ContentFlex
        app.register_responsive::<SetClipping>()
            .register_animatable::<Padding>()
            .register_animatable::<Splat<Padding>>()
            .register_responsive::<SetFlexDirection>()
            .register_responsive::<SetFlexWrap>()
            .register_responsive::<SetJustifyLines>()
            .register_responsive::<SetJustifyMain>()
            .register_responsive::<SetJustifyCross>()
            .register_animatable::<ColumnGap>()
            .register_animatable::<RowGap>();

        // SelfFlex
        app.register_animatable::<Margin>()
            .register_animatable::<Splat<Margin>>()
            .register_animatable::<FlexBasis>()
            .register_animatable::<FlexGrow>()
            .register_animatable::<FlexShrink>()
            .register_responsive::<SetJustifySelfCross>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

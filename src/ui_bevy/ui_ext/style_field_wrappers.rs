use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

trait ApplyToNode: Sized + Send + Sync + 'static
{
    fn apply_to_node(self, node: &mut Node);
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_to_node_component<T: ApplyToNode>(param: T, entity: Entity, world: &mut World)
{
    let Ok(mut emut) = world.get_entity_mut(entity) else { return };

    // Get node.
    if let Some(node) = emut.get_mut::<Node>() {
        param.apply_to_node(node.into_inner());
        return;
    }

    // Fall back to inserting a flex node.
    let mut node: Node = FlexNode::default().into();
    param.apply_to_node(&mut node);
    emut.insert(node);
}

//-------------------------------------------------------------------------------------------------------------------

fn get_node_value<T>(entity: Entity, world: &World, callback: impl FnOnce(&Node) -> T) -> Option<T>
{
    let node = world.get::<Node>(entity)?;
    Some((callback)(node))
}

//-------------------------------------------------------------------------------------------------------------------

fn remove_node(entity: Entity, world: &mut World)
{
    let _ = world.get_entity_mut(entity).map(|mut e| {
        e.remove_with_requires::<Node>();
    });
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
    fn apply_to_node(self, node: &mut Node)
    {
        node.width = self.0;
    }
}

impl Instruction for Width
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for Width
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.width)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.height = self.0;
    }
}

impl Instruction for Height
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for Height
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.height)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.min_width = self.0;
    }
}

impl Instruction for MinWidth
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for MinWidth
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.min_width)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.min_height = self.0;
    }
}

impl Instruction for MinHeight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for MinHeight
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.min_height)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.max_width = self.0;
    }
}

impl Instruction for MaxWidth
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for MaxWidth
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.max_width)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.max_height = self.0;
    }
}

impl Instruction for MaxHeight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for MaxHeight
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.max_height)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.aspect_ratio = Some(self.0);
    }
}

impl Instruction for AspectRatio
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for AspectRatio
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.aspect_ratio)?
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::border`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Border(pub StyleRect);

impl ApplyToNode for Border
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.border = self.0.into();
    }
}

impl Instruction for Border
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for Border
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.border.into())
    }
}

impl Splattable for Border
{
    type Splat = Val;
    fn splat(single: Self::Splat) -> Self
    {
        Border(StyleRect::splat(single))
    }
    fn splat_value(self) -> Option<Self::Splat>
    {
        Some(self.0.top)
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
    fn apply_to_node(self, node: &mut Node)
    {
        node.top = self.0;
    }
}

impl Instruction for DimsTop
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for DimsTop
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.top)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.bottom = self.0;
    }
}

impl Instruction for DimsBottom
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for DimsBottom
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.bottom)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.left = self.0;
    }
}

impl Instruction for DimsLeft
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for DimsLeft
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.left)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.right = self.0;
    }
}

impl Instruction for DimsRight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for DimsRight
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.right)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`FlexContent::clipping`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetClipping(pub Clipping);

impl ApplyToNode for SetClipping
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.overflow = self.0.into();
    }
}

impl Instruction for SetClipping
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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

/// Mirrors [`FlexContent::clip_margin`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetClipMargin(pub OverflowClipMargin);

impl ApplyToNode for SetClipMargin
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.overflow_clip_margin = self.0;
    }
}

impl Instruction for SetClipMargin
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for SetClipMargin
{
    type Value = OverflowClipMargin;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetClipMargin {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`FlexContent::padding`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Padding(pub StyleRect);

impl ApplyToNode for Padding
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.padding = self.0.into();
    }
}

impl Instruction for Padding
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for Padding
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.padding.into())
    }
}

impl Splattable for Padding
{
    type Splat = Val;
    fn splat(single: Self::Splat) -> Self
    {
        Padding(StyleRect::splat(single))
    }
    fn splat_value(self) -> Option<Self::Splat>
    {
        Some(self.0.top)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`FlexContent::flex_direction`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetFlexDirection(pub FlexDirection);

impl ApplyToNode for SetFlexDirection
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.flex_direction = self.0;
    }
}

impl Instruction for SetFlexDirection
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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

/// Mirrors [`FlexContent::flex_wrap`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetFlexWrap(pub FlexWrap);

impl ApplyToNode for SetFlexWrap
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.flex_wrap = self.0;
    }
}

impl Instruction for SetFlexWrap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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

/// Mirrors [`GridContent::grid_auto_flow`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetGridFlow(pub GridAutoFlow);

impl ApplyToNode for SetGridFlow
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.grid_auto_flow = self.0;
    }
}

impl Instruction for SetGridFlow
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for SetGridFlow
{
    type Value = GridAutoFlow;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for SetGridFlow {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`GridContent::grid_auto_rows`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct GridAutoRows(pub Vec<GridVal>);

impl ApplyToNode for GridAutoRows
{
    fn apply_to_node(mut self, node: &mut Node)
    {
        node.grid_auto_rows = self.0.drain(..).map(|v| v.into()).collect();
    }
}

impl Instruction for GridAutoRows
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for GridAutoRows
{
    type Value = Vec<GridVal>;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for GridAutoRows {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`GridContent::grid_auto_columns`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct GridAutoColumns(pub Vec<GridVal>);

impl ApplyToNode for GridAutoColumns
{
    fn apply_to_node(mut self, node: &mut Node)
    {
        node.grid_auto_columns = self.0.drain(..).map(|v| v.into()).collect();
    }
}

impl Instruction for GridAutoColumns
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for GridAutoColumns
{
    type Value = Vec<GridVal>;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for GridAutoColumns {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`GridContent::grid_template_rows`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct GridTemplateRows(pub Vec<RepeatedGridVal>);

impl ApplyToNode for GridTemplateRows
{
    fn apply_to_node(mut self, node: &mut Node)
    {
        node.grid_template_rows = self.0.drain(..).map(|v| v.into()).collect();
    }
}

impl Instruction for GridTemplateRows
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for GridTemplateRows
{
    type Value = Vec<RepeatedGridVal>;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for GridTemplateRows {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`GridContent::grid_template_columns`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct GridTemplateColumns(pub Vec<RepeatedGridVal>);

impl ApplyToNode for GridTemplateColumns
{
    fn apply_to_node(mut self, node: &mut Node)
    {
        node.grid_template_columns = self.0.drain(..).map(|v| v.into()).collect();
    }
}

impl Instruction for GridTemplateColumns
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for GridTemplateColumns
{
    type Value = Vec<RepeatedGridVal>;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for GridTemplateColumns {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`FlexContent::justify_lines`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifyLines(pub JustifyLines);

impl ApplyToNode for SetJustifyLines
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.align_content = self.0.into();
    }
}

impl Instruction for SetJustifyLines
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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

/// Mirrors [`FlexContent::justify_main`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifyMain(pub JustifyMain);

impl ApplyToNode for SetJustifyMain
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.justify_content = self.0.into();
    }
}

impl Instruction for SetJustifyMain
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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

/// Mirrors [`FlexContent::justify_cross`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifyCross(pub JustifyCross);

impl ApplyToNode for SetJustifyCross
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.align_items = self.0.into();
    }
}

impl Instruction for SetJustifyCross
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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

/// Mirrors [`FlexContent::column_gap`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct ColumnGap(pub Val);

impl ApplyToNode for ColumnGap
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.column_gap = self.0;
    }
}

impl Instruction for ColumnGap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for ColumnGap
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.column_gap)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`FlexContent::row_gap`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct RowGap(pub Val);

impl ApplyToNode for RowGap
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.row_gap = self.0;
    }
}

impl Instruction for RowGap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for RowGap
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.row_gap)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::margin`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Margin(pub StyleRect);

impl ApplyToNode for Margin
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.margin = self.0.into();
    }
}

impl Instruction for Margin
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for Margin
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.margin.into())
    }
}

impl Splattable for Margin
{
    type Splat = Val;
    fn splat(single: Self::Splat) -> Self
    {
        Margin(StyleRect::splat(single))
    }
    fn splat_value(self) -> Option<Self::Splat>
    {
        Some(self.0.top)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.align_self = self.0.into();
    }
}

impl Instruction for SetJustifySelfCross
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
    fn apply_to_node(self, node: &mut Node)
    {
        node.flex_basis = self.0;
    }
}

impl Instruction for FlexBasis
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for FlexBasis
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.flex_basis)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.flex_grow = self.0;
    }
}

impl Instruction for FlexGrow
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for FlexGrow
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.flex_grow)
    }
}

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
    fn apply_to_node(self, node: &mut Node)
    {
        node.flex_shrink = self.0;
    }
}

impl Instruction for FlexShrink
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
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
impl AnimatedAttribute for FlexShrink
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        get_node_value(entity, world, |n| n.flex_shrink)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfGrid::grid_row`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct GridRow(pub GridInsertion);

impl ApplyToNode for GridRow
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.grid_row = self.0.into();
    }
}

impl Instruction for GridRow
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for GridRow
{
    type Value = GridInsertion;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for GridRow {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfGrid::grid_column`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct GridColumn(pub GridInsertion);

impl ApplyToNode for GridColumn
{
    fn apply_to_node(self, node: &mut Node)
    {
        node.grid_column = self.0.into();
    }
}

impl Instruction for GridColumn
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_node_component(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_node(entity, world);
    }
}

impl StaticAttribute for GridColumn
{
    type Value = GridInsertion;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for GridColumn {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiStyleFieldWrappersPlugin;

impl Plugin for UiStyleFieldWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
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

        // Content-shared
        app.register_responsive::<SetClipping>()
            .register_responsive::<SetClipMargin>()
            .register_animatable::<Padding>()
            .register_animatable::<Splat<Padding>>()
            .register_responsive::<SetFlexDirection>()
            .register_responsive::<SetFlexWrap>()
            .register_responsive::<SetJustifyLines>()
            .register_responsive::<SetJustifyMain>()
            .register_responsive::<SetJustifyCross>()
            .register_animatable::<ColumnGap>()
            .register_animatable::<RowGap>();

        // FlexContent-specific
        app.register_responsive::<SetFlexDirection>()
            .register_responsive::<SetFlexWrap>();

        // GridContent-specific
        app.register_responsive::<SetGridFlow>()
            .register_responsive::<GridAutoRows>()
            .register_responsive::<GridAutoColumns>()
            .register_responsive::<GridTemplateRows>()
            .register_responsive::<GridTemplateColumns>();

        // Self-shared
        app.register_animatable::<Margin>()
            .register_animatable::<Splat<Margin>>()
            .register_responsive::<SetJustifySelfCross>();

        // SelfFlex-specific
        app.register_animatable::<FlexBasis>()
            .register_animatable::<FlexGrow>()
            .register_animatable::<FlexShrink>();

        // SelfGrid-specific
        app.register_responsive::<GridRow>()
            .register_responsive::<GridColumn>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

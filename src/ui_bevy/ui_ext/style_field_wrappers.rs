use std::any::type_name;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

trait ApplyToStyle: Sized + Send + Sync + 'static
{
    fn apply_to_absolute(self, _style: &mut AbsoluteStyle, entity: Entity)
    {
        tracing::warn!("tried to apply {} to {:?} that has AbsoluteStyle; only FlexStyle is supported",
            type_name::<Self>(), entity);
    }
    fn apply_to_flex(self, style: &mut FlexStyle);
}

//-------------------------------------------------------------------------------------------------------------------

fn initialize_absolute_style(
    In(entity): In<Entity>,
    mut c: Commands,
    query: Query<(Has<React<AbsoluteStyle>>, Has<React<FlexStyle>>)>,
)
{
    let Ok((maybe_absolute, maybe_flex)) = query.get(entity) else { return };

    // Check absolute style.
    if maybe_absolute {
        return;
    }

    // Check flex style.
    if maybe_flex {
        tracing::warn!("tried initializing absolute style on entity {:?} that has flex style", entity);
        return;
    }

    // Insert absolute style.
    c.react().insert(entity, AbsoluteStyle::default());
}

//-------------------------------------------------------------------------------------------------------------------

fn initialize_flex_style(
    In(entity): In<Entity>,
    mut c: Commands,
    query: Query<(Has<React<AbsoluteStyle>>, Has<React<FlexStyle>>)>,
)
{
    let Ok((maybe_absolute, maybe_flex)) = query.get(entity) else { return };

    // Check flex style.
    if maybe_flex {
        return;
    }

    // Check absolute style.
    if maybe_absolute {
        tracing::warn!("tried initializing flex style on entity {:?} that has absolute style", entity);
        return;
    }

    // Insert flex style.
    c.react().insert(entity, FlexStyle::default());
}

//-------------------------------------------------------------------------------------------------------------------

fn remove_styles(entity: Entity, world: &mut World)
{
    let _ = world.get_entity_mut(entity).map(|mut e| {
        //e.remove::<(React<AbsoluteStyle>, React<FlexStyle>, Node)>();
        // TODO: need https://github.com/bevyengine/bevy/pull/16288 to remove Node
        e.remove::<(React<AbsoluteStyle>, React<FlexStyle>)>();
        e.insert(Node::default());
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_to_style<T: ApplyToStyle>(param: T, entity: Entity, world: &mut World)
{
    let Ok(mut emut) = world.get_entity_mut(entity) else { return };

    // Check flex style.
    if let Some(mut flex) = emut.get_mut::<React<FlexStyle>>() {
        param.apply_to_flex(&mut flex.get_noreact());
        React::<FlexStyle>::trigger_mutation(entity, world);
        return;
    }

    // Check absolute style.
    if let Some(mut absolute) = emut.get_mut::<React<AbsoluteStyle>>() {
        param.apply_to_absolute(&mut absolute.get_noreact(), entity);
        React::<AbsoluteStyle>::trigger_mutation(entity, world);
        return;
    }

    // Fall back to inserting flex style.
    let mut style = FlexStyle::default();
    param.apply_to_flex(&mut style);
    world.react(|rc| rc.insert(entity, style));
}

//-------------------------------------------------------------------------------------------------------------------

/// Initializes [`AbsoluteStyle`] on an entity.
///
/// This instruction should be inserted before all other style field wrappers.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct WithAbsoluteStyle;

impl Instruction for WithAbsoluteStyle
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(entity, initialize_absolute_style);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
    }
}
impl StaticAttribute for WithAbsoluteStyle
{
    type Value = ();
    fn construct(_: Self::Value) -> Self
    {
        Self
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Initializes [`FlexStyle`] on an entity.
///
/// This instruction should be inserted before all other style field wrappers.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct WithFlexStyle;

impl Instruction for WithFlexStyle
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(entity, initialize_flex_style);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
    }
}
impl StaticAttribute for WithFlexStyle
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

impl ApplyToStyle for Width
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.width = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.width = self.0;
    }
}

impl Instruction for Width
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for Width {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::height`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Height(pub Val);

impl ApplyToStyle for Height
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.height = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.height = self.0;
    }
}

impl Instruction for Height
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for Height {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::min_width`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MinWidth(pub Val);

impl ApplyToStyle for MinWidth
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.min_width = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.min_width = self.0;
    }
}

impl Instruction for MinWidth
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for MinWidth {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::min_height`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MinHeight(pub Val);

impl ApplyToStyle for MinHeight
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.min_height = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.min_height = self.0;
    }
}

impl Instruction for MinHeight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for MinHeight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::max_width`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MaxWidth(pub Val);

impl ApplyToStyle for MaxWidth
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.max_width = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.max_width = self.0;
    }
}

impl Instruction for MaxWidth
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for MaxWidth {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::max_height`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct MaxHeight(pub Val);

impl ApplyToStyle for MaxHeight
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.max_height = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.max_height = self.0;
    }
}

impl Instruction for MaxHeight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for MaxHeight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::aspect_ratio`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct AspectRatio(pub f32);

impl ApplyToStyle for AspectRatio
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.aspect_ratio = Some(self.0);
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.aspect_ratio = Some(self.0);
    }
}

impl Instruction for AspectRatio
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for AspectRatio {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::border`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Border(pub StyleRect);

impl ApplyToStyle for Border
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.border = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.border = self.0;
    }
}

impl Instruction for Border
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for Border {}

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

impl ApplyToStyle for DimsTop
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.top = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.top = self.0;
    }
}

impl Instruction for DimsTop
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for DimsTop {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::bottom`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct DimsBottom(pub Val);

impl ApplyToStyle for DimsBottom
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.bottom = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.bottom = self.0;
    }
}

impl Instruction for DimsBottom
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for DimsBottom {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::left`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct DimsLeft(pub Val);

impl ApplyToStyle for DimsLeft
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.left = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.left = self.0;
    }
}

impl Instruction for DimsLeft
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for DimsLeft {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::right`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct DimsRight(pub Val);

impl ApplyToStyle for DimsRight
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.right = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.right = self.0;
    }
}

impl Instruction for DimsRight
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for DimsRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::clipping`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetClipping(pub Clipping);

impl ApplyToStyle for SetClipping
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.clipping = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.clipping = self.0;
    }
}

impl Instruction for SetClipping
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Padding(pub StyleRect);

impl ApplyToStyle for Padding
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.padding = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.padding = self.0;
    }
}

impl Instruction for Padding
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for Padding {}

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

impl ApplyToStyle for SetFlexDirection
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.flex_direction = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.flex_direction = self.0;
    }
}

impl Instruction for SetFlexDirection
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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

impl ApplyToStyle for SetFlexWrap
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.flex_wrap = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.flex_wrap = self.0;
    }
}

impl Instruction for SetFlexWrap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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

impl ApplyToStyle for SetJustifyLines
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.justify_lines = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.justify_lines = self.0;
    }
}

impl Instruction for SetJustifyLines
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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

impl ApplyToStyle for SetJustifyMain
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.justify_main = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.justify_main = self.0;
    }
}

impl Instruction for SetJustifyMain
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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

impl ApplyToStyle for SetJustifyCross
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.justify_cross = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.justify_cross = self.0;
    }
}

impl Instruction for SetJustifyCross
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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

impl ApplyToStyle for ColumnGap
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.column_gap = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.column_gap = self.0;
    }
}

impl Instruction for ColumnGap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for ColumnGap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::row_gap`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct RowGap(pub Val);

impl ApplyToStyle for RowGap
{
    fn apply_to_absolute(self, style: &mut AbsoluteStyle, _: Entity)
    {
        style.row_gap = self.0;
    }
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.row_gap = self.0;
    }
}

impl Instruction for RowGap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for RowGap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::margin`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Margin(pub StyleRect);

impl ApplyToStyle for Margin
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.margin = self.0;
    }
}

impl Instruction for Margin
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for Margin {}

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

impl ApplyToStyle for FlexBasis
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.flex_basis = self.0;
    }
}

impl Instruction for FlexBasis
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for FlexBasis {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::flex_grow`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct FlexGrow(pub f32);

impl ApplyToStyle for FlexGrow
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.flex_grow = self.0;
    }
}

impl Instruction for FlexGrow
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for FlexGrow {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::flex_shrink`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct FlexShrink(pub f32);

impl ApplyToStyle for FlexShrink
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.flex_shrink = self.0;
    }
}

impl Instruction for FlexShrink
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
impl AnimatableAttribute for FlexShrink {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::justify_self_cross`], can be loaded as an instruction.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SetJustifySelfCross(pub JustifySelfCross);

impl ApplyToStyle for SetJustifySelfCross
{
    // no apply_to_absolute, absolute not supported for SelfFlex fields
    fn apply_to_flex(self, style: &mut FlexStyle)
    {
        style.justify_self_cross = self.0;
    }
}

impl Instruction for SetJustifySelfCross
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        apply_to_style(self, entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        remove_styles(entity, world);
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
        app.register_themed::<WithAbsoluteStyle>();
        app.register_themed::<WithFlexStyle>();

        // Dims
        app.register_animatable::<Width>()
            .register_animatable::<Height>()
            .register_animatable::<MinWidth>()
            .register_animatable::<MinHeight>()
            .register_animatable::<MaxWidth>()
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

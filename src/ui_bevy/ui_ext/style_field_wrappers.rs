use std::any::type_name;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

trait ApplyToDims: Send + Sync + 'static
{
    fn apply_to_dims(self, dims: &mut Dims);
}

trait ApplyToContentFlex: Send + Sync + 'static
{
    fn apply_to_content_flex(self, content: &mut ContentFlex);
}

trait ApplyToSelfFlex: Send + Sync + 'static
{
    fn apply_to_self_flex(self, flex: &mut SelfFlex);
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

fn apply_to_dims<T: ApplyToDims>(
    In((entity, param)): In<(Entity, T)>,
    mut c: Commands,
    mut query: Query<(Option<&mut React<AbsoluteStyle>>, Option<&mut React<FlexStyle>>)>,
)
{
    let Ok((maybe_absolute, maybe_flex)) = query.get_mut(entity) else { return };

    // Prioritize absolute style.
    if let Some(mut absolute) = maybe_absolute {
        param.apply_to_dims(&mut absolute.get_mut(&mut c).dims);
        return;
    }

    // Check flex style.
    if let Some(mut flex) = maybe_flex {
        param.apply_to_dims(&mut flex.get_mut(&mut c).dims);
        return;
    }

    // Fall back to inserting flex style.
    let mut style = FlexStyle::default();
    param.apply_to_dims(&mut style.dims);
    c.react().insert(entity, style);
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_to_content_flex<T: ApplyToContentFlex>(
    In((entity, param)): In<(Entity, T)>,
    mut c: Commands,
    mut query: Query<(Option<&mut React<AbsoluteStyle>>, Option<&mut React<FlexStyle>>)>,
)
{
    let Ok((maybe_absolute, maybe_flex)) = query.get_mut(entity) else { return };

    // Prioritize absolute style.
    if let Some(mut absolute) = maybe_absolute {
        param.apply_to_content_flex(&mut absolute.get_mut(&mut c).content);
        return;
    }

    // Check flex style.
    if let Some(mut flex) = maybe_flex {
        param.apply_to_content_flex(&mut flex.get_mut(&mut c).content);
        return;
    }

    // Fall back to inserting flex style.
    let mut style = FlexStyle::default();
    param.apply_to_content_flex(&mut style.content);
    c.react().insert(entity, style);
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_to_self_flex<T: ApplyToSelfFlex>(
    In((entity, param)): In<(Entity, T)>,
    mut c: Commands,
    mut query: Query<(Has<React<AbsoluteStyle>>, Option<&mut React<FlexStyle>>)>,
)
{
    let Ok((has_absolute, maybe_flex)) = query.get_mut(entity) else { return };

    // Check absolute style.
    if has_absolute {
        tracing::warn!("tried to apply {} to {:?} that has AbsoluteStyle; only FlexStyle is supported",
            type_name::<T>(), entity);
        return;
    }

    // Check flex style.
    if let Some(mut flex) = maybe_flex {
        param.apply_to_self_flex(&mut flex.get_mut(&mut c).flex);
        return;
    }

    // Fall back to inserting flex style.
    let mut style = FlexStyle::default();
    param.apply_to_self_flex(&mut style.flex);
    c.react().insert(entity, style);
}

//-------------------------------------------------------------------------------------------------------------------

/// Initializes [`AbsoluteStyle`] on an entity.
///
/// Should be inserted before all other style field wrappers.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithAbsoluteStyle;

impl ApplyLoadable for WithAbsoluteStyle
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall(id, initialize_absolute_style);
    }
}
impl ThemedAttribute for WithAbsoluteStyle
{
    type Value = ();
    fn update(ec: &mut EntityCommands, _value: Self::Value)
    {
        Self.apply(ec);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Initializes [`FlexStyle`] on an entity.
///
/// Should be inserted before all other style field wrappers.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithFlexStyle;

impl ApplyLoadable for WithFlexStyle
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall(id, initialize_flex_style);
    }
}
impl ThemedAttribute for WithFlexStyle
{
    type Value = ();
    fn update(ec: &mut EntityCommands, _value: Self::Value)
    {
        Self.apply(ec);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::width`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Width(pub Val);

impl ApplyToDims for Width
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.width = self.0;
    }
}

impl ApplyLoadable for Width
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for Width
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for Width {}
impl AnimatableAttribute for Width {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::height`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Height(pub Val);

impl ApplyToDims for Height
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.height = self.0;
    }
}

impl ApplyLoadable for Height
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for Height
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for Height {}
impl AnimatableAttribute for Height {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::min_width`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinWidth(pub Val);

impl ApplyToDims for MinWidth
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.min_width = self.0;
    }
}

impl ApplyLoadable for MinWidth
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for MinWidth
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for MinWidth {}
impl AnimatableAttribute for MinWidth {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::min_height`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinHeight(pub Val);

impl ApplyToDims for MinHeight
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.min_height = self.0;
    }
}

impl ApplyLoadable for MinHeight
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for MinHeight
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for MinHeight {}
impl AnimatableAttribute for MinHeight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::max_width`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaxWidth(pub Val);

impl ApplyToDims for MaxWidth
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.max_width = self.0;
    }
}

impl ApplyLoadable for MaxWidth
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for MaxWidth
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for MaxWidth {}
impl AnimatableAttribute for MaxWidth {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::max_height`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaxHeight(pub Val);

impl ApplyToDims for MaxHeight
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.max_height = self.0;
    }
}

impl ApplyLoadable for MaxHeight
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for MaxHeight
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for MaxHeight {}
impl AnimatableAttribute for MaxHeight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::aspect_ratio`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AspectRatio(pub f32);

impl ApplyToDims for AspectRatio
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.aspect_ratio = Some(self.0);
    }
}

impl ApplyLoadable for AspectRatio
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for AspectRatio
{
    type Value = f32;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for AspectRatio {}
impl AnimatableAttribute for AspectRatio {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::border`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Border(pub StyleRect);

impl ApplyToDims for Border
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.border = self.0;
    }
}

impl ApplyLoadable for Border
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for Border
{
    type Value = StyleRect;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
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

/// Mirrors [`Dims::top`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimsTop(pub Val);

impl ApplyToDims for DimsTop
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.top = self.0;
    }
}

impl ApplyLoadable for DimsTop
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for DimsTop
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsTop {}
impl AnimatableAttribute for DimsTop {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::bottom`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimsBottom(pub Val);

impl ApplyToDims for DimsBottom
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.bottom = self.0;
    }
}

impl ApplyLoadable for DimsBottom
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for DimsBottom
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsBottom {}
impl AnimatableAttribute for DimsBottom {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::left`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimsLeft(pub Val);

impl ApplyToDims for DimsLeft
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.left = self.0;
    }
}

impl ApplyLoadable for DimsLeft
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for DimsLeft
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsLeft {}
impl AnimatableAttribute for DimsLeft {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Dims::right`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimsRight(pub Val);

impl ApplyToDims for DimsRight
{
    fn apply_to_dims(self, dims: &mut Dims)
    {
        dims.right = self.0;
    }
}

impl ApplyLoadable for DimsRight
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_dims);
    }
}

impl ThemedAttribute for DimsRight
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsRight {}
impl AnimatableAttribute for DimsRight {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::clipping`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetClipping(pub Clipping);

impl ApplyToContentFlex for SetClipping
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.clipping = self.0;
    }
}

impl ApplyLoadable for SetClipping
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for SetClipping
{
    type Value = Clipping;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetClipping {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::padding`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Padding(pub StyleRect);

impl ApplyToContentFlex for Padding
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.padding = self.0;
    }
}

impl ApplyLoadable for Padding
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for Padding
{
    type Value = StyleRect;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
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

/// Mirrors [`ContentFlex::flex_direction`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFlexDirection(pub FlexDirection);

impl ApplyToContentFlex for SetFlexDirection
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.flex_direction = self.0;
    }
}

impl ApplyLoadable for SetFlexDirection
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for SetFlexDirection
{
    type Value = FlexDirection;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetFlexDirection {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::flex_wrap`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFlexWrap(pub FlexWrap);

impl ApplyToContentFlex for SetFlexWrap
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.flex_wrap = self.0;
    }
}

impl ApplyLoadable for SetFlexWrap
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for SetFlexWrap
{
    type Value = FlexWrap;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetFlexWrap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_lines`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetJustifyLines(pub JustifyLines);

impl ApplyToContentFlex for SetJustifyLines
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.justify_lines = self.0;
    }
}

impl ApplyLoadable for SetJustifyLines
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for SetJustifyLines
{
    type Value = JustifyLines;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetJustifyLines {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_main`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetJustifyMain(pub JustifyMain);

impl ApplyToContentFlex for SetJustifyMain
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.justify_main = self.0;
    }
}

impl ApplyLoadable for SetJustifyMain
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for SetJustifyMain
{
    type Value = JustifyMain;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetJustifyMain {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_cross`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetJustifyCross(pub JustifyCross);

impl ApplyToContentFlex for SetJustifyCross
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.justify_cross = self.0;
    }
}

impl ApplyLoadable for SetJustifyCross
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for SetJustifyCross
{
    type Value = JustifyCross;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetJustifyCross {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::text_direction`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetTextDirection(pub Direction);

impl ApplyToContentFlex for SetTextDirection
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.text_direction = self.0;
    }
}

impl ApplyLoadable for SetTextDirection
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for SetTextDirection
{
    type Value = Direction;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetTextDirection {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::column_gap`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnGap(pub Val);

impl ApplyToContentFlex for ColumnGap
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.column_gap = self.0;
    }
}

impl ApplyLoadable for ColumnGap
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for ColumnGap
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for ColumnGap {}
impl AnimatableAttribute for ColumnGap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::row_gap`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowGap(pub Val);

impl ApplyToContentFlex for RowGap
{
    fn apply_to_content_flex(self, content: &mut ContentFlex)
    {
        content.row_gap = self.0;
    }
}

impl ApplyLoadable for RowGap
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_content_flex);
    }
}

impl ThemedAttribute for RowGap
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for RowGap {}
impl AnimatableAttribute for RowGap {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::margin`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Margin(pub StyleRect);

impl ApplyToSelfFlex for Margin
{
    fn apply_to_self_flex(self, content: &mut SelfFlex)
    {
        content.margin = self.0;
    }
}

impl ApplyLoadable for Margin
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_self_flex);
    }
}

impl ThemedAttribute for Margin
{
    type Value = StyleRect;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
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

/// Mirrors [`SelfFlex::flex_basis`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlexBasis(pub Val);

impl ApplyToSelfFlex for FlexBasis
{
    fn apply_to_self_flex(self, content: &mut SelfFlex)
    {
        content.flex_basis = self.0;
    }
}

impl ApplyLoadable for FlexBasis
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_self_flex);
    }
}

impl ThemedAttribute for FlexBasis
{
    type Value = Val;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for FlexBasis {}
impl AnimatableAttribute for FlexBasis {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::flex_grow`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlexGrow(pub f32);

impl ApplyToSelfFlex for FlexGrow
{
    fn apply_to_self_flex(self, content: &mut SelfFlex)
    {
        content.flex_grow = self.0;
    }
}

impl ApplyLoadable for FlexGrow
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_self_flex);
    }
}

impl ThemedAttribute for FlexGrow
{
    type Value = f32;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for FlexGrow {}
impl AnimatableAttribute for FlexGrow {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::flex_shrink`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlexShrink(pub f32);

impl ApplyToSelfFlex for FlexShrink
{
    fn apply_to_self_flex(self, content: &mut SelfFlex)
    {
        content.flex_shrink = self.0;
    }
}

impl ApplyLoadable for FlexShrink
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_self_flex);
    }
}

impl ThemedAttribute for FlexShrink
{
    type Value = f32;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
    }
}
impl ResponsiveAttribute for FlexShrink {}
impl AnimatableAttribute for FlexShrink {}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`SelfFlex::justify_self_cross`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetJustifySelfCross(pub JustifySelfCross);

impl ApplyToSelfFlex for SetJustifySelfCross
{
    fn apply_to_self_flex(self, content: &mut SelfFlex)
    {
        content.justify_self_cross = self.0;
    }
}

impl ApplyLoadable for SetJustifySelfCross
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), apply_to_self_flex);
    }
}

impl ThemedAttribute for SetJustifySelfCross
{
    type Value = JustifySelfCross;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        Self(value).apply(ec);
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
            .register_responsive::<SetTextDirection>()
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

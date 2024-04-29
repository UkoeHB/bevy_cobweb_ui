use std::any::type_name;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

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

    // Fall back to inserting absolute style.
    let mut style = AbsoluteStyle::default();
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

    // Fall back to inserting absolute style.
    let mut style = AbsoluteStyle::default();
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
        Width(value).apply(ec);
    }
}
impl ResponsiveAttribute for Width
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for Width
{
    type Interactive = Interactive;
}

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
        Height(value).apply(ec);
    }
}
impl ResponsiveAttribute for Height
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for Height
{
    type Interactive = Interactive;
}

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
        MinWidth(value).apply(ec);
    }
}
impl ResponsiveAttribute for MinWidth
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for MinWidth
{
    type Interactive = Interactive;
}

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
        MinHeight(value).apply(ec);
    }
}
impl ResponsiveAttribute for MinHeight
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for MinHeight
{
    type Interactive = Interactive;
}

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
        MaxWidth(value).apply(ec);
    }
}
impl ResponsiveAttribute for MaxWidth
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for MaxWidth
{
    type Interactive = Interactive;
}

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
        MaxHeight(value).apply(ec);
    }
}
impl ResponsiveAttribute for MaxHeight
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for MaxHeight
{
    type Interactive = Interactive;
}

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
        AspectRatio(value).apply(ec);
    }
}
impl ResponsiveAttribute for AspectRatio
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for AspectRatio
{
    type Interactive = Interactive;
}

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
        Border(value).apply(ec);
    }
}
impl ResponsiveAttribute for Border
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for Border
{
    type Interactive = Interactive;
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
        DimsTop(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsTop
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for DimsTop
{
    type Interactive = Interactive;
}

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
        DimsBottom(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsBottom
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for DimsBottom
{
    type Interactive = Interactive;
}

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
        DimsLeft(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsLeft
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for DimsLeft
{
    type Interactive = Interactive;
}

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
        DimsRight(value).apply(ec);
    }
}
impl ResponsiveAttribute for DimsRight
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for DimsRight
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::clipping`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetClipping(Clipping);

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
impl ResponsiveAttribute for SetClipping
{
    type Interactive = Interactive;
}

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
        Padding(value).apply(ec);
    }
}
impl ResponsiveAttribute for Padding
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for Padding
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::flex_direction`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFlexDirection(FlexDirection);

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
impl ResponsiveAttribute for SetFlexDirection
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::flex_wrap`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFlexWrap(FlexWrap);

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
impl ResponsiveAttribute for SetFlexWrap
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_lines`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetJustifyLines(JustifyLines);

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
impl ResponsiveAttribute for SetJustifyLines
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_main`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetJustifyMain(JustifyMain);

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
impl ResponsiveAttribute for SetJustifyMain
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::justify_cross`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetJustifyCross(JustifyCross);

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
impl ResponsiveAttribute for SetJustifyCross
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`ContentFlex::text_direction`], can be loaded as a style.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetTextDirection(Direction);

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
impl ResponsiveAttribute for SetTextDirection
{
    type Interactive = Interactive;
}

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
        ColumnGap(value).apply(ec);
    }
}
impl ResponsiveAttribute for ColumnGap
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for ColumnGap
{
    type Interactive = Interactive;
}

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
        RowGap(value).apply(ec);
    }
}
impl ResponsiveAttribute for RowGap
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for RowGap
{
    type Interactive = Interactive;
}

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
        Margin(value).apply(ec);
    }
}
impl ResponsiveAttribute for Margin
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for Margin
{
    type Interactive = Interactive;
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
        FlexBasis(value).apply(ec);
    }
}
impl ResponsiveAttribute for FlexBasis
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for FlexBasis
{
    type Interactive = Interactive;
}

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
        FlexGrow(value).apply(ec);
    }
}
impl ResponsiveAttribute for FlexGrow
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for FlexGrow
{
    type Interactive = Interactive;
}

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
        FlexShrink(value).apply(ec);
    }
}
impl ResponsiveAttribute for FlexShrink
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for FlexShrink
{
    type Interactive = Interactive;
}

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
        SetJustifySelfCross(value).apply(ec);
    }
}
impl ResponsiveAttribute for SetJustifySelfCross
{
    type Interactive = Interactive;
}

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
            .register_animatable::<DimsTop>()
            .register_animatable::<DimsBottom>()
            .register_animatable::<DimsLeft>()
            .register_animatable::<DimsRight>();

        // ContentFlex
        app.register_responsive::<SetClipping>()
            .register_animatable::<Padding>()
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
            .register_animatable::<FlexBasis>()
            .register_animatable::<FlexGrow>()
            .register_animatable::<FlexShrink>()
            .register_responsive::<SetJustifySelfCross>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

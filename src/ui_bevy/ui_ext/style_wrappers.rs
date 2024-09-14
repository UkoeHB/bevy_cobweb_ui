use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::lerp::Lerp;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiRect`] for stylesheet serialization.
///
/// All fields default to `Val::Px(0.)`.
#[derive(Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleRect
{
    #[reflect(default = "StyleRect::default_field")]
    pub top: Val,
    #[reflect(default = "StyleRect::default_field")]
    pub bottom: Val,
    #[reflect(default = "StyleRect::default_field")]
    pub left: Val,
    #[reflect(default = "StyleRect::default_field")]
    pub right: Val,
}

impl StyleRect
{
    fn default_field() -> Val
    {
        Val::Px(0.)
    }

    /// Constructs a style rect with all sides equal to `single`.
    pub fn splat(single: Val) -> Self
    {
        Self { top: single, bottom: single, left: single, right: single }
    }
}

impl Into<UiRect> for StyleRect
{
    fn into(self) -> UiRect
    {
        UiRect {
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
        }
    }
}

impl Default for StyleRect
{
    fn default() -> Self
    {
        Self {
            top: Self::default_field(),
            bottom: Self::default_field(),
            left: Self::default_field(),
            right: Self::default_field(),
        }
    }
}

impl Lerp for StyleRect
{
    fn lerp(&self, to: Self, t: f32) -> Self
    {
        Self {
            left: self.left.lerp(to.left, t),
            right: self.right.lerp(to.right, t),
            top: self.top.lerp(to.top, t),
            bottom: self.bottom.lerp(to.bottom, t),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Overflow`] for stylesheet serialization.
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Clipping
{
    #[default]
    None,
    ClipX,
    ClipY,
    ClipXY,
}

impl Into<Overflow> for Clipping
{
    fn into(self) -> Overflow
    {
        match self {
            Self::None => Overflow { x: OverflowAxis::Visible, y: OverflowAxis::Visible },
            Self::ClipX => Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Visible },
            Self::ClipY => Overflow { x: OverflowAxis::Visible, y: OverflowAxis::Clip },
            Self::ClipXY => Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Clip },
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls cross-axis alignment of the parallel rectangular sections where lines of children are arranged after
/// wrapping.
///
/// Does nothing if there are no wrapped lines.
///
/// Mirrors [`AlignContent`].
/// Excludes [`AlignContent::Start`] and [`AlignContent::End`] which are equivalent to the `FlexStart`/`FlexEnd`
/// variants (except when [`FlexWrap::WrapReverse`] is used, but don't use that).
///
/// Defaults to [`Self::FlexStart`].
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum JustifyLines
{
    /// Pack lines toward the start of the cross axis.
    ///
    /// Affected by [`ContentFlex::text_direction`] for [`FlexDirection::Column`].
    #[default]
    FlexStart,
    /// Pack lines toward the end of the cross axis.
    ///
    /// Affected by [`ContentFlex::text_direction`] for [`FlexDirection::Column`].
    FlexEnd,
    /// Pack lines toward the center of the cross axis.
    Center,
    /// Stretches the cross-axis lengths of lines of children. Lines are stretched to be equal in size if
    /// possible.
    ///
    /// The 'pre-stretch' size of a section is equal in main-length to the parent, and equal in cross-length to
    /// its largest pre-stretch child.
    Stretch,
    /// Even gaps between each line. No gap at the ends.
    SpaceBetween,
    /// Add space between each line and the ends.
    ///
    /// There will be one layer of space at the ends and one layer between each line.
    SpaceEvenly,
    /// Add space on each side of each line.
    ///
    /// There will be one layer of space at the ends and two layers between each line.
    SpaceAround,
}

impl Into<AlignContent> for JustifyLines
{
    fn into(self) -> AlignContent
    {
        match self {
            Self::FlexStart => AlignContent::FlexStart,
            Self::FlexEnd => AlignContent::FlexEnd,
            Self::Center => AlignContent::Center,
            Self::Stretch => AlignContent::Stretch,
            Self::SpaceBetween => AlignContent::SpaceBetween,
            Self::SpaceEvenly => AlignContent::SpaceEvenly,
            Self::SpaceAround => AlignContent::SpaceAround,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls alignment of children on the main axis within each wrapping line.
///
/// Has no effect in a line if at least one child in the line has `SelfFlex::flex_grow > 0`, since all space will
/// be taken up by flexing children.
///
/// Mirrors [`JustifyContent`].
/// Excludes [`JustifyContent::Default`] which is equivalent to `FlexStart`.
/// Excludes [`JustifyContent::Stretch`] which is only used for CSS Grid layouts (use [`SelfFlex::flex_grow`]/
/// [`SelfFlex::flex_shrink`] instead).
/// Excludes [`JustifyContent::Start`] and [`JustifyContent::End`], which are equivalent to the
/// `FlexStart`/`FlexEnd` variants for everything except [`FlexDirection::RowReverse`], where the `Start`/`End`
/// variants have the same behavior as for [`FlexDirection::Row`]. (There is additional complexity when
/// [`FlexWrap::WrapReverse`] is used, but don't use that.)
///
/// Defaults to [`Self::FlexStart`].
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum JustifyMain
{
    /*
    /// Cluster items at the start of the main axis.
    /// - [`FlexDirection::Row`]: Start according to [`ContentFlex::text_direction`].
    /// - [`FlexDirection::ReverseRow`]: Start according to [`ContentFlex::text_direction`]. ** difference
    /// - [`FlexDirection::Column`]: Top.
    /// - [`FlexDirection::ColumnReverse`]: Bottom.
    Start,
    /// Cluster items at the end of the main axis.
    /// - [`FlexDirection::Row`]: End according to [`ContentFlex::text_direction`].
    /// - [`FlexDirection::ReverseRow`]: End according to [`ContentFlex::text_direction`]. ** difference
    /// - [`FlexDirection::Column`]: Bottom.
    /// - [`FlexDirection::ColumnReverse`]: Top.
    End,
    */
    /// Cluster items at the start of the main axis.
    /// - [`FlexDirection::Row`]: Start according to [`ContentFlex::text_direction`].
    /// - [`FlexDirection::RowReverse`]: End according to [`ContentFlex::text_direction`].
    /// - [`FlexDirection::Column`]: Top.
    /// - [`FlexDirection::ColumnReverse`]: Bottom.
    #[default]
    FlexStart,
    /// Cluster items at the end of the main axis.
    /// - [`FlexDirection::Row`]: End according to [`ContentFlex::text_direction`].
    /// - [`FlexDirection::RowReverse`]: Start according to [`ContentFlex::text_direction`].
    /// - [`FlexDirection::Column`]: Bottom.
    /// - [`FlexDirection::ColumnReverse`]: Top.
    FlexEnd,
    /// Cluster items in the center of the main axis.
    Center,
    /// Even gaps between each child on the main axis. No gap at the ends.
    SpaceBetween,
    /// Add space between each child and the ends.
    ///
    /// There will be one layer of space at the ends and one layer between each child.
    SpaceEvenly,
    /// Add space on each side of each child on the main axis.
    ///
    /// There will be one layer of space at the ends and two layers between each child.
    SpaceAround,
}

impl Into<JustifyContent> for JustifyMain
{
    fn into(self) -> JustifyContent
    {
        match self {
            Self::FlexStart => JustifyContent::FlexStart,
            Self::FlexEnd => JustifyContent::FlexEnd,
            Self::Center => JustifyContent::Center,
            Self::SpaceBetween => JustifyContent::SpaceBetween,
            Self::SpaceEvenly => JustifyContent::SpaceEvenly,
            Self::SpaceAround => JustifyContent::SpaceAround,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets the default cross-axis alignment of children within each wrapping line.
///
/// Mirrors [`AlignItems`].
/// Excludes [`AlignItems::Baseline`] which is too confusing to use easily.
/// Excludes [`AlignItems::Default`] which is usually [`Self::Stretch`] but sometimes [`Self::FlexStart`].
/// Excludes [`AlignItems::Start`] and [`AlignItems::End`] which are equivalent to the `FlexStart`/`FlexEnd`
/// variants (except when [`FlexWrap::WrapReverse`] is used, but don't use that).
///
/// Defaults to [`Self::FlexStart`].
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum JustifyCross
{
    /// Align children to the start of the cross axis in each line.
    #[default]
    FlexStart,
    /// Align children to the end of the cross axis in each line.
    FlexEnd,
    /// Align children to the center of the cross axis in each line.
    Center,
    /// Children along the cross-axis are stretched to fill the available space on that axis (respecting min/max
    /// limits).
    ///
    /// If a cross-axis has multiple lines due to [`ContentFlex::flex_wrap`], stretching will only fill a given
    /// line without overflow.
    ///
    /// Stretch is applied after other sizing and positioning is calculated. It's a kind of 'bonus sizing'.
    ///
    /// If using [`AbsoluteStyle`] and [`Dims::top`]/[`Dims::bottom`]/[`Dims::left`]/[`Dims::right`] are set to
    /// all auto, then this falls back to [`JustifyCross::FlexStart`].
    Stretch,
}

impl Into<AlignItems> for JustifyCross
{
    fn into(self) -> AlignItems
    {
        match self {
            Self::FlexStart => AlignItems::FlexStart,
            Self::FlexEnd => AlignItems::FlexEnd,
            Self::Center => AlignItems::Center,
            Self::Stretch => AlignItems::Stretch,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets the cross-axis alignment of a node, overriding its parent's [`JustifyCross`] setting.
///
/// Mirrors [`AlignSelf`].
/// Excludes [`AlignSelf::Baseline`] which is too confusing to use easily.
/// Excludes [`AlignSelf::Start`] and [`AlignSelf::End`] which are equivalent to the `FlexStart`/`FlexEnd` variants
/// (except when [`FlexWrap::WrapReverse`] is used, but don't use that).
///
/// Defaults to [`Self::Auto`].
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum JustifySelfCross
{
    /// Adopt the parent's [`JustifyCross`] setting.
    #[default]
    Auto,
    /// Align self to the start of the cross axis in the line where it resides.
    FlexStart,
    /// Align self to the end of the cross axis in the line where it resides.
    FlexEnd,
    /// Align self to the center of the cross axis in the line where it resides.
    Center,
    /// See [`JustifyCross::Stretch`].
    Stretch,
}

impl Into<AlignSelf> for JustifySelfCross
{
    fn into(self) -> AlignSelf
    {
        match self {
            Self::Auto => AlignSelf::Auto,
            Self::FlexStart => AlignSelf::FlexStart,
            Self::FlexEnd => AlignSelf::FlexEnd,
            Self::Center => AlignSelf::Center,
            Self::Stretch => AlignSelf::Stretch,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls a node's size and offset.
///
/// Mirrors fields in [`Style`].
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dims
{
    /// Indicates the `desired` width of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'content-sized'.
    ///
    /// If set to non-[`Val::Auto`], then the desired width will be overriden if:
    /// - [`FlexStyle`]: If [`FlexDirection::Row`]/[`FlexDirection::RowReverse`] is set and
    ///   [`SelfFlex::flex_basis`] is set to non-auto.
    ///
    /// If set to [`Val::Auto`], then the desired width will be overriden if:
    /// - [`AbsoluteStyle`]: [`Dims::left`] and [`Dims::right`] are set.
    /// - [`FlexStyle`]: Parent is using [`FlexDirection::Column`]/[`FlexDirection::ColumnReverse`] and
    ///   [`JustifyCross::Stretch`]. Or, if parent is using [`FlexDirection::Row`]/[`FlexDirection::RowReverse`]
    ///   and self is using [`SelfFlex::flex_grow`]/[`SelfFlex::flex_shrink`].
    #[reflect(default)]
    pub width: Val,
    /// Indicates the `desired` height of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'content-sized'.
    ///
    /// If set to non-[`Val::Auto`], then the desired height will be overriden if:
    /// - [`FlexStyle`]: If [`FlexDirection::Column`]/[`FlexDirection::ColumnReverse`] is set and
    ///   [`SelfFlex::flex_basis`] is set to non-auto.
    ///
    /// If set to [`Val::Auto`], then the desired height will be overriden if:
    /// - [`AbsoluteStyle`]: [`Dims::top`] and [`Dims::bottom`] are set.
    /// - [`FlexStyle`]: Parent is using [`FlexDirection::Row`]/[`FlexDirection::RowReverse`] and
    ///   [`JustifyCross::Stretch`]. Or, if the parent is using
    ///   [`FlexDirection::Column`]/[`FlexDirection::ColumnReverse`] and self is using
    ///   [`SelfFlex::flex_grow`]/[`SelfFlex::flex_shrink`].
    #[reflect(default)]
    pub height: Val,
    /// Controls the absolute maximum width of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'infinite'.
    #[reflect(default)]
    pub max_width: Val,
    /// Controls the absolute maximum height of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'infinite'.
    #[reflect(default)]
    pub max_height: Val,
    /// Controls the absolute minimum width of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'same as `width`'.
    #[reflect(default)]
    pub min_width: Val,
    /// Controls the absolute minimum height of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'same as `height`'.
    #[reflect(default)]
    pub min_height: Val,
    /// Forces a specific `width/height` ratio.
    ///
    /// Only takes effect if at least one of [`Self::width`] or [`Self::height`] is set to [`Val::Auto`].
    ///
    /// - [`AbsoluteStyle`]: If `width`/`height` are set to auto and all offset fields are set, then the offset's
    ///   `bottom` parameter will be ignored and the aspect ratio will use the `left`/`right` controlled width.
    /// - [`FlexStyle`]: [`SelfFlex::flex_basis`] can override the width/height, which affects whether they are
    ///   considered 'auto'.
    #[reflect(default)]
    pub aspect_ratio: Option<f32>,
    /// Region between a node's boundary and its padding.
    ///
    /// See [`BorderColor`] for a typical use-case.
    ///
    /// Defaults to zero border.
    #[reflect(default)]
    pub border: StyleRect,
    /// Position offsets applied to the edges of a node's margin.
    /// - [`AbsoluteStyle`] (absolute): Relative to its parent's boundary (ignoring padding). Can be used to
    ///   resize the node if `width`/`height` is set to auto and both `left`/`right` or `top`/`bottom` are set.
    /// - [`FlexStyle`] (relative): Relative to the final position of the edges of its margin after layout is
    ///   calculated. Does not affect the layout of other nodes. Cannot be used to resize the node (see note
    ///   following).
    ///
    /// If both `left` and `right` are set, then `right` will be overridden by the `width` field unless it is
    /// [`Val::Auto`]. The same goes for `top`/`bottom`/`height`. In practice this means [`FlexStyle`] cannot
    /// use both `left`/`right` or `top`/`bottom` parameters since if `width`/`height` are auto then the
    /// layout algorithm will control the node size.
    ///
    /// Defaults to `StyleRect{ left: Val::Pixels(0.), top: Val::Pixels(0.), right: Val::Auto, left: Val::Auto }`.
    /// This ensures [`AbsoluteStyle`] nodes will start in the upper left corner of their parents. If the
    /// offset is set to all [`Val::Auto`] then the node's position will be controlled by its parent's
    /// [`ContentFlex`] parameters. You must set the `left`/`top` fields to auto if using [`FlexStyle`] and
    /// you want to use `right`/`bottom`.
    #[reflect(default = "Dims::default_top")]
    pub top: Val,
    /// See [`Self::top`].
    #[reflect(default)]
    pub bottom: Val,
    /// See [`Self::top`].
    #[reflect(default = "Dims::default_left")]
    pub left: Val,
    /// See [`Self::top`].
    #[reflect(default)]
    pub right: Val,
}

impl Dims
{
    /// Adds this struct's contents to [`Style`].
    pub fn set_in_style(&self, style: &mut Style)
    {
        style.width = self.width;
        style.height = self.height;
        style.max_width = self.max_width;
        style.max_height = self.max_height;
        style.min_width = self.min_width;
        style.min_height = self.min_height;
        style.aspect_ratio = self.aspect_ratio;
        style.border = self.border.into();
        style.left = self.left;
        style.right = self.right;
        style.top = self.top;
        style.bottom = self.bottom;
    }

    fn default_top() -> Val
    {
        Val::Px(0.)
    }
    fn default_left() -> Val
    {
        Val::Px(0.)
    }
}

impl Default for Dims
{
    fn default() -> Self
    {
        Self {
            width: Default::default(),
            height: Default::default(),
            max_width: Default::default(),
            max_height: Default::default(),
            min_width: Default::default(),
            min_height: Default::default(),
            aspect_ratio: Default::default(),
            border: Default::default(),
            top: Dims::default_top(),
            bottom: Default::default(),
            left: Dims::default_left(),
            right: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls the layout of a node's children.
///
/// Mirrors fields in [`Style`].
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentFlex
{
    /// Determines whether the node contents will be clipped at the node boundary.
    ///
    /// Defaults to no clipping.
    #[reflect(default)]
    pub clipping: Clipping,
    /// Inserts space between the node's [`Dims::border`] and its contents.
    ///
    /// Defaults to zero padding.
    #[reflect(default)]
    pub padding: StyleRect,
    /// Controls which direction the main flex axis points within this node.
    ///
    /// - [`FlexDirection::Row`]: same direction as [`Self::text_direction`], flex wrapped lines are added down
    /// - [`FlexDirection::Column`]: top-to-bottom, flex wrapped rows are added in [`Self::text_direction`]
    /// - [`FlexDirection::RowReverse`]: opposite direction to [`Self::text_direction`], flex wrapped rows are
    ///   added down
    /// - [`FlexDirection::ColumnReverse`]: bottom-to-top, flex wrapped rows are added in [`Self::text_direction`]
    #[reflect(default)]
    pub flex_direction: FlexDirection,
    /// Controls whether children should wrap to multiple lines when overflowing the main axis.
    ///
    /// If children wrap, then wrapping lines can potentially overflow the cross axis.
    ///
    /// It is not recommended to use [`FlexWrap::WrapReverse`] unless you are prepared for the added complexity of
    /// figuring out how
    /// [`JustifyMain`]/[`JustifyCross`]/[`JustifyLines`]/[`ContentFlex::text_direction`]/[`FlexDirection`]
    /// interlace with it to produce the final layout.
    ///
    /// Defaults to [`FlexWrap::NoWrap`].
    #[reflect(default = "ContentFlex::default_flex_wrap")]
    pub flex_wrap: FlexWrap,
    /// Controls how lines containing wrapped children should be aligned within the space of the parent.
    ///
    /// Line alignment is calculated after child nodes compute their target sizes, but before stretch factors are
    /// applied.
    ///
    /// Has no effect if [`Self::flex_wrap`] is set to [`FlexWrap::NoWrap`].
    ///
    /// Mirrors [`Style::align_content`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// Controls how children should be aligned on the main axis.
    ///
    /// Does nothing in a wrapped line if:
    /// - Any child in the line has a [`SelfFlex::margin`] with [`Val::Auto`] set for a side on the main axis, or
    ///   has [`SelfFlex::flex_grow`] greater than `0.`.
    ///
    /// Mirrors [`Style::justify_content`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// Controls how children should be aligned on the cross axis.
    ///
    /// Child cross-alignment is calculated after line alignment ([`Self::justify_lines`]), since line alignment
    /// affects how wide the cross-axis of each line will be.
    ///
    /// Has no effect on a child if it has a [`SelfFlex::margin`] with [`Val::Auto`] set for a side on the cross
    /// axis.
    ///
    /// Mirrors [`Style::align_items`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// Controls the direction of text (left-to-right (e.g. English) or right-to-left (e.g. Arabic)).
    ///
    /// Defaults to [`Direction::Inherit`] with fallback to [`Direction::LeftToRight`].
    #[reflect(default)]
    pub text_direction: Direction,
    /// Gap applied between columns when organizing children.
    ///
    /// This is essentially a fixed gap inserted between children on the main axis, or lines on the cross axis.
    #[reflect(default)]
    pub column_gap: Val,
    /// Gap applied between rows when organizing children.
    ///
    /// This is essentially a fixed gap inserted between children on the main axis, or lines on the cross axis.
    #[reflect(default)]
    pub row_gap: Val,
}

impl ContentFlex
{
    /// Adds this struct's contents to [`Style`].
    pub fn set_in_style(&self, style: &mut Style)
    {
        style.overflow = self.clipping.into();
        style.padding = self.padding.into();
        style.flex_direction = self.flex_direction;
        style.align_content = self.justify_lines.into();
        style.justify_content = self.justify_main.into();
        style.align_items = self.justify_cross.into();
        style.direction = self.text_direction;
        style.column_gap = self.column_gap;
        style.row_gap = self.row_gap;
    }

    fn default_flex_wrap() -> FlexWrap
    {
        FlexWrap::NoWrap
    }
}

impl Default for ContentFlex
{
    fn default() -> Self
    {
        Self {
            flex_wrap: Self::default_flex_wrap(),

            clipping: Default::default(),
            padding: Default::default(),
            flex_direction: Default::default(),
            justify_lines: Default::default(),
            justify_main: Default::default(),
            justify_cross: Default::default(),
            text_direction: Default::default(),
            column_gap: Default::default(),
            row_gap: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls a node's flex behavior in its parent.
///
/// Mirrors fields in [`Style`].
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelfFlex
{
    /// Adds space outside the boundary of a node.
    ///
    /// If the main-axis values are set to [`Val::Auto`] then [`JustifyMain`] will do nothing, and similarly for
    /// the cross-axis with [`JustifyCross`].
    ///
    /// Defaults to zero margin.
    #[reflect(default)]
    pub margin: StyleRect,
    /// Overrides [`Dims::width`] or [`Dims::height`] along the parent's main axis.
    ///
    /// Defaults to [`Val::Auto`], which means 'fall back to width/height'.
    #[reflect(default)]
    pub flex_basis: Val,
    /// Controls automatic growing of a node up to its max size when its parent has excess space.
    ///
    /// When a line in the parent contains extra space on the main axis, it is distributed to each child
    /// proportional to `flex_grow / sum(flex_grow)`.
    ///
    /// Has no effect if the parent is using [`FlexWrap::Wrap`].
    ///
    /// Defaults to `0.`.
    #[reflect(default)]
    pub flex_grow: f32,
    /// Controls automatic shrinking of a node down to its minimum size when its parent doesn't have enough space.
    ///
    /// When a line in the parent overflows the main axis, shrinkage is distributed to each child proportional to
    /// `flex_shrink / sum(flex_shrink)`.
    /// If `sum(flex_shrink)` is zero then no nodes will shrink.
    /// If a child shrinks all the way to its minimum size, then its remaining shrink-share is distributed to
    /// other children with `flex_shrink`.
    ///
    /// Has no effect if the parent is using [`FlexWrap::Wrap`].
    ///
    /// Defaults to `1.`.
    #[reflect(default)]
    pub flex_shrink: f32,
    /// Controls how this node should be aligned on its parent's cross axis.
    ///
    /// If not set to [`JustifySelfCross::Auto`], then this overrides the parent's [`ContentFlex::justify_cross`]
    /// setting.
    ///
    /// Does nothing if the node's [`Self::margin`] has [`Val::Auto`] set on either of its cross-axis sides.
    ///
    /// Mirrors [`Style::align_self`].
    ///
    /// Defaults to [`JustifySelfCross::Auto`].
    #[reflect(default)]
    pub justify_self_cross: JustifySelfCross,
}

impl SelfFlex
{
    /// Adds this struct's contents to [`Style`].
    pub fn set_in_style(&self, style: &mut Style)
    {
        style.margin = self.margin.into();
        style.flex_basis = self.flex_basis;
        style.flex_grow = self.flex_grow;
        style.flex_shrink = self.flex_shrink;
        style.align_self = self.justify_self_cross.into();
    }
}

impl Default for SelfFlex
{
    fn default() -> Self
    {
        Self {
            margin: Default::default(),
            flex_basis: Default::default(),
            flex_grow: Default::default(),
            flex_shrink: 1.,
            justify_self_cross: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// UI style for absolute-positioned nodes.
///
/// Represents a [`Style`] with [`Display::Flex`] and [`PositionType::Absolute`].
/// Note that if you want an absolute node's position to be controlled by its parent's [`ContentFlex`], then set
/// the node's [`Dims::top`]/[`Dims::bottom`]/[`Dims::left`]/[`Dims::right`] fields to [`Val::Auto`].
///
/// See [`FlexStyle`] for flexbox-controlled nodes.
#[derive(ReactComponent, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsoluteStyle
{
    #[reflect(default)]
    pub dims: Dims,
    #[reflect(default)]
    pub content: ContentFlex,
}

impl Into<Style> for AbsoluteStyle
{
    fn into(self) -> Style
    {
        let mut style = Style::default();
        style.display = Display::Flex;
        style.position_type = PositionType::Absolute;
        self.dims.set_in_style(&mut style);
        self.content.set_in_style(&mut style);
        style
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// UI style for flexbox-controlled nodes.
///
/// Represents a [`Style`] with [`Display::Flex`] and [`PositionType::Relative`].
///
/// See [`AbsoluteStyle`] for absolute-positioned nodes.
#[derive(ReactComponent, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlexStyle
{
    #[reflect(default)]
    pub dims: Dims,
    #[reflect(default)]
    pub content: ContentFlex,
    #[reflect(default)]
    pub flex: SelfFlex,
}

impl Into<Style> for FlexStyle
{
    fn into(self) -> Style
    {
        let mut style = Style::default();
        style.display = Display::Flex;
        style.position_type = PositionType::Relative;
        self.dims.set_in_style(&mut style);
        self.content.set_in_style(&mut style);
        self.flex.set_in_style(&mut style);
        style
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive component that toggles the [`Style::display`] field.
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisplayControl
{
    /// Corresponds to [`Display::Flex`].
    #[default]
    Display,
    /// Corresponds to [`Display::None`].
    Hide,
}

impl Into<Display> for DisplayControl
{
    fn into(self) -> Display
    {
        match self {
            Self::Display => Display::Flex,
            Self::Hide => Display::None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_absolute_style(
    mut commands: Commands,
    insertion: InsertionEvent<AbsoluteStyle>,
    mutation: MutationEvent<AbsoluteStyle>,
    node: Query<(&React<AbsoluteStyle>, Option<&React<DisplayControl>>)>,
)
{
    let entity = insertion.get().unwrap_or_else(|| mutation.entity());
    let Ok((style, maybe_display_control)) = node.get(entity) else { return };
    let mut style: Style = (*style).clone().into();
    if let Some(control) = maybe_display_control {
        style.display = (**control).into();
    }
    commands.entity(entity).try_insert(style.clone());
}

struct DetectAbsoluteStyle;
impl WorldReactor for DetectAbsoluteStyle
{
    type StartingTriggers = (InsertionTrigger<AbsoluteStyle>, MutationTrigger<AbsoluteStyle>);
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback
    {
        SystemCommandCallback::new(detect_absolute_style)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_flex_style(
    mut commands: Commands,
    insertion: InsertionEvent<FlexStyle>,
    mutation: MutationEvent<FlexStyle>,
    node: Query<(&React<FlexStyle>, Option<&React<DisplayControl>>)>,
)
{
    let entity = insertion.get().unwrap_or_else(|| mutation.entity());
    let Ok((style, maybe_display_control)) = node.get(entity) else { return };
    let mut style: Style = (*style).clone().into();
    if let Some(control) = maybe_display_control {
        style.display = (**control).into();
    }
    commands.entity(entity).try_insert(style.clone());
}

struct DetectFlexStyle;
impl WorldReactor for DetectFlexStyle
{
    type StartingTriggers = (InsertionTrigger<FlexStyle>, MutationTrigger<FlexStyle>);
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback
    {
        SystemCommandCallback::new(detect_flex_style)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_display_control(
    insertion: InsertionEvent<DisplayControl>,
    mutation: MutationEvent<DisplayControl>,
    mut node: Query<(&mut Style, &React<DisplayControl>)>,
)
{
    let entity = insertion.get().unwrap_or_else(|| mutation.entity());
    let Ok((mut style, control)) = node.get_mut(entity) else { return };
    style.display = (**control).into();
}

struct DetectDisplayControl;
impl WorldReactor for DetectDisplayControl
{
    type StartingTriggers = (InsertionTrigger<DisplayControl>, MutationTrigger<DisplayControl>);
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback
    {
        SystemCommandCallback::new(detect_display_control)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct StyleWrappersPlugin;

impl Plugin for StyleWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<StyleRect>()
            .register_type::<Clipping>()
            .register_type::<JustifyLines>()
            .register_type::<JustifyMain>()
            .register_type::<JustifyCross>()
            .register_type::<JustifySelfCross>()
            .register_type::<Dims>()
            .register_type::<ContentFlex>()
            .register_type::<SelfFlex>()
            .register_type::<AbsoluteStyle>()
            .register_type::<FlexStyle>()
            .register_type::<DisplayControl>()
            .add_world_reactor_with(
                DetectAbsoluteStyle,
                (insertion::<AbsoluteStyle>(), mutation::<AbsoluteStyle>()),
            )
            .add_world_reactor_with(DetectFlexStyle, (insertion::<FlexStyle>(), mutation::<FlexStyle>()))
            .add_world_reactor_with(
                DetectDisplayControl,
                (insertion::<DisplayControl>(), mutation::<DisplayControl>()),
            )
            .register_reactive_loadable::<AbsoluteStyle>()
            .register_reactive_loadable::<FlexStyle>()
            .register_reactive_loadable::<DisplayControl>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

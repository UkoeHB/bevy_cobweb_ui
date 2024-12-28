use bevy::prelude::*;
use bevy::ui::UiSystem;

use crate::prelude::*;
use crate::sickle::Lerp;

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiRect`] for stylesheet serialization.
///
/// All fields default to `Val::Px(0.)`.
#[derive(Reflect, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleRect {
    #[reflect(default = "StyleRect::default_field")]
    pub top: Val,
    #[reflect(default = "StyleRect::default_field")]
    pub bottom: Val,
    #[reflect(default = "StyleRect::default_field")]
    pub left: Val,
    #[reflect(default = "StyleRect::default_field")]
    pub right: Val,
}

impl StyleRect {
    fn default_field() -> Val {
        Val::Px(0.)
    }

    /// Constructs a style rect with all sides equal to `single`.
    pub fn splat(single: Val) -> Self {
        Self { top: single, bottom: single, left: single, right: single }
    }
}

impl Into<UiRect> for StyleRect {
    fn into(self) -> UiRect {
        UiRect {
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
        }
    }
}

impl From<UiRect> for StyleRect {
    fn from(rect: UiRect) -> Self {
        Self {
            left: rect.left,
            right: rect.right,
            top: rect.top,
            bottom: rect.bottom,
        }
    }
}

impl Default for StyleRect {
    fn default() -> Self {
        Self {
            top: Self::default_field(),
            bottom: Self::default_field(),
            left: Self::default_field(),
            right: Self::default_field(),
        }
    }
}

impl Lerp for StyleRect {
    fn lerp(&self, to: Self, t: f32) -> Self {
        Self {
            left: self.left.lerp(to.left, t),
            right: self.right.lerp(to.right, t),
            top: self.top.lerp(to.top, t),
            bottom: self.bottom.lerp(to.bottom, t),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`Overflow`] for the [`FlexNode`] and [`AbsoluteNode`] loadables.
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum Clipping {
    #[default]
    None,
    ClipX,
    ClipY,
    ClipXY,
    ScrollX,
    ScrollY,
    /// Recommended for horizontal scrolling.
    ScrollXClipY,
    /// Recommended for vertical scrolling.
    ScrollYClipX,
    /// For bi-directional scrolling.
    ScrollXY,
}

impl Into<Overflow> for Clipping {
    fn into(self) -> Overflow {
        match self {
            Self::None => Overflow { x: OverflowAxis::Visible, y: OverflowAxis::Visible },
            Self::ClipX => Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Visible },
            Self::ClipY => Overflow { x: OverflowAxis::Visible, y: OverflowAxis::Clip },
            Self::ClipXY => Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Clip },
            Self::ScrollX => Overflow { x: OverflowAxis::Scroll, y: OverflowAxis::Visible },
            Self::ScrollY => Overflow { x: OverflowAxis::Visible, y: OverflowAxis::Scroll },
            Self::ScrollXClipY => Overflow { x: OverflowAxis::Scroll, y: OverflowAxis::Clip },
            Self::ScrollYClipX => Overflow { x: OverflowAxis::Clip, y: OverflowAxis::Scroll },
            Self::ScrollXY => Overflow { x: OverflowAxis::Scroll, y: OverflowAxis::Scroll },
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
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum JustifyLines {
    /// Pack lines toward the start of the cross axis.
    ///
    /// Affected by `text_direction` (unimplemented) for [`FlexDirection::Column`].
    #[default]
    FlexStart,
    /// Pack lines toward the end of the cross axis.
    ///
    /// Affected by `text_direction` (unimplemented) for [`FlexDirection::Column`].
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

impl Into<AlignContent> for JustifyLines {
    fn into(self) -> AlignContent {
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
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum JustifyMain {
    /*
    /// Cluster items at the start of the main axis.
    /// - [`FlexDirection::Row`]: Start according to `text_direction` (unimplemented).
    /// - [`FlexDirection::ReverseRow`]: Start according to `text_direction` (unimplemented). ** difference
    /// - [`FlexDirection::Column`]: Top.
    /// - [`FlexDirection::ColumnReverse`]: Bottom.
    Start,
    /// Cluster items at the end of the main axis.
    /// - [`FlexDirection::Row`]: End according to `text_direction` (unimplemented).
    /// - [`FlexDirection::ReverseRow`]: End according to `text_direction` (unimplemented). ** difference
    /// - [`FlexDirection::Column`]: Bottom.
    /// - [`FlexDirection::ColumnReverse`]: Top.
    End,
    */
    /// Cluster items at the start of the main axis.
    /// - `FlexDirection::Row`: Start according to `text_direction` (unimplemented).
    /// - `FlexDirection::RowReverse`: End according to `text_direction` (unimplemented).
    /// - `FlexDirection::Column`: Top.
    /// - `FlexDirection::ColumnReverse`: Bottom.
    #[default]
    FlexStart,
    /// Cluster items at the end of the main axis.
    /// - `FlexDirection::Row`: End according to `text_direction` (unimplemented).
    /// - `FlexDirection::RowReverse`: Start according to `text_direction` (unimplemented).
    /// - `FlexDirection::Column`: Bottom.
    /// - `FlexDirection::ColumnReverse`: Top.
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

impl Into<JustifyContent> for JustifyMain {
    fn into(self) -> JustifyContent {
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
/// Can be overwridden on individual items with [`JustifySelfCross`].
///
/// Mirrors [`AlignItems`].
/// Excludes [`AlignItems::Baseline`] which is too confusing to use easily.
/// Excludes [`AlignItems::Default`] which is usually [`Self::Stretch`] but sometimes [`Self::FlexStart`].
/// Excludes [`AlignItems::Start`] and [`AlignItems::End`] which are equivalent to the `FlexStart`/`FlexEnd`
/// variants (except when [`FlexWrap::WrapReverse`] is used, but don't use that).
///
/// Defaults to [`Self::FlexStart`].
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum JustifyCross {
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
    /// If using [`AbsoluteNode`] and [`Dims::top`]/[`Dims::bottom`]/[`Dims::left`]/[`Dims::right`] are set to
    /// all auto, then this falls back to [`JustifyCross::FlexStart`].
    Stretch,
}

impl Into<AlignItems> for JustifyCross {
    fn into(self) -> AlignItems {
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
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum JustifySelfCross {
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

impl Into<AlignSelf> for JustifySelfCross {
    fn into(self) -> AlignSelf {
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
/// Mirrors fields in [`Node`].
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dims {
    /// Indicates the `desired` width of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'content-sized'.
    ///
    /// If set to non-[`Val::Auto`], then the desired width will be overriden if:
    /// - [`FlexNode`]: If [`FlexDirection::Row`]/[`FlexDirection::RowReverse`] is set and
    ///   [`SelfFlex::flex_basis`] is set to non-auto.
    ///
    /// If set to [`Val::Auto`], then the desired width will be overriden if:
    /// - [`AbsoluteNode`]: [`Dims::left`] and [`Dims::right`] are set.
    /// - [`FlexNode`]: Parent is using [`FlexDirection::Column`]/[`FlexDirection::ColumnReverse`] and
    ///   [`JustifyCross::Stretch`]. Or, if parent is using [`FlexDirection::Row`]/[`FlexDirection::RowReverse`]
    ///   and self is using [`SelfFlex::flex_grow`]/[`SelfFlex::flex_shrink`].
    #[reflect(default)]
    pub width: Val,
    /// Indicates the `desired` height of the node.
    ///
    /// Defaults to [`Val::Auto`], which means 'content-sized'.
    ///
    /// If set to non-[`Val::Auto`], then the desired height will be overriden if:
    /// - [`FlexNode`]: If [`FlexDirection::Column`]/[`FlexDirection::ColumnReverse`] is set and
    ///   [`SelfFlex::flex_basis`] is set to non-auto.
    ///
    /// If set to [`Val::Auto`], then the desired height will be overriden if:
    /// - [`AbsoluteNode`]: [`Dims::top`] and [`Dims::bottom`] are set.
    /// - [`FlexNode`]: Parent is using [`FlexDirection::Row`]/[`FlexDirection::RowReverse`] and
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
    /// - [`AbsoluteNode`]: If `width`/`height` are set to auto and all offset fields are set, then the offset's
    ///   `bottom` parameter will be ignored and the aspect ratio will use the `left`/`right` controlled width.
    /// - [`FlexNode`]: [`SelfFlex::flex_basis`] can override the width/height, which affects whether they are
    ///   considered 'auto'.
    #[reflect(default)]
    pub aspect_ratio: Option<f32>,
    /// Region between a node's boundary and its padding.
    ///
    /// All border sizes with [`Val::Percent`] are computed with respect to the *width* of the node.
    ///
    /// See [`BorderColor`] for a typical use-case.
    ///
    /// Defaults to zero border.
    #[reflect(default)]
    pub border: StyleRect,
    /// Position offsets applied to the edges of a node's margin.
    /// - [`AbsoluteNode`] (absolute): Relative to its parent's boundary (ignoring padding). Can be used to resize
    ///   the node if `width`/`height` is set to auto and both `left`/`right` or `top`/`bottom` are set.
    /// - [`FlexNode`] (relative): Relative to the final position of the edges of its margin after layout is
    ///   calculated. Does not affect the layout of other nodes. Cannot be used to resize the node (see note
    ///   following).
    ///
    /// If both `left` and `right` are set, then `right` will be overridden by the `width` field unless it is
    /// [`Val::Auto`]. The same goes for `top`/`bottom`/`height`. In practice this means [`FlexNode`] cannot
    /// use both `left`/`right` or `top`/`bottom` parameters since if `width`/`height` are auto then the
    /// layout algorithm will control the node size.
    ///
    /// Defaults to `StyleRect{ left: Val::Pixels(0.), top: Val::Pixels(0.), right: Val::Auto, left: Val::Auto }`.
    /// This ensures [`AbsoluteNode`] nodes will start in the upper left corner of their parents. If the
    /// offset is set to all [`Val::Auto`] then the node's position will be controlled by its parent's
    /// [`ContentFlex`] parameters. You must set the `left`/`top` fields to auto if using [`FlexNode`] and
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

impl Dims {
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(&self, node: &mut Node) {
        node.width = self.width;
        node.height = self.height;
        node.max_width = self.max_width;
        node.max_height = self.max_height;
        node.min_width = self.min_width;
        node.min_height = self.min_height;
        node.aspect_ratio = self.aspect_ratio;
        node.border = self.border.into();
        node.left = self.left;
        node.right = self.right;
        node.top = self.top;
        node.bottom = self.bottom;
    }

    fn default_top() -> Val {
        Val::Px(0.)
    }
    fn default_left() -> Val {
        Val::Px(0.)
    }
}

impl Default for Dims {
    fn default() -> Self {
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
/// Mirrors fields in [`Node`].
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContentFlex {
    /// Determines whether the node contents will be clipped at the node boundary.
    ///
    /// Can be used to make a node scrollable.
    ///
    /// Defaults to no clipping.
    #[reflect(default)]
    pub clipping: Clipping,
    /// Controls the boundaries of [`Self::clipping`]. See [`OverflowClipMargin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// Inserts space between the node's [`Dims::border`] and its contents.
    ///
    /// All padding sizes with [`Val::Percent`] are computed with respect to the *width* of the node.
    ///
    /// Defaults to zero padding.
    #[reflect(default)]
    pub padding: StyleRect,
    /// Controls which direction the main flex axis points within this node.
    ///
    /// - [`FlexDirection::Row`]: left to right, flex wrapped lines are added down
    /// - [`FlexDirection::Column`]: top-to-bottom, flex wrapped lines are added left to right
    /// - [`FlexDirection::RowReverse`]: right to left, flex wrapped lines are added down
    /// - [`FlexDirection::ColumnReverse`]: bottom-to-top, flex wrapped lines are added in right to left
    ///
    /// Note: if `text_direction` gets implemented, then it will affect how flex wrapped lines are added.
    #[reflect(default)]
    pub flex_direction: FlexDirection,
    /// Controls whether children should wrap to multiple lines when overflowing the main axis.
    ///
    /// If children wrap, then wrapping lines can potentially overflow the cross axis.
    ///
    /// It is not recommended to use [`FlexWrap::WrapReverse`] unless you are prepared for the added complexity of
    /// figuring out how
    /// [`JustifyMain`]/[`JustifyCross`]/[`JustifyLines`]/`text_direction` (unimplemented)/[`FlexDirection`]
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
    /// Mirrors [`Node::align_content`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// Controls how children should be aligned on the main axis.
    ///
    /// Does nothing in a wrapped line if:
    /// - Any child in the line has a [`SelfFlex::margin`] with [`Val::Auto`] set for a side on the main axis, or
    ///   has [`SelfFlex::flex_grow`] greater than `0.`.
    ///
    /// Mirrors [`Node::justify_content`].
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
    /// Mirrors [`Node::align_items`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
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

impl ContentFlex {
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(&self, node: &mut Node) {
        node.overflow = self.clipping.into();
        node.overflow_clip_margin = self.clip_margin;
        node.padding = self.padding.into();
        node.flex_direction = self.flex_direction;
        node.flex_wrap = self.flex_wrap;
        node.align_content = self.justify_lines.into();
        node.justify_content = self.justify_main.into();
        node.align_items = self.justify_cross.into();
        node.column_gap = self.column_gap;
        node.row_gap = self.row_gap;
    }

    fn default_flex_wrap() -> FlexWrap {
        FlexWrap::NoWrap
    }
}

impl Default for ContentFlex {
    fn default() -> Self {
        Self {
            flex_wrap: Self::default_flex_wrap(),

            clipping: Default::default(),
            clip_margin: Default::default(),
            padding: Default::default(),
            flex_direction: Default::default(),
            justify_lines: Default::default(),
            justify_main: Default::default(),
            justify_cross: Default::default(),
            column_gap: Default::default(),
            row_gap: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls a node's flex behavior in its parent.
///
/// Mirrors fields in [`Node`].
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfFlex {
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
    /// Mirrors [`Node::align_self`].
    ///
    /// Defaults to [`JustifySelfCross::Auto`].
    #[reflect(default)]
    pub justify_self_cross: JustifySelfCross,
}

impl SelfFlex {
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(&self, node: &mut Node) {
        node.margin = self.margin.into();
        node.flex_basis = self.flex_basis;
        node.flex_grow = self.flex_grow;
        node.flex_shrink = self.flex_shrink;
        node.align_self = self.justify_self_cross.into();
    }
}

impl Default for SelfFlex {
    fn default() -> Self {
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

/// Instruction loadable for absolute-positioned nodes.
///
/// Inserts a [`Node`] with [`Display::Flex`] and [`PositionType::Absolute`].
/// Note that if you want an absolute node's position to be controlled by its parent's [`ContentFlex`], then set
/// the node's [`Dims::top`]/[`Dims::bottom`]/[`Dims::left`]/[`Dims::right`] fields to [`Val::Auto`].
///
/// See [`FlexNode`] for flexbox-controlled nodes. See [`DisplayControl`] for setting [`Display::None`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbsoluteNode {
    // TODO: re-enable once #[reflect(flatten)] is available
    // #[reflect(default)]
    // pub dims: Dims,
    // #[reflect(default)]
    // pub content: ContentFlex,

    // DIMS
    /// See [`Dims::width`].
    #[reflect(default)]
    pub width: Val,
    /// See [`Dims::height`].
    #[reflect(default)]
    pub height: Val,
    /// See [`Dims::max_width`].
    #[reflect(default)]
    pub max_width: Val,
    /// See [`Dims::max_height`].
    #[reflect(default)]
    pub max_height: Val,
    /// See [`Dims::min_width`].
    #[reflect(default)]
    pub min_width: Val,
    /// See [`Dims::min_height`].
    #[reflect(default)]
    pub min_height: Val,
    /// See [`Dims::aspect_ratio`].
    #[reflect(default)]
    pub aspect_ratio: Option<f32>,
    /// See [`Dims::border`].
    #[reflect(default)]
    pub border: StyleRect,
    /// See [`Dims::top`].
    #[reflect(default = "Dims::default_top")]
    pub top: Val,
    /// See [`Dims::bottom`].
    #[reflect(default)]
    pub bottom: Val,
    /// See [`Dims::left`].
    #[reflect(default = "Dims::default_left")]
    pub left: Val,
    /// See [`Dims::right`].
    #[reflect(default)]
    pub right: Val,

    // CONTENT
    /// See [`ContentFlex::clipping`].
    #[reflect(default)]
    pub clipping: Clipping,
    /// See [`ContentFlex::clip_margin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// See [`ContentFlex::padding`].
    #[reflect(default)]
    pub padding: StyleRect,
    /// See [`ContentFlex::flex_direction`].
    #[reflect(default)]
    pub flex_direction: FlexDirection,
    /// See [`ContentFlex::flex_wrap`].
    #[reflect(default = "ContentFlex::default_flex_wrap")]
    pub flex_wrap: FlexWrap,
    /// See [`ContentFlex::justify_lines`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// See [`ContentFlex::justify_main`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// See [`ContentFlex::justify_cross`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// See [`ContentFlex::column_gap`].
    #[reflect(default)]
    pub column_gap: Val,
    /// See [`ContentFlex::row_gap`].
    #[reflect(default)]
    pub row_gap: Val,
}

impl Into<Node> for AbsoluteNode {
    fn into(self) -> Node {
        let mut node = Node::default();
        node.display = Display::Flex;
        node.position_type = PositionType::Absolute;
        Dims {
            width: self.width,
            height: self.height,
            max_width: self.max_width,
            max_height: self.max_height,
            min_width: self.min_width,
            min_height: self.min_height,
            aspect_ratio: self.aspect_ratio,
            border: self.border,
            top: self.top,
            bottom: self.bottom,
            left: self.left,
            right: self.right,
        }
        .set_in_node(&mut node);
        ContentFlex {
            clipping: self.clipping,
            clip_margin: self.clip_margin,
            padding: self.padding,
            flex_direction: self.flex_direction,
            flex_wrap: self.flex_wrap,
            justify_lines: self.justify_lines,
            justify_main: self.justify_main,
            justify_cross: self.justify_cross,
            column_gap: self.column_gap,
            row_gap: self.row_gap,
        }
        .set_in_node(&mut node);
        node
    }
}

impl Instruction for AbsoluteNode {
    fn apply(self, entity: Entity, world: &mut World) {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        let display = emut.get::<DisplayControl>().copied().unwrap_or_default();
        let mut node: Node = self.into();
        node.display = display.into();

        emut.insert(node);
    }

    fn revert(entity: Entity, world: &mut World) {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove_with_requires::<Node>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for flexbox-controlled nodes.
///
/// Inserts a [`Node`] with [`Display::Flex`] and [`PositionType::Relative`].
///
/// See [`AbsoluteNode`] for absolute-positioned nodes. See [`DisplayControl`] for setting [`Display::None`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlexNode {
    // TODO: re-enable once #[reflect(flatten)] is available
    // #[reflect(default)]
    // pub dims: Dims,
    // #[reflect(default)]
    // pub content: ContentFlex,
    // #[reflect(default)]
    // pub flex: SelfFlex,

    // DIMS
    /// See [`Dims::width`].
    #[reflect(default)]
    pub width: Val,
    /// See [`Dims::height`].
    #[reflect(default)]
    pub height: Val,
    /// See [`Dims::max_width`].
    #[reflect(default)]
    pub max_width: Val,
    /// See [`Dims::max_height`].
    #[reflect(default)]
    pub max_height: Val,
    /// See [`Dims::min_width`].
    #[reflect(default)]
    pub min_width: Val,
    /// See [`Dims::min_height`].
    #[reflect(default)]
    pub min_height: Val,
    /// See [`Dims::aspect_ratio`].
    #[reflect(default)]
    pub aspect_ratio: Option<f32>,
    /// See [`Dims::border`].
    #[reflect(default)]
    pub border: StyleRect,
    /// See [`Dims::top`].
    #[reflect(default = "Dims::default_top")]
    pub top: Val,
    /// See [`Dims::bottom`].
    #[reflect(default)]
    pub bottom: Val,
    /// See [`Dims::left`].
    #[reflect(default = "Dims::default_left")]
    pub left: Val,
    /// See [`Dims::right`].
    #[reflect(default)]
    pub right: Val,

    // CONTENT
    /// See [`ContentFlex::clipping`].
    #[reflect(default)]
    pub clipping: Clipping,
    /// See [`ContentFlex::clip_margin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// See [`ContentFlex::padding`].
    #[reflect(default)]
    pub padding: StyleRect,
    /// See [`ContentFlex::flex_direction`].
    #[reflect(default)]
    pub flex_direction: FlexDirection,
    /// See [`ContentFlex::flex_wrap`].
    #[reflect(default = "ContentFlex::default_flex_wrap")]
    pub flex_wrap: FlexWrap,
    /// See [`ContentFlex::justify_lines`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// See [`ContentFlex::justify_main`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// See [`ContentFlex::justify_cross`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// See [`ContentFlex::column_gap`].
    #[reflect(default)]
    pub column_gap: Val,
    /// See [`ContentFlex::row_gap`].
    #[reflect(default)]
    pub row_gap: Val,

    // SELF FLEX
    /// See [`SelfFlex::margin`].
    #[reflect(default)]
    pub margin: StyleRect,
    /// See [`SelfFlex::flex_basis`].
    #[reflect(default)]
    pub flex_basis: Val,
    /// See [`SelfFlex::flex_grow`].
    #[reflect(default)]
    pub flex_grow: f32,
    /// See [`SelfFlex::flex_shrink`].
    #[reflect(default)]
    pub flex_shrink: f32,
    /// See [`SelfFlex::justify_self_cross`].
    #[reflect(default)]
    pub justify_self_cross: JustifySelfCross,
}

impl Into<Node> for FlexNode {
    fn into(self) -> Node {
        let mut node = Node::default();
        node.display = Display::Flex;
        node.position_type = PositionType::Relative;
        Dims {
            width: self.width,
            height: self.height,
            max_width: self.max_width,
            max_height: self.max_height,
            min_width: self.min_width,
            min_height: self.min_height,
            aspect_ratio: self.aspect_ratio,
            border: self.border,
            top: self.top,
            bottom: self.bottom,
            left: self.left,
            right: self.right,
        }
        .set_in_node(&mut node);
        ContentFlex {
            clipping: self.clipping,
            clip_margin: self.clip_margin,
            padding: self.padding,
            flex_direction: self.flex_direction,
            flex_wrap: self.flex_wrap,
            justify_lines: self.justify_lines,
            justify_main: self.justify_main,
            justify_cross: self.justify_cross,
            column_gap: self.column_gap,
            row_gap: self.row_gap,
        }
        .set_in_node(&mut node);
        SelfFlex {
            margin: self.margin,
            flex_basis: self.flex_basis,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            justify_self_cross: self.justify_self_cross,
        }
        .set_in_node(&mut node);
        node
    }
}

impl Instruction for FlexNode {
    fn apply(self, entity: Entity, world: &mut World) {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        let display = emut.get::<DisplayControl>().copied().unwrap_or_default();
        let mut node: Node = self.into();
        node.display = display.into();

        emut.insert(node);
    }

    fn revert(entity: Entity, world: &mut World) {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove_with_requires::<Node>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for Grid-controlled nodes.
///
/// Inserts a [`Node`] with [`Display::Grid`] and [`PositionType::Relative`].
///
/// **NOTE** This does not support all Gridding features in bevy
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridNode {
    // DIMS
    /// See [`Dims::width`].
    #[reflect(default)]
    pub width: Val,
    /// See [`Dims::height`].
    #[reflect(default)]
    pub height: Val,
    /// See [`Dims::max_width`].
    #[reflect(default)]
    pub max_width: Val,
    /// See [`Dims::max_height`].
    #[reflect(default)]
    pub max_height: Val,
    /// See [`Dims::min_width`].
    #[reflect(default)]
    pub min_width: Val,
    /// See [`Dims::min_height`].
    #[reflect(default)]
    pub min_height: Val,
    /// See [`Dims::aspect_ratio`].
    #[reflect(default)]
    pub aspect_ratio: Option<f32>,
    /// See [`Dims::border`].
    #[reflect(default)]
    pub border: StyleRect,
    /// See [`Dims::top`].
    #[reflect(default = "Dims::default_top")]
    pub top: Val,
    /// See [`Dims::bottom`].
    #[reflect(default)]
    pub bottom: Val,
    /// See [`Dims::left`].
    #[reflect(default = "Dims::default_left")]
    pub left: Val,
    /// See [`Dims::right`].
    #[reflect(default)]
    pub right: Val,

    /// Grid layout
    /// 
    /// Put values inside `[]` brackets.
    /// 
    /// **Note**: Only [`GridVal`] variants are supported, which only has a subset of the `GridTrack` API.
    #[reflect(default)]
    pub grid_template_rows: Vec<GridVal>,

    /// Grid layout
    /// 
    /// Put values inside `[]` brackets.
    /// 
    /// **Note**: Only [`GridVal`] variants are supported, which only has a subset of the `GridTrack` API.
    #[reflect(default)]
    pub grid_template_columns: Vec<GridVal>,

    // CONTENT
    /// See [`ContentFlex::clipping`].
    #[reflect(default)]
    pub clipping: Clipping,
    /// See [`ContentFlex::clip_margin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// See [`ContentFlex::padding`].
    #[reflect(default)]
    pub padding: StyleRect,
    /// See [`ContentFlex::flex_direction`].
    #[reflect(default)]
    pub flex_direction: FlexDirection,
    /// See [`ContentFlex::flex_wrap`].
    #[reflect(default = "ContentFlex::default_flex_wrap")]
    pub flex_wrap: FlexWrap,
    /// See [`ContentFlex::justify_lines`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// See [`ContentFlex::justify_main`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// See [`ContentFlex::justify_cross`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// See [`ContentFlex::column_gap`].
    #[reflect(default)]
    pub column_gap: Val,
    /// See [`ContentFlex::row_gap`].
    #[reflect(default)]
    pub row_gap: Val,
}

impl Instruction for GridNode {
    fn apply(self, entity: Entity, world: &mut World) {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // let display = emut.get::<DisplayControl>().copied().unwrap_or_default();
        let mut node: Node = self.into();
        node.display = Display::Grid;

        emut.insert(node);
    }

    fn revert(entity: Entity, world: &mut World) {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove_with_requires::<Node>();
        });
    }
}

impl Into<Node> for GridNode {
    fn into(self) -> Node {
        let mut node = Node::default();
        node.display = Display::Grid;
        node.position_type = PositionType::Relative;

        Dims {
            width: self.width,
            height: self.height,
            max_width: self.max_width,
            max_height: self.max_height,
            min_width: self.min_width,
            min_height: self.min_height,
            aspect_ratio: self.aspect_ratio,
            border: self.border,
            top: self.top,
            bottom: self.bottom,
            left: self.left,
            right: self.right,
        }
        .set_in_node(&mut node);

        ContentFlex {
            clipping: self.clipping,
            clip_margin: self.clip_margin,
            padding: self.padding,
            flex_direction: self.flex_direction,
            flex_wrap: self.flex_wrap,
            justify_lines: self.justify_lines,
            justify_main: self.justify_main,
            justify_cross: self.justify_cross,
            column_gap: self.column_gap,
            row_gap: self.row_gap,
        }
        .set_in_node(&mut node);

        node.grid_template_rows = self
            .grid_template_rows
            .iter()
            .map(|x| GridTrack::from(*x).into())
            .collect();

        node.grid_template_columns = self
            .grid_template_columns
            .iter()
            .map(|x| GridTrack::from(*x).into())
            .collect();
        node
    }
} //-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable that toggles the [`Node::display`] field.
///
/// Inserts self as a component so the `AbsoluteNode` and `FlexNode` loadables can read the correct display value
/// when they are applied.
#[derive(Component, Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum DisplayControl {
    /// Corresponds to [`Display::Flex`].
    #[default]
    Flex,
    ///There for grid based displays
    Grid,
    /// Corresponds to [`Display::None`].
    Hide,
}

impl DisplayControl {
    fn refresh(mut nodes: Query<(&mut Node, &DisplayControl), Or<(Changed<Node>, Changed<DisplayControl>)>>) {
        for (mut node, control) in nodes.iter_mut() {
            if node.display != (*control).into() {
                node.display = (*control).into();
            }
        }
    }
}

impl Into<Display> for DisplayControl {
    fn into(self) -> Display {
        match self {
            Self::Flex => Display::Flex,
            Self::Hide => Display::None,
            Self::Grid => Display::Grid,
        }
    }
}

impl Instruction for DisplayControl {
    fn apply(self, entity: Entity, world: &mut World) {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.insert(self);
    }

    fn revert(entity: Entity, world: &mut World) {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
            if let Some(mut node) = e.get_mut::<Node>() {
                node.display = Self::Flex.into();
            }
        });
    }
}

impl StaticAttribute for DisplayControl {
    type Value = Self;
    fn construct(value: Self::Value) -> Self {
        value
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Like [`Val`] but also accepts additional values for grid layouts.
///
/// Can be converted to a [`GridTrack`] for use in [`Node`].
#[derive(Clone, Copy, Default, Debug, Reflect, PartialEq)]
#[reflect(Default, PartialEq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum GridVal {
    #[default]
    Auto,
    Percent(f32),
    Px(f32),
    Vw(f32),
    Vh(f32),
    VMin(f32),
    VMax(f32),
    Fr(f32),
}

impl From<GridVal> for GridTrack {
    fn from(value: GridVal) -> Self {
        match value {
            GridVal::Auto => GridTrack::auto(),
            GridVal::Percent(f) => GridTrack::percent(f),
            GridVal::Px(f) => GridTrack::px(f),
            GridVal::Vw(f) => GridTrack::vw(f),
            GridVal::Vh(f) => GridTrack::vh(f),
            GridVal::VMin(f) => GridTrack::vmin(f),
            GridVal::VMax(f) => GridTrack::vmax(f),
            GridVal::Fr(f) => GridTrack::fr(f),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct StyleWrappersPlugin;

impl Plugin for StyleWrappersPlugin {
    fn build(&self, app: &mut App) {
        app.register_instruction_type::<AbsoluteNode>()
            .register_instruction_type::<FlexNode>()
            .register_instruction_type::<GridNode>()
            .register_static::<DisplayControl>()
            .add_systems(PostUpdate, DisplayControl::refresh.before(UiSystem::Prepare));
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;
use bevy::ui::UiSystem;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::sickle::Lerp;

//-------------------------------------------------------------------------------------------------------------------

/// Helper component for caching a `Node::display` value when display is hidden. This way when display is
/// shown again, the correct layout algorithm can be set.
///
/// The `From<Display>` impl converts `Display::None` and `Display::Block` to `Display::Flex`.
#[derive(Component, Default, Copy, Clone, Debug, PartialEq)]
enum DisplayType
{
    #[default]
    Flex,
    Grid,
}

impl From<Display> for DisplayType
{
    fn from(display: Display) -> Self
    {
        match display {
            Display::Flex => Self::Flex,
            Display::Grid => Self::Grid,
            Display::None | Display::Block => Self::Flex,
        }
    }
}

impl Into<Display> for DisplayType
{
    fn into(self) -> Display
    {
        match self {
            Self::Flex => Display::Flex,
            Self::Grid => Display::Grid,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Like [`Val`] but also accepts additional values for grid layouts.
///
/// Can be converted to a [`GridTrack`] for use in [`Node`].
///
/// See [`RepeatedGridVal`] also.
#[derive(Clone, Default, Debug, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq, Debug)]
#[reflect(no_field_bounds)]
#[cfg_attr(feature = "serde", reflect(Serialize, Deserialize))]
pub enum GridVal
{
    #[default]
    Auto,
    /// Includes a px limit.
    FitContentPx(f32),
    /// Includes a percentage limit.
    FitContentPercent(f32),
    /// Equivalent to `Self::MinMax(vec![Self::Px(0.0), Seof::Fraction(val)])`.
    Flex(f32),
    MinContent,
    MaxContent,
    Percent(f32),
    Px(f32),
    Vw(f32),
    Vh(f32),
    VMin(f32),
    VMax(f32),
    /// [Fraction size](https://www.w3.org/TR/css3-grid-layout/#fr-unit).
    ///
    /// Fraction of available grid space. Is divided by sum of fractions in the relevant grid dimension to get the
    /// 'real' fraction assigned to this node.
    ///
    /// Will set the minimum size to `Auto`, which makes it content-based. [`Self::Flex`] is recommended if you
    /// want a minimum size of zero.
    Fraction(f32),
    /// Must be a vector of size 2. If size one then the max function will be defaulted. If size zero both will be
    /// defaulted. If greater than size two, excess entries will be ignored. If an element fails to convert to a
    /// sizing function then it will fall back to `auto`.
    ///
    /// Element 1: maps to [`MinTrackSizingFunction`].
    /// - Allowed variants: `auto`, `px`, `%`, `vmin`, `vmax`, `vw`, `vh`, `MinContent`, `MaxContent`
    ///
    /// Element 2: maps to [`MaxTrackSizingFunction`].
    /// - Allowed variants: `auto`, `px`, `%`, `fr`, `vmin`, `vmax`, `vw`, `vh`, `FitContentPx`,
    ///   `FitContentPercent`,
    /// `MinContent`, `MaxContent`,
    MinMax(Vec<Self>),
    /// Used for [`RepeatedGridTrack::repeat_many`] by [`RepeatedGridVal`].
    ///
    /// `GridTrack::from(GridVal::Many(...))` will return the first grid value or default to [`GridTrack::auto`].
    Many(Vec<Self>),
}

impl From<GridVal> for GridTrack
{
    fn from(value: GridVal) -> Self
    {
        match value {
            GridVal::Auto => Self::auto(),
            GridVal::FitContentPx(v) => Self::fit_content_px(v),
            GridVal::FitContentPercent(v) => Self::fit_content_percent(v),
            GridVal::Flex(v) => Self::flex(v),
            GridVal::MinContent => Self::min_content(),
            GridVal::MaxContent => Self::max_content(),
            GridVal::Percent(v) => Self::percent(v),
            GridVal::Px(v) => Self::px(v),
            GridVal::Vw(v) => Self::vw(v),
            GridVal::Vh(v) => Self::vh(v),
            GridVal::VMin(v) => Self::vmin(v),
            GridVal::VMax(v) => Self::vmax(v),
            GridVal::Fraction(v) => Self::fr(v),
            GridVal::MinMax(mut v) => {
                let mut vals_iter = v.drain(..);
                let min_fn: MinTrackSizingFunction = vals_iter.next().map(|v| v.into()).unwrap_or_default();
                let max_fn: MaxTrackSizingFunction = vals_iter.next().map(|v| v.into()).unwrap_or_default();
                Self::minmax(min_fn, max_fn)
            }
            GridVal::Many(mut v) => {
                let mut vals_iter = v.drain(..);
                let single: GridTrack = vals_iter.next().map(|v| v.into()).unwrap_or_default();
                single
            }
        }
    }
}

impl From<GridVal> for MinTrackSizingFunction
{
    fn from(value: GridVal) -> Self
    {
        match value {
            GridVal::Auto => Self::Auto,
            GridVal::MinContent => Self::MinContent,
            GridVal::MaxContent => Self::MaxContent,
            GridVal::Percent(v) => Self::Percent(v),
            GridVal::Px(v) => Self::Px(v),
            GridVal::Vw(v) => Self::Vw(v),
            GridVal::Vh(v) => Self::Vh(v),
            GridVal::VMin(v) => Self::VMin(v),
            GridVal::VMax(v) => Self::VMax(v),
            GridVal::FitContentPx(_)
            | GridVal::FitContentPercent(_)
            | GridVal::Flex(_)
            | GridVal::Fraction(_)
            | GridVal::MinMax(_)
            | GridVal::Many(_) => Self::Auto,
        }
    }
}

impl From<GridVal> for MaxTrackSizingFunction
{
    fn from(value: GridVal) -> Self
    {
        match value {
            GridVal::Auto => Self::Auto,
            GridVal::MinContent => Self::MinContent,
            GridVal::MaxContent => Self::MaxContent,
            GridVal::Percent(v) => Self::Percent(v),
            GridVal::Px(v) => Self::Px(v),
            GridVal::Vw(v) => Self::Vw(v),
            GridVal::Vh(v) => Self::Vh(v),
            GridVal::VMin(v) => Self::VMin(v),
            GridVal::VMax(v) => Self::VMax(v),
            GridVal::Fraction(v) => Self::Fraction(v),
            GridVal::FitContentPx(v) => Self::FitContentPx(v),
            GridVal::FitContentPercent(v) => Self::FitContentPercent(v),
            GridVal::Flex(_) | GridVal::MinMax(_) | GridVal::Many(_) => Self::Auto,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Default, PartialEq)]
#[cfg_attr(feature = "serde", reflect(Serialize, Deserialize))]
/// How many times to repeat a repeated grid track.
///
/// Mirrors [`GridTrackRepetition`].
pub enum GridValRepetition
{
    /// Repeat the track fixed number of times
    Count(u16),
    /// Repeat the track to fill available space
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/repeat#auto-fill>
    AutoFill,
    /// Repeat the track to fill available space but collapse any tracks that do not end up with
    /// an item placed in them.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/repeat#auto-fit>
    AutoFit,
}

impl Into<GridTrackRepetition> for GridValRepetition
{
    fn into(self) -> GridTrackRepetition
    {
        match self {
            Self::Count(count) => GridTrackRepetition::Count(count),
            Self::AutoFill => GridTrackRepetition::AutoFill,
            Self::AutoFit => GridTrackRepetition::AutoFit,
        }
    }
}

impl Default for GridValRepetition
{
    fn default() -> Self
    {
        Self::Count(1)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Like [`Val`] but also accepts additional values for grid layouts.
///
/// Can be converted to a [`RepeatedGridVal`] for use in [`Node`]. Note that many `RepeatedGridVal` variants
/// only accept a `repetition_count`. If [`GridValRepetition::AutoFill`] or [`GridValRepetition::AutoFit`]
/// is set, then it will fall back to `GridValRepetition::Count(1)` for those cases.
///
/// In COB files, a single [`GridVal`] will implicitly convert to `(Count(1), val)`. This reduces boilerplate
/// when constructing grid nodes.
#[derive(Clone, Default, Debug, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Default, PartialEq, Debug)]
#[cfg_attr(feature = "serde", reflect(Serialize, Deserialize))]
pub struct RepeatedGridVal(pub GridValRepetition, pub GridVal);

impl From<RepeatedGridVal> for RepeatedGridTrack
{
    fn from(value: RepeatedGridVal) -> Self
    {
        let repetition_count = match &value.0 {
            GridValRepetition::Count(count) => *count,
            GridValRepetition::AutoFill | GridValRepetition::AutoFit => 1,
        };
        match value.1 {
            GridVal::Auto => RepeatedGridTrack::auto(repetition_count),
            GridVal::FitContentPx(v) => RepeatedGridTrack::fit_content_px(repetition_count, v),
            GridVal::FitContentPercent(v) => RepeatedGridTrack::fit_content_percent(repetition_count, v),
            GridVal::Flex(v) => RepeatedGridTrack::flex(repetition_count, v),
            GridVal::MinContent => RepeatedGridTrack::min_content(repetition_count),
            GridVal::MaxContent => RepeatedGridTrack::max_content(repetition_count),
            GridVal::Percent(v) => RepeatedGridTrack::percent(value.0, v),
            GridVal::Px(v) => RepeatedGridTrack::px(value.0, v),
            GridVal::Vw(v) => RepeatedGridTrack::vw(value.0, v),
            GridVal::Vh(v) => RepeatedGridTrack::vh(value.0, v),
            GridVal::VMin(v) => RepeatedGridTrack::vmin(value.0, v),
            GridVal::VMax(v) => RepeatedGridTrack::vmax(value.0, v),
            GridVal::Fraction(v) => RepeatedGridTrack::fr(repetition_count, v),
            GridVal::MinMax(mut v) => {
                let mut vals_iter = v.drain(..);
                let min_fn: MinTrackSizingFunction = vals_iter.next().map(|v| v.into()).unwrap_or_default();
                let max_fn: MaxTrackSizingFunction = vals_iter.next().map(|v| v.into()).unwrap_or_default();
                Self::minmax(value.0, min_fn, max_fn)
            }
            GridVal::Many(mut v) => {
                let tracks: Vec<GridTrack> = v.drain(..).map(|v| GridTrack::from(v)).collect();
                Self::repeat_many(value.0, tracks)
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`GridPlacement`].
#[derive(Reflect, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct GridInsertion
{
    /// The grid line at which the item should start.
    ///
    /// Lines are 1-indexed.
    ///
    /// Negative indexes count backwards from the end of the grid.
    ///
    /// Zero is treated as 'no start'.
    #[reflect(default)]
    pub start: i16,
    /// How many grid tracks the item should span.
    ///
    /// Zero is treated as 'no span' if both `start` and `end` are non-zero. Otherwise it falls back to `1`.
    ///
    /// Defaults to 1.
    #[reflect(default = "GridInsertion::default_span")]
    pub span: u16,
    /// The grid line at which the item should end.
    ///
    /// Lines are 1-indexed.
    ///
    /// Negative indexes count backwards from the end of the grid.
    ///
    /// Zero is treated as 'no end'.
    #[reflect(default)]
    pub end: i16,
}

impl GridInsertion
{
    fn default_span() -> u16
    {
        1
    }
}

impl Default for GridInsertion
{
    fn default() -> Self
    {
        Self { start: 0, span: Self::default_span(), end: 0 }
    }
}

impl Into<GridPlacement> for GridInsertion
{
    fn into(self) -> GridPlacement
    {
        let mut placement = GridPlacement::default();
        if self.span == 0 && self.start != 0 && self.end != 0 {
            placement = GridPlacement::start_end(self.start, self.end);
        } else {
            if self.start != 0 {
                placement = placement.set_start(self.start);
            }
            if self.span != 0 {
                placement = placement.set_span(self.span);
            }
            if self.end != 0 {
                placement = placement.set_end(self.end);
            }
        }
        placement
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Mirrors [`UiRect`] for stylesheet serialization.
///
/// All fields default to `Val::Px(0.)`.
#[derive(Reflect, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl From<UiRect> for StyleRect
{
    fn from(rect: UiRect) -> Self
    {
        Self {
            left: rect.left,
            right: rect.right,
            top: rect.top,
            bottom: rect.bottom,
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

/// Mirrors [`Overflow`] for the [`FlexNode`] and [`AbsoluteNode`] loadables.
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum Clipping
{
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

impl Into<Overflow> for Clipping
{
    fn into(self) -> Overflow
    {
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
pub enum JustifyLines
{
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
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum JustifyMain
{
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
    /// If a cross-axis has multiple lines due to [`FlexContent::flex_wrap`], stretching will only fill a given
    /// line without overflow.
    ///
    /// Stretch is applied after other sizing and positioning is calculated. It's a kind of 'bonus sizing'.
    ///
    /// If using [`AbsoluteNode`] and [`Dims::top`]/[`Dims::bottom`]/[`Dims::left`]/[`Dims::right`] are set to
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
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
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
/// Mirrors fields in [`Node`].
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dims
{
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
    /// [`FlexContent`] parameters. You must set the `left`/`top` fields to auto if using [`FlexNode`] and
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
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(self, node: &mut Node)
    {
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

/// Controls the flex layout of a node's children.
///
/// Mirrors fields in [`Node`].
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlexContent
{
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
    #[reflect(default = "FlexContent::default_flex_wrap")]
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

impl FlexContent
{
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(self, node: &mut Node)
    {
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

    fn default_flex_wrap() -> FlexWrap
    {
        FlexWrap::NoWrap
    }
}

impl Default for FlexContent
{
    fn default() -> Self
    {
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
    /// Controls how this node should be aligned on its parent's cross axis.
    ///
    /// If not set to [`JustifySelfCross::Auto`], then this overrides the parent's [`FlexContent::justify_cross`]
    /// setting.
    ///
    /// Does nothing if the node's [`Self::margin`] has [`Val::Auto`] set on either of its cross-axis sides.
    ///
    /// Mirrors [`Node::align_self`].
    ///
    /// Defaults to [`JustifySelfCross::Auto`].
    #[reflect(default)]
    pub justify_self_cross: JustifySelfCross,
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
}

impl SelfFlex
{
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(self, node: &mut Node)
    {
        node.margin = self.margin.into();
        node.align_self = self.justify_self_cross.into();
        node.flex_basis = self.flex_basis;
        node.flex_grow = self.flex_grow;
        node.flex_shrink = self.flex_shrink;
    }
}

impl Default for SelfFlex
{
    fn default() -> Self
    {
        Self {
            margin: Default::default(),
            justify_self_cross: Default::default(),
            flex_basis: Default::default(),
            flex_grow: Default::default(),
            flex_shrink: 1.,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls the grid layout of a node's children.
///
/// Mirrors fields in [`Node`].
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridContent
{
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
    /// Controls how lines containing wrapped children should be aligned within the space of the parent.
    ///
    /// Line alignment is calculated after child nodes compute their target sizes, but before stretch factors are
    /// applied.
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
    /// Controls the direction that automatically-placed children are inserted.
    ///
    /// See [`GridAutoFlow`].
    #[reflect(default)]
    pub grid_auto_flow: GridAutoFlow,
    /// See [`Node::grid_auto_rows`].
    #[reflect(default)]
    pub grid_auto_rows: Vec<GridVal>,
    /// See [`Node::grid_auto_columns`].
    #[reflect(default)]
    pub grid_auto_columns: Vec<GridVal>,
    /// See [`Node::grid_template_rows`].
    #[reflect(default)]
    pub grid_template_rows: Vec<RepeatedGridVal>,
    /// See [`Node::grid_template_columns`].
    #[reflect(default)]
    pub grid_template_columns: Vec<RepeatedGridVal>,
}

impl GridContent
{
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(mut self, node: &mut Node)
    {
        node.overflow = self.clipping.into();
        node.overflow_clip_margin = self.clip_margin;
        node.padding = self.padding.into();
        node.align_content = self.justify_lines.into();
        node.justify_content = self.justify_main.into();
        node.align_items = self.justify_cross.into();
        node.column_gap = self.column_gap;
        node.row_gap = self.row_gap;
        node.grid_auto_flow = self.grid_auto_flow;
        node.grid_auto_rows = self.grid_auto_rows.drain(..).map(|v| v.into()).collect();
        node.grid_auto_columns = self.grid_auto_columns.drain(..).map(|v| v.into()).collect();
        node.grid_template_rows = self
            .grid_template_rows
            .drain(..)
            .map(|v| v.into())
            .collect();
        node.grid_template_columns = self
            .grid_template_columns
            .drain(..)
            .map(|v| v.into())
            .collect();
    }
}

impl Default for GridContent
{
    fn default() -> Self
    {
        Self {
            clipping: Default::default(),
            clip_margin: Default::default(),
            padding: Default::default(),
            justify_lines: Default::default(),
            justify_main: Default::default(),
            justify_cross: Default::default(),
            column_gap: Default::default(),
            row_gap: Default::default(),
            grid_auto_flow: Default::default(),
            grid_auto_rows: Default::default(),
            grid_auto_columns: Default::default(),
            grid_template_rows: Default::default(),
            grid_template_columns: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Controls a node's grid behavior in its parent.
///
/// Mirrors fields in [`Node`].
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfGrid
{
    /// Adds space outside the boundary of a node.
    ///
    /// If the main-axis values are set to [`Val::Auto`] then [`JustifyMain`] will do nothing, and similarly for
    /// the cross-axis with [`JustifyCross`].
    ///
    /// Defaults to zero margin.
    #[reflect(default)]
    pub margin: StyleRect,
    /// Controls how this node should be aligned on its parent's cross axis.
    ///
    /// If not set to [`JustifySelfCross::Auto`], then this overrides the parent's [`FlexContent::justify_cross`]
    /// setting.
    ///
    /// Does nothing if the node's [`Self::margin`] has [`Val::Auto`] set on either of its cross-axis sides.
    ///
    /// Mirrors [`Node::align_self`].
    ///
    /// Defaults to [`JustifySelfCross::Auto`].
    #[reflect(default)]
    pub justify_self_cross: JustifySelfCross,
    /// See [`Node::grid_row`].
    #[reflect(default)]
    pub grid_row: GridInsertion,
    /// See [`Node::grid_column`].
    #[reflect(default)]
    pub grid_column: GridInsertion,
}

impl SelfGrid
{
    /// Adds this struct's contents to [`Node`].
    pub fn set_in_node(self, node: &mut Node)
    {
        node.margin = self.margin.into();
        node.align_self = self.justify_self_cross.into();
        node.grid_row = self.grid_row.into();
        node.grid_column = self.grid_column.into();
    }
}

impl Default for SelfGrid
{
    fn default() -> Self
    {
        Self {
            margin: Default::default(),
            justify_self_cross: Default::default(),
            grid_row: Default::default(),
            grid_column: Default::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for absolute-positioned flex nodes.
///
/// Inserts a [`Node`] with [`Display::Flex`] and [`PositionType::Absolute`].
/// Note that if you want an absolute node's position to be controlled by its parent's [`FlexContent`], then set
/// the node's [`Dims::top`]/[`Dims::bottom`]/[`Dims::left`]/[`Dims::right`] fields to [`Val::Auto`].
///
/// See [`FlexNode`] for flexbox-controlled nodes. See [`DisplayControl`] for setting [`Display::None`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbsoluteNode
{
    // TODO: re-enable once #[reflect(flatten)] is available
    // #[reflect(default)]
    // pub dims: Dims,
    // #[reflect(default)]
    // pub content: FlexContent,

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

    // CONTENT FLEX
    /// See [`FlexContent::clipping`].
    #[reflect(default)]
    pub clipping: Clipping,
    /// See [`FlexContent::clip_margin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// See [`FlexContent::padding`].
    #[reflect(default)]
    pub padding: StyleRect,
    /// See [`FlexContent::flex_direction`].
    #[reflect(default)]
    pub flex_direction: FlexDirection,
    /// See [`FlexContent::flex_wrap`].
    #[reflect(default = "FlexContent::default_flex_wrap")]
    pub flex_wrap: FlexWrap,
    /// See [`FlexContent::justify_lines`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// See [`FlexContent::justify_main`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// See [`FlexContent::justify_cross`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// See [`FlexContent::column_gap`].
    #[reflect(default)]
    pub column_gap: Val,
    /// See [`FlexContent::row_gap`].
    #[reflect(default)]
    pub row_gap: Val,
}

impl Into<Node> for AbsoluteNode
{
    fn into(self) -> Node
    {
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
        FlexContent {
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

impl Instruction for AbsoluteNode
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        let display = emut.get::<DisplayControl>().copied().unwrap_or_default();
        let mut node: Node = self.into();
        node.display = display.to_display(Display::Flex);
        emut.insert(DisplayType::Flex);

        emut.insert(node);
    }

    fn revert(entity: Entity, world: &mut World)
    {
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
/// See [`AbsoluteNode`] for absolute-positioned flex nodes. See [`DisplayControl`] for setting [`Display::None`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlexNode
{
    // TODO: re-enable once #[reflect(flatten)] is available
    // #[reflect(default)]
    // pub dims: Dims,
    // #[reflect(default)]
    // pub content: FlexContent,
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
    /// See [`FlexContent::clipping`].
    #[reflect(default)]
    pub clipping: Clipping,
    /// See [`FlexContent::clip_margin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// See [`FlexContent::padding`].
    #[reflect(default)]
    pub padding: StyleRect,
    /// See [`FlexContent::flex_direction`].
    #[reflect(default)]
    pub flex_direction: FlexDirection,
    /// See [`FlexContent::flex_wrap`].
    #[reflect(default = "FlexContent::default_flex_wrap")]
    pub flex_wrap: FlexWrap,
    /// See [`FlexContent::justify_lines`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// See [`FlexContent::justify_main`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// See [`FlexContent::justify_cross`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// See [`FlexContent::column_gap`].
    #[reflect(default)]
    pub column_gap: Val,
    /// See [`FlexContent::row_gap`].
    #[reflect(default)]
    pub row_gap: Val,

    // SELF FLEX
    /// See [`SelfFlex::margin`].
    #[reflect(default)]
    pub margin: StyleRect,
    /// See [`SelfFlex::justify_self_cross`].
    #[reflect(default)]
    pub justify_self_cross: JustifySelfCross,
    /// See [`SelfFlex::flex_basis`].
    #[reflect(default)]
    pub flex_basis: Val,
    /// See [`SelfFlex::flex_grow`].
    #[reflect(default)]
    pub flex_grow: f32,
    /// See [`SelfFlex::flex_shrink`].
    #[reflect(default)]
    pub flex_shrink: f32,
}

impl Into<Node> for FlexNode
{
    fn into(self) -> Node
    {
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
        FlexContent {
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
            justify_self_cross: self.justify_self_cross,
            flex_basis: self.flex_basis,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
        }
        .set_in_node(&mut node);
        node
    }
}

impl Instruction for FlexNode
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        let display = emut.get::<DisplayControl>().copied().unwrap_or_default();
        let mut node: Node = self.into();
        node.display = display.to_display(Display::Flex);
        emut.insert(DisplayType::Flex);

        emut.insert(node);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove_with_requires::<Node>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for absolute-positioned Grid-controlled nodes.
///
/// Inserts a [`Node`] with [`Display::Grid`] and [`PositionType::Absolute`].
///
/// See [`GridNode`] for grid-positioned grid nodes.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbsoluteGridNode
{
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
    /// See [`GridContent::clipping`].
    #[reflect(default)]
    pub clipping: Clipping,
    /// See [`GridContent::clip_margin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// See [`GridContent::padding`].
    #[reflect(default)]
    pub padding: StyleRect,
    /// See [`GridContent::justify_lines`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// See [`GridContent::justify_main`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// See [`GridContent::justify_cross`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// See [`GridContent::column_gap`].
    #[reflect(default)]
    pub column_gap: Val,
    /// See [`GridContent::row_gap`].
    #[reflect(default)]
    pub row_gap: Val,
    /// See [`GridContent::grid_auto_flow`].
    #[reflect(default)]
    pub grid_auto_flow: GridAutoFlow,
    /// See [`GridContent::grid_auto_rows`].
    #[reflect(default)]
    pub grid_auto_rows: Vec<GridVal>,
    /// See [`GridContent::grid_auto_columns`].
    #[reflect(default)]
    pub grid_auto_columns: Vec<GridVal>,
    /// See [`GridContent::grid_template_rows`].
    #[reflect(default)]
    pub grid_template_rows: Vec<RepeatedGridVal>,
    /// See [`GridContent::grid_template_rows`].
    #[reflect(default)]
    pub grid_template_columns: Vec<RepeatedGridVal>,
}

impl Into<Node> for AbsoluteGridNode
{
    fn into(self) -> Node
    {
        let mut node = Node::default();
        node.display = Display::Grid;
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
        GridContent {
            clipping: self.clipping,
            clip_margin: self.clip_margin,
            padding: self.padding,
            justify_lines: self.justify_lines,
            justify_main: self.justify_main,
            justify_cross: self.justify_cross,
            column_gap: self.column_gap,
            row_gap: self.row_gap,
            grid_auto_flow: self.grid_auto_flow,
            grid_auto_rows: self.grid_auto_rows,
            grid_auto_columns: self.grid_auto_columns,
            grid_template_rows: self.grid_template_rows,
            grid_template_columns: self.grid_template_columns,
        }
        .set_in_node(&mut node);

        node
    }
}

impl Instruction for AbsoluteGridNode
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        let display = emut.get::<DisplayControl>().copied().unwrap_or_default();
        let mut node: Node = self.into();
        node.display = display.to_display(Display::Grid);
        emut.insert(DisplayType::Grid);

        emut.insert(node);
    }

    fn revert(entity: Entity, world: &mut World)
    {
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
/// See [`AbsoluteGridNode`] for absolute-positioned grid nodes.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridNode
{
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
    /// See [`GridContent::clipping`].
    #[reflect(default)]
    pub clipping: Clipping,
    /// See [`GridContent::clip_margin`].
    #[reflect(default)]
    pub clip_margin: OverflowClipMargin,
    /// See [`GridContent::padding`].
    #[reflect(default)]
    pub padding: StyleRect,
    /// See [`GridContent::justify_lines`].
    #[reflect(default)]
    pub justify_lines: JustifyLines,
    /// See [`GridContent::justify_main`].
    #[reflect(default)]
    pub justify_main: JustifyMain,
    /// See [`GridContent::justify_cross`].
    #[reflect(default)]
    pub justify_cross: JustifyCross,
    /// See [`GridContent::column_gap`].
    #[reflect(default)]
    pub column_gap: Val,
    /// See [`GridContent::row_gap`].
    #[reflect(default)]
    pub row_gap: Val,
    /// See [`GridContent::grid_auto_flow`].
    #[reflect(default)]
    pub grid_auto_flow: GridAutoFlow,
    /// See [`GridContent::grid_auto_rows`].
    #[reflect(default)]
    pub grid_auto_rows: Vec<GridVal>,
    /// See [`GridContent::grid_auto_columns`].
    #[reflect(default)]
    pub grid_auto_columns: Vec<GridVal>,
    /// See [`GridContent::grid_template_rows`].
    #[reflect(default)]
    pub grid_template_rows: Vec<RepeatedGridVal>,
    /// See [`GridContent::grid_template_rows`].
    #[reflect(default)]
    pub grid_template_columns: Vec<RepeatedGridVal>,

    // SELF GRID
    /// See [`SelfGrid::margin`].
    #[reflect(default)]
    pub margin: StyleRect,
    /// See [`SelfGrid::justify_self_cross`].
    #[reflect(default)]
    pub justify_self_cross: JustifySelfCross,
    /// See [`SelfGrid::grid_row`].
    #[reflect(default)]
    pub grid_row: GridInsertion,
    /// See [`SelfGrid::grid_column`].
    #[reflect(default)]
    pub grid_column: GridInsertion,
}

impl Into<Node> for GridNode
{
    fn into(self) -> Node
    {
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
        GridContent {
            clipping: self.clipping,
            clip_margin: self.clip_margin,
            padding: self.padding,
            justify_lines: self.justify_lines,
            justify_main: self.justify_main,
            justify_cross: self.justify_cross,
            column_gap: self.column_gap,
            row_gap: self.row_gap,
            grid_auto_flow: self.grid_auto_flow,
            grid_auto_rows: self.grid_auto_rows,
            grid_auto_columns: self.grid_auto_columns,
            grid_template_rows: self.grid_template_rows,
            grid_template_columns: self.grid_template_columns,
        }
        .set_in_node(&mut node);
        SelfGrid {
            margin: self.margin,
            justify_self_cross: self.justify_self_cross,
            grid_row: self.grid_row,
            grid_column: self.grid_column,
        }
        .set_in_node(&mut node);

        node
    }
}

impl Instruction for GridNode
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        let display = emut.get::<DisplayControl>().copied().unwrap_or_default();
        let mut node: Node = self.into();
        node.display = display.to_display(Display::Grid);
        emut.insert(DisplayType::Grid);

        emut.insert(node);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove_with_requires::<(Node, DisplayType)>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable that toggles the [`Node::display`] field.
///
/// Inserts self as a component so the `AbsoluteNode`, `FlexNode`, `AbsoluteGridNode`, and `GridNode` loadables can
/// read the correct display value when they are applied.
#[derive(Component, Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum DisplayControl
{
    /// Corresponds to [`Display::Flex`] and [`Display::Grid`].
    ///
    /// The correct layout algorithm will be internally tracked so it can be recovered when toggling from `Hide`
    /// to `Show`.
    #[default]
    Show,
    /// Corresponds to [`Display::None`].
    Hide,
}

impl DisplayControl
{
    fn refresh(
        mut nodes: Query<
            (&mut Node, &DisplayControl, Option<&DisplayType>),
            Or<(Changed<Node>, Changed<DisplayControl>)>,
        >,
    )
    {
        for (mut node, control, maybe_cache) in nodes.iter_mut() {
            let cache = maybe_cache.copied().unwrap_or_default();
            let display = control.to_display(cache.into());
            if node.display != display {
                node.display = display;
            }
        }
    }

    /// Converts self to [`Display`].
    ///
    /// If self is `Self::Show`, then the `request` value will be returned. Otherwise `Display::None` will be
    /// returned.
    pub fn to_display(&self, request: Display) -> Display
    {
        match self {
            Self::Show => request,
            Self::Hide => Display::None,
        }
    }
}

impl Instruction for DisplayControl
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.insert(self);
        if !emut.contains::<DisplayType>() {
            let display = emut.get::<Node>().map(|n| n.display).unwrap_or_default();
            // NOTE: this will fall back to DisplayType::Flex if it's currently Display::None. We assume the
            // user only controls `Display` using `DisplayControl` and `*Node` instructions.
            emut.insert(DisplayType::from(display));
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
            let cache = e.get::<DisplayType>().copied().unwrap_or_default();
            if let Some(mut node) = e.get_mut::<Node>() {
                node.display = cache.into();
            }
        });
    }
}

impl StaticAttribute for DisplayControl
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct StyleWrappersPlugin;

impl Plugin for StyleWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<AbsoluteNode>()
            .register_instruction_type::<FlexNode>()
            .register_instruction_type::<AbsoluteGridNode>()
            .register_instruction_type::<GridNode>()
            .register_static::<DisplayControl>()
            .add_systems(PostUpdate, DisplayControl::refresh.before(UiSystem::Prepare));
    }
}

//-------------------------------------------------------------------------------------------------------------------

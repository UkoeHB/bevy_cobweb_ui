use std::cmp::Ordering;
use std::ops::Add;

use bevy::prelude::*;
use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for building font requests.
pub trait UpdateFontRequest
{
    fn update(self, req: FontRequest) -> FontRequest;
}

//-------------------------------------------------------------------------------------------------------------------

/// The ID of a family of fonts.
///
/// Example: `"Fira Sans"`.
///
/// Note that [generic][generic-families] font families are not currently supported (e.g. `serif`). Only explicit
/// font names can be used.
///
/// [generic-families]: https://drafts.csswg.org/css-fonts-4/#generic-family-name-syntax
#[derive(Reflect, Default, Deref, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct FontFamily(pub SmolStr);

impl FontFamily
{
    /// Makes a new font family.
    pub fn new(s: impl Into<SmolStr>) -> Self
    {
        Self(s.into())
    }

    /// Makes a new font family from a static string.
    pub const fn new_static(s: &'static str) -> Self
    {
        Self(SmolStr::new_static(s))
    }

    /// Converts the family into a [`FontRequest`].
    pub fn request(self) -> FontRequest
    {
        FontRequest::from(self)
    }
}

impl<S: Into<SmolStr>> From<S> for FontFamily
{
    fn from(s: S) -> Self
    {
        Self::new(s)
    }
}

impl UpdateFontRequest for FontFamily
{
    fn update(self, mut req: FontRequest) -> FontRequest
    {
        req.family = self;
        req
    }
}

/// Adds font family methods to a type.
pub trait FontFamilyExt
{
    fn family(self, family: impl Into<FontFamily>) -> FontRequest;
}

impl<T: Into<FontRequest>> FontFamilyExt for T
{
    fn family(self, family: impl Into<FontFamily>) -> FontRequest
    {
        self.into().set(family.into())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Font widths from [CSS][css-spec].
///
/// [css-spec]: https://drafts.csswg.org/css-fonts-4/#propdef-font-width
#[derive(Reflect, Default, Debug, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum FontWidth
{
    /// 0.5
    UltraCondensed,
    /// 0.625
    ExtraCondensed,
    /// 0.75
    Condensed,
    /// 1.0
    #[default]
    Normal,
    /// 1.125
    SemiExpanded,
    /// 1.25
    Expanded,
    /// 1.5
    ExtraExpanded,
    /// 2.0
    UltraExpanded,
    /// A custom font width from 0.5 to 2.0.
    Width(f32),
}

impl FontWidth
{
    /// Returns the font width as a percent in `[0.5, 2.0]`.
    pub fn width(&self) -> f32
    {
        match self {
            Self::UltraCondensed => 0.5,
            Self::ExtraCondensed => 0.625,
            Self::Condensed => 0.75,
            Self::Normal => 1.,
            Self::SemiExpanded => 1.125,
            Self::Expanded => 1.25,
            Self::ExtraExpanded => 1.5,
            Self::UltraExpanded => 2.,
            Self::Width(width) => width.clamp(0.5, 2.),
        }
    }

    /// Identifies the best-matching font width based on a requested width.
    ///
    /// `test_vals_fn` should return an iterator over available widths.
    ///
    /// We follow the [CSS font matching algorithm][css-algo] as close as possible.
    ///
    /// [css-algo]: https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm
    pub fn negotiate<I, F>(request: FontWidth, test_vals_fn: F) -> Option<FontWidth>
    where
        I: Iterator<Item = FontWidth>,
        F: Fn() -> I,
    {
        // Look for values closest to our request above and below the request.
        let width = request.width();
        let mut nearest_min: Option<f32> = None; // nearest to width in the direction of min (0.5)
        let mut nearest_max: Option<f32> = None; // nearest to width in the direction of max (2.0)
        for test_width in (test_vals_fn)().map(|w| w.width()) {
            // Check if we found a better candidate.
            if test_width == width {
                return Some(request);
            } else if test_width < width {
                let diff = width - test_width;
                if diff <= (width - nearest_min.unwrap_or(0.5)) {
                    nearest_min = Some(test_width);
                }
            } else {
                let diff = test_width - width;
                if diff <= (nearest_max.unwrap_or(2.) - width) {
                    nearest_max = Some(test_width);
                }
            }
        }

        // When requested width is <= 100% we favor smaller widths.
        if width <= 1. {
            if let Some(nearest_min) = nearest_min {
                return Some(FontWidth::Width(nearest_min));
            }
            if let Some(nearest_max) = nearest_max {
                return Some(FontWidth::Width(nearest_max));
            }
        }

        // When requested width is > 100% we favor larger widths.
        if width > 1. {
            if let Some(nearest_max) = nearest_max {
                return Some(FontWidth::Width(nearest_max));
            }
            if let Some(nearest_min) = nearest_min {
                return Some(FontWidth::Width(nearest_min));
            }
        }

        None
    }
}

impl PartialEq for FontWidth
{
    fn eq(&self, other: &Self) -> bool
    {
        self.width() == other.width()
    }
}

impl PartialOrd for FontWidth
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        self.width().partial_cmp(&other.width())
    }
}

impl UpdateFontRequest for FontWidth
{
    fn update(self, mut req: FontRequest) -> FontRequest
    {
        req.width = self;
        req
    }
}

/// Adds font width methods to a type.
pub trait FontWidthExt
{
    /// See [`FontWidth::UltraCondensed`].
    fn ultra_condensed(self) -> FontRequest;
    /// See [`FontWidth::ExtraCondensed`].
    fn extra_condensed(self) -> FontRequest;
    /// See [`FontWidth::Condensed`].
    fn condensed(self) -> FontRequest;
    /// See [`FontWidth::Normal`].
    fn normal_width(self) -> FontRequest;
    /// See [`FontWidth::SemiExpanded`].
    fn semi_expanded(self) -> FontRequest;
    /// See [`FontWidth::Expanded`].
    fn expanded(self) -> FontRequest;
    /// See [`FontWidth::ExtraExpanded`].
    fn extra_expanded(self) -> FontRequest;
    /// See [`FontWidth::UltraExpanded`].
    fn ultra_expanded(self) -> FontRequest;
    /// See [`FontWidth::Width`].
    fn width(self, width: f32) -> FontRequest;
}

impl<T: Into<FontRequest>> FontWidthExt for T
{
    fn ultra_condensed(self) -> FontRequest
    {
        self.into() + FontWidth::UltraCondensed
    }
    fn extra_condensed(self) -> FontRequest
    {
        self.into() + FontWidth::ExtraCondensed
    }
    fn condensed(self) -> FontRequest
    {
        self.into() + FontWidth::Condensed
    }
    fn normal_width(self) -> FontRequest
    {
        self.into() + FontWidth::Normal
    }
    fn semi_expanded(self) -> FontRequest
    {
        self.into() + FontWidth::SemiExpanded
    }
    fn expanded(self) -> FontRequest
    {
        self.into() + FontWidth::Expanded
    }
    fn extra_expanded(self) -> FontRequest
    {
        self.into() + FontWidth::ExtraExpanded
    }
    fn ultra_expanded(self) -> FontRequest
    {
        self.into() + FontWidth::UltraExpanded
    }
    fn width(self, width: f32) -> FontRequest
    {
        self.into() + FontWidth::Width(FontWidth::Width(width).width())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Style of a font from [CSS](https://developer.mozilla.org/en-US/docs/Web/CSS/font-style).
///
/// Takes precedence over [`FontWeight`] when negotiating a [`FontRequest`] against available fonts (see
/// [`FontMap`](crate::prelude::FontMap)).
#[derive(Reflect, Default, Debug, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum FontStyle
{
    /// Font classified as normal in the font family.
    ///
    /// This is equivalent to `Self::Oblique(Some(0))`.
    #[default]
    Normal,
    /// Font classified as italic in the font family.
    ///
    /// Falls back to `Self::Oblique(None)`.
    ///
    /// Currently we assume italics are forward-slanting.
    Italic,
    /// Font classified as oblique in the font family.
    ///
    /// Falls back to `Self::Italic`.
    ///
    /// See [CSS font-style oblique](https://developer.mozilla.org/en-US/docs/Web/CSS/font-style#oblique_2) for
    /// details about the oblique angle. A value of `None` corresponds to 14 degrees.
    Oblique(Option<i16>),
}

impl FontStyle
{
    /// Gets the inner oblique angle if this style is oblique.
    ///
    /// Returns `Some(0)` for `Self::Normal` styles.
    pub fn oblique_angle(&self) -> Option<i16>
    {
        match *self {
            Self::Normal => Some(0),
            Self::Italic => None,
            Self::Oblique(angle) => Some(Self::normalize_oblique(angle)),
        }
    }

    /// Identifies the best-matching font style based on a requested style.
    ///
    /// `test_vals_fn` should return an iterator over available styles.
    ///
    /// We follow the [CSS font matching algorithm][css-algo] as close as possible.
    ///
    /// [css-algo]: https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm
    pub fn negotiate<I, F>(request: FontStyle, test_vals_fn: F) -> Option<FontStyle>
    where
        I: Iterator<Item = FontStyle>,
        F: Fn() -> I,
    {
        match request {
            Self::Normal => Self::negotiate_inner(Self::Oblique(Some(0)), &test_vals_fn)
                .or_else(|| Self::negotiate_inner(Self::Italic, &test_vals_fn))
                .or_else(|| Self::negotiate_inner(Self::Oblique(Some(-1)), &test_vals_fn)),
            Self::Italic => Self::negotiate_inner(Self::Italic, &test_vals_fn)
                .or_else(|| Self::negotiate_inner(Self::Oblique(Some(11)), &test_vals_fn))
                .or_else(|| Self::negotiate_inner(Self::Oblique(Some(-1)), &test_vals_fn)),
            Self::Oblique(angle) => {
                let angle = Self::normalize_oblique(angle);
                let mut angle_res = Self::negotiate_inner(Self::Oblique(Some(angle)), &test_vals_fn);

                // We assume italics have positive slant.
                if angle >= 0 {
                    angle_res = angle_res.or_else(|| Self::negotiate_inner(Self::Italic, &test_vals_fn));
                }

                let opposite = if angle >= 0 { -1 } else { 0 };
                angle_res =
                    angle_res.or_else(|| Self::negotiate_inner(Self::Oblique(Some(opposite)), &test_vals_fn));

                if angle < 0 {
                    angle_res = angle_res.or_else(|| Self::negotiate_inner(Self::Italic, &test_vals_fn));
                }

                angle_res
            }
        }
    }

    fn negotiate_inner<I, F>(request: FontStyle, test_vals_fn: &F) -> Option<FontStyle>
    where
        I: Iterator<Item = FontStyle>,
        F: Fn() -> I,
    {
        match request {
            Self::Normal | Self::Italic => (*test_vals_fn)().find(|t| *t == request),
            // Search for the closest angle with the same sign as the input angle.
            Self::Oblique(angle) => {
                let angle = Self::normalize_oblique(angle);
                let mut nearest_zero: Option<i16> = None; // nearest to angle in the direction of zero
                let mut nearest_max: Option<i16> = None; // nearest to angle in the direction of max (+90 or -90)
                for test_angle in (test_vals_fn)().filter_map(|s| s.oblique_angle()) {
                    // Check sign is the same.
                    if !((angle >= 0 && test_angle >= 0) || (angle < 0 && test_angle < 0)) {
                        return None;
                    }

                    // Check if we found a better candidate.
                    if test_angle.abs() == angle.abs() {
                        return Some(request);
                    } else if test_angle.abs() < angle.abs() {
                        let diff = angle.abs() - test_angle.abs();
                        if diff <= (angle.abs() - nearest_zero.unwrap_or(0).abs()) {
                            nearest_zero = Some(test_angle);
                        }
                    } else {
                        let diff = test_angle.abs() - angle.abs();
                        if diff <= (nearest_max.unwrap_or(90).abs() - angle.abs()) {
                            nearest_max = Some(test_angle);
                        }
                    }
                }

                // When requested angle is below 11 degrees we favor smaller angles.
                if angle.abs() < 11 {
                    if nearest_zero.is_some() {
                        return Some(FontStyle::Oblique(nearest_zero));
                    }
                    if nearest_max.is_some() {
                        return Some(FontStyle::Oblique(nearest_max));
                    }
                }

                // When requested angle is >= 11 degrees we favor larger angles.
                if angle.abs() >= 11 {
                    if nearest_max.is_some() {
                        return Some(FontStyle::Oblique(nearest_max));
                    }
                    if nearest_zero.is_some() {
                        return Some(FontStyle::Oblique(nearest_zero));
                    }
                }

                None
            }
        }
    }

    /// See https://drafts.csswg.org/css-fonts-4/#font-style-prop
    fn normalize_oblique(angle: Option<i16>) -> i16
    {
        angle.unwrap_or(14).clamp(-90, 90)
    }
}

impl PartialEq for FontStyle
{
    fn eq(&self, other: &Self) -> bool
    {
        match *self {
            Self::Normal => matches!(*other, Self::Normal) || matches!(*other, Self::Oblique(Some(0))),
            Self::Italic => matches!(*other, Self::Italic),
            Self::Oblique(angle) => match *other {
                Self::Normal => angle == Some(0),
                Self::Oblique(other_angle) => {
                    Self::normalize_oblique(angle) == Self::normalize_oblique(other_angle)
                }
                Self::Italic => false,
            },
        }
    }
}

impl UpdateFontRequest for FontStyle
{
    fn update(self, mut req: FontRequest) -> FontRequest
    {
        req.style = self;
        req
    }
}

/// Adds font style methods to a type.
pub trait FontStyleExt
{
    /// See [`FontStyle::Normal`].
    fn normal_style(self) -> FontRequest;
    /// See [`FontStyle::Italic`].
    fn italic(self) -> FontRequest;
    /// See [`FontStyle::Oblique`].
    fn oblique(self) -> FontRequest;
    /// See [`FontStyle::Oblique`].
    fn oblique_angle(self, angle: i16) -> FontRequest;
}

impl<T: Into<FontRequest>> FontStyleExt for T
{
    fn normal_style(self) -> FontRequest
    {
        self.into() + FontStyle::Normal
    }
    fn italic(self) -> FontRequest
    {
        self.into() + FontStyle::Italic
    }
    fn oblique(self) -> FontRequest
    {
        self.into() + FontStyle::Oblique(None)
    }
    fn oblique_angle(self, angle: i16) -> FontRequest
    {
        self.into() + FontStyle::Oblique(Some(FontStyle::normalize_oblique(Some(angle))))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Font weights from the OpenType [specification][open-type-spec].
///
/// [open-type-spec]: https://learn.microsoft.com/en-us/typography/opentype/spec/os2#usweightclass
#[derive(Reflect, Default, Debug, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum FontWeight
{
    /// 100
    Thin,
    /// 100
    Hairline,
    /// 200
    ExtraLight,
    /// 200
    UltraLight,
    /// 300
    Light,
    /// 400
    #[default]
    Normal,
    /// 400
    Regular,
    /// 500
    Medium,
    /// 600
    SemiBold,
    /// 600
    DemiBold,
    /// 700
    Bold,
    /// 800
    ExtraBold,
    /// 800
    UltraBold,
    /// 900
    Black,
    /// 900
    Heavy,
    /// 950
    ExtraBlack,
    /// 950
    UltraBlack,
    /// A custom font weight from 1 to 1000.
    Weight(u16),
}

impl FontWeight
{
    /// Returns the font weight as an integer in `[1, 1000]`.
    pub fn weight(&self) -> u16
    {
        match *self {
            Self::Thin => 100,
            Self::Hairline => 100,
            Self::ExtraLight => 200,
            Self::UltraLight => 200,
            Self::Light => 300,
            Self::Normal => 400,
            Self::Regular => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::DemiBold => 600,
            Self::Bold => 700,
            Self::ExtraBold => 800,
            Self::UltraBold => 800,
            Self::Black => 900,
            Self::Heavy => 900,
            Self::ExtraBlack => 950,
            Self::UltraBlack => 950,
            Self::Weight(weight) => weight.clamp(1, 1000),
        }
    }

    /// Identifies the best-matching font weight based on a requested weight.
    ///
    /// `test_vals_fn` should return an iterator over available weights.
    ///
    /// We follow the [CSS font matching algorithm][css-algo] as close as possible.
    ///
    /// [css-algo]: https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm
    pub fn negotiate<I, F>(request: FontWeight, test_vals_fn: F) -> Option<FontWeight>
    where
        I: Iterator<Item = FontWeight>,
        F: Fn() -> I,
    {
        // Look for values closest to our request above and below the request.
        let weight = request.weight();
        let mut nearest_min: Option<u16> = None; // nearest to weight in the direction of min (1)
        let mut nearest_max: Option<u16> = None; // nearest to weight in the direction of max (1000)
        for test_weight in (test_vals_fn)().map(|w| w.weight()) {
            // Check if we found a better candidate.
            if test_weight == weight {
                return Some(request);
            } else if test_weight < weight {
                let diff = weight - test_weight;
                if diff <= (weight - nearest_min.unwrap_or(1)) {
                    nearest_min = Some(test_weight);
                }
            } else {
                let diff = test_weight - weight;
                if diff <= (nearest_max.unwrap_or(1000) - weight) {
                    nearest_max = Some(test_weight);
                }
            }
        }

        // When requested weight is in [400, 500], favor weights in [weight, 500], then [0, weight), then (500,
        // 1000].
        if weight >= 400 && weight <= 500 {
            if let Some(nearest_max) = nearest_max {
                if nearest_max <= 500 {
                    return Some(FontWeight::Weight(nearest_max));
                }
            }
            if let Some(nearest_min) = nearest_min {
                return Some(FontWeight::Weight(nearest_min));
            }
            if let Some(nearest_max) = nearest_max {
                return Some(FontWeight::Weight(nearest_max));
            }
        }

        // When requested weight is < 400 we favor smaller weights.
        if weight < 400 {
            if let Some(nearest_min) = nearest_min {
                return Some(FontWeight::Weight(nearest_min));
            }
            if let Some(nearest_max) = nearest_max {
                return Some(FontWeight::Weight(nearest_max));
            }
        }

        // When requested weight is > 500 we favor larger weights.
        if weight > 500 {
            if let Some(nearest_max) = nearest_max {
                return Some(FontWeight::Weight(nearest_max));
            }
            if let Some(nearest_min) = nearest_min {
                return Some(FontWeight::Weight(nearest_min));
            }
        }

        None
    }
}

impl PartialEq for FontWeight
{
    fn eq(&self, other: &Self) -> bool
    {
        self.weight() == other.weight()
    }
}

impl PartialOrd for FontWeight
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        self.weight().partial_cmp(&other.weight())
    }
}

impl UpdateFontRequest for FontWeight
{
    fn update(self, mut req: FontRequest) -> FontRequest
    {
        req.weight = self;
        req
    }
}

/// Adds font weight methods to a type.
pub trait FontWeightExt
{
    /// See [`FontWeight::Thin`].
    fn thin(self) -> FontRequest;
    /// See [`FontWeight::Hairline`].
    fn hairline(self) -> FontRequest;
    /// See [`FontWeight::ExtraLight`].
    fn extra_light(self) -> FontRequest;
    /// See [`FontWeight::UltraLight`].
    fn ultra_light(self) -> FontRequest;
    /// See [`FontWeight::Light`].
    fn light(self) -> FontRequest;
    /// See [`FontWeight::Normal`].
    fn normal_weight(self) -> FontRequest;
    /// See [`FontWeight::Regular`].
    fn regular(self) -> FontRequest;
    /// See [`FontWeight::Medium`].
    fn medium(self) -> FontRequest;
    /// See [`FontWeight::SemiBold`].
    fn semi_bold(self) -> FontRequest;
    /// See [`FontWeight::DemiBold`].
    fn demi_bold(self) -> FontRequest;
    /// See [`FontWeight::Bold`].
    fn bold(self) -> FontRequest;
    /// See [`FontWeight::ExtraBold`].
    fn extra_bold(self) -> FontRequest;
    /// See [`FontWeight::UltraBold`].
    fn ultra_bold(self) -> FontRequest;
    /// See [`FontWeight::Black`].
    fn black(self) -> FontRequest;
    /// See [`FontWeight::Heavy`].
    fn heavy(self) -> FontRequest;
    /// See [`FontWeight::ExtraBlack`].
    fn extra_black(self) -> FontRequest;
    /// See [`FontWeight::UltraBlack`].
    fn ultra_black(self) -> FontRequest;
    /// See [`FontWeight::Weight`].
    fn weight(self, weight: u16) -> FontRequest;
}

impl<T: Into<FontRequest>> FontWeightExt for T
{
    fn thin(self) -> FontRequest
    {
        self.into() + FontWeight::Thin
    }
    fn hairline(self) -> FontRequest
    {
        self.into() + FontWeight::Hairline
    }
    fn extra_light(self) -> FontRequest
    {
        self.into() + FontWeight::ExtraLight
    }
    fn ultra_light(self) -> FontRequest
    {
        self.into() + FontWeight::UltraLight
    }
    fn light(self) -> FontRequest
    {
        self.into() + FontWeight::Light
    }
    fn normal_weight(self) -> FontRequest
    {
        self.into() + FontWeight::Normal
    }
    fn regular(self) -> FontRequest
    {
        self.into() + FontWeight::Regular
    }
    fn medium(self) -> FontRequest
    {
        self.into() + FontWeight::Medium
    }
    fn semi_bold(self) -> FontRequest
    {
        self.into() + FontWeight::SemiBold
    }
    fn demi_bold(self) -> FontRequest
    {
        self.into() + FontWeight::DemiBold
    }
    fn bold(self) -> FontRequest
    {
        self.into() + FontWeight::Bold
    }
    fn extra_bold(self) -> FontRequest
    {
        self.into() + FontWeight::ExtraBold
    }
    fn ultra_bold(self) -> FontRequest
    {
        self.into() + FontWeight::UltraBold
    }
    fn black(self) -> FontRequest
    {
        self.into() + FontWeight::Black
    }
    fn heavy(self) -> FontRequest
    {
        self.into() + FontWeight::Heavy
    }
    fn extra_black(self) -> FontRequest
    {
        self.into() + FontWeight::ExtraBlack
    }
    fn ultra_black(self) -> FontRequest
    {
        self.into() + FontWeight::UltraBlack
    }
    fn weight(self, weight: u16) -> FontRequest
    {
        self.into() + FontWeight::Weight(FontWeight::Weight(weight).weight())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Font attributes that combine with a [`FontFamily`] to make a [`FontRequest`].
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FontAttributes
{
    #[reflect(default)]
    pub width: FontWidth,
    #[reflect(default)]
    pub style: FontStyle,
    #[reflect(default)]
    pub weight: FontWeight,
}

impl FontAttributes
{
    /// Negotiates available attributes against `self` to find the best-matching available font.
    ///
    /// `attrs_fn` should return an iterator over available attributes.
    ///
    /// See the [CSS font-matching algorithm](https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm).
    pub fn negotiate_eligible_fonts<I>(self, attrs_fn: impl Fn() -> I) -> Option<Self>
    where
        I: Iterator<Item = FontAttributes>,
    {
        // Identify the best-fitting font width.
        let width = FontWidth::negotiate(self.width, || (attrs_fn)().map(|a| a.width))?;

        // Identify the best-fitting font style using the identified width.
        let style = FontStyle::negotiate(self.style, || {
            (attrs_fn)().filter(|a| a.width == width).map(|a| a.style)
        })?;

        // Identify the best-fitting font weight using the identified width and style.
        let weight = FontWeight::negotiate(self.weight, || {
            (attrs_fn)()
                .filter(|a| (a.width == width) && (a.style == style))
                .map(|a| a.weight)
        })?;

        Some(Self { width, style, weight })
    }
}

impl UpdateFontRequest for FontAttributes
{
    fn update(self, req: FontRequest) -> FontRequest
    {
        req + self.width + self.style + self.weight
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A font request.
///
/// Can be used with [`FontMap`](crate::prelude::FontMap) to get a handle to the closest matching font.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FontRequest
{
    pub family: FontFamily,
    #[reflect(default)]
    pub width: FontWidth,
    #[reflect(default)]
    pub style: FontStyle,
    #[reflect(default)]
    pub weight: FontWeight,
}

impl FontRequest
{
    /// Makes a new request with normal attributes.
    pub fn new(family: impl Into<FontFamily>) -> Self
    {
        Self {
            family: family.into(),
            width: FontWidth::Normal,
            style: FontStyle::Normal,
            weight: FontWeight::Normal,
        }
    }

    /// Makes a new request with normal attributes from a static family name.
    pub const fn new_static(family: &'static str) -> Self
    {
        Self {
            family: FontFamily::new_static(family),
            width: FontWidth::Normal,
            style: FontStyle::Normal,
            weight: FontWeight::Normal,
        }
    }

    /// Makes a new request with specified values.
    pub fn with(
        family: impl Into<FontFamily>,
        width: impl Into<FontWidth>,
        style: impl Into<FontStyle>,
        weight: impl Into<FontWeight>,
    ) -> Self
    {
        Self {
            family: family.into(),
            width: width.into(),
            style: style.into(),
            weight: weight.into(),
        }
    }

    /// Gets the inner [`FontAttributes`].
    pub fn attributes(&self) -> FontAttributes
    {
        FontAttributes { width: self.width, style: self.style, weight: self.weight }
    }

    /// Sets a value in the request.
    pub fn set(self, val: impl UpdateFontRequest) -> Self
    {
        val.update(self)
    }
}

impl From<FontFamily> for FontRequest
{
    fn from(family: FontFamily) -> Self
    {
        Self::new(family)
    }
}

impl<U: UpdateFontRequest> Add<U> for FontRequest
{
    type Output = FontRequest;
    fn add(self, val: U) -> Self::Output
    {
        self.set(val)
    }
}

impl<U: UpdateFontRequest> Add<U> for FontFamily
{
    type Output = FontRequest;
    fn add(self, val: U) -> Self::Output
    {
        FontRequest::from(self).set(val)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct FontExtPlugin;

impl Plugin for FontExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<FontFamily>()
            .register_type::<FontWidth>()
            .register_type::<FontStyle>()
            .register_type::<FontWeight>()
            .register_type::<FontRequest>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

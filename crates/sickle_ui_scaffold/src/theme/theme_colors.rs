use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::theme_data::Contrast;

/// Custom serialization and deserialization functions necessary for the loading and saving of
/// [`Color`] structs to their hex string representation.
mod serialize_color {

    use bevy::color::{Color, Srgba};
    use serde::{
        de::{Error, Visitor},
        Deserializer, Serializer,
    };

    pub(super) fn serialize<S, T>(color: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Into<Option<Color>> + Clone,
    {
        let color: Option<Color> = Into::into((*color).clone());
        if let Some(color) = color {
            let srgba = color.to_srgba();
            let hex = srgba.to_hex();
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_none()
        }
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }

    struct ColorVisitor;

    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = Color;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("valid color hex string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Color::Srgba(
                Srgba::hex(v).map_err(|err| Error::custom(err.to_string()))?,
            ))
        }
    }
}

pub mod loader {
    use bevy::asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};

    use super::ThemeColors;

    #[derive(Default)]
    pub(crate) struct ThemeColorsLoader;

    impl AssetLoader for ThemeColorsLoader {
        type Asset = ThemeColors;
        type Settings = ();
        type Error = std::io::Error;

        async fn load<'a>(
            &'a self,
            reader: &'a mut Reader<'_>,
            _settings: &'a Self::Settings,
            _load_context: &'a mut LoadContext<'_>,
        ) -> Result<Self::Asset, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let theme_colors_asset = serde_json::from_slice(&bytes)?;
            Ok(theme_colors_asset)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Surface {
    Background,
    Surface,
    SurfaceVariant,
    SurfaceDim,
    SurfaceBright,
    InverseSurface,
}

#[derive(Clone, Copy, Debug)]
pub enum Accent {
    Primary,
    PrimaryFixed,
    PrimaryFixedDim,
    InversePrimary,
    Secondary,
    SecondaryFixed,
    SecondaryFixedDim,
    Tertiary,
    TertiaryFixed,
    TertiaryFixedDim,
    Error,
    Outline,
    OutlineVariant,
    Shadow,
    Scrim,
}

#[derive(Clone, Copy, Debug)]
pub enum Container {
    Primary,
    Secondary,
    Tertiary,
    Error,
    SurfaceLowest,
    SurfaceLow,
    SurfaceMid,
    SurfaceHigh,
    SurfaceHighest,
}

#[derive(Clone, Copy, Debug)]
pub enum OnColor {
    Primary,
    PrimaryContainer,
    PrimaryFixed,
    PrimaryFixedVariant,
    Secondary,
    SecondaryContainer,
    SecondaryFixed,
    SecondaryFixedVariant,
    Tertiary,
    TertiaryContainer,
    TertiaryFixed,
    TertiaryFixedVariant,
    Error,
    ErrorContainer,
    Background,
    Surface,
    SurfaceVariant,
    InverseSurface,
}

#[derive(Clone, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct ExtendedColor {
    pub name: String,
    #[serde(with = "serialize_color")]
    pub color: Color,
    pub description: String,
    pub harmonized: bool,
}

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct CoreColors {
    #[serde(with = "serialize_color")]
    pub primary: Color,

    #[serde(
        serialize_with = "serialize_color::serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary: Option<Color>,

    #[serde(
        serialize_with = "serialize_color::serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub tertiary: Option<Color>,

    #[serde(
        serialize_with = "serialize_color::serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub error: Option<Color>,

    #[serde(
        serialize_with = "serialize_color::serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub neutral: Option<Color>,

    #[serde(
        serialize_with = "serialize_color::serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub neutral_variant: Option<Color>,
}

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemeColors {
    #[serde(with = "serialize_color")]
    pub primary: Color,

    #[serde(with = "serialize_color")]
    pub on_primary: Color,

    #[serde(with = "serialize_color")]
    pub primary_container: Color,

    #[serde(with = "serialize_color")]
    pub on_primary_container: Color,

    #[serde(with = "serialize_color")]
    pub secondary: Color,

    #[serde(with = "serialize_color")]
    pub on_secondary: Color,

    #[serde(with = "serialize_color")]
    pub secondary_container: Color,

    #[serde(with = "serialize_color")]
    pub on_secondary_container: Color,

    #[serde(with = "serialize_color")]
    pub tertiary: Color,

    #[serde(with = "serialize_color")]
    pub on_tertiary: Color,

    #[serde(with = "serialize_color")]
    pub tertiary_container: Color,

    #[serde(with = "serialize_color")]
    pub on_tertiary_container: Color,

    #[serde(with = "serialize_color")]
    pub error: Color,

    #[serde(with = "serialize_color")]
    pub on_error: Color,

    #[serde(with = "serialize_color")]
    pub error_container: Color,

    #[serde(with = "serialize_color")]
    pub on_error_container: Color,

    #[serde(with = "serialize_color")]
    pub background: Color,

    #[serde(with = "serialize_color")]
    pub on_background: Color,

    #[serde(with = "serialize_color")]
    pub surface: Color,

    #[serde(with = "serialize_color")]
    pub on_surface: Color,

    #[serde(with = "serialize_color")]
    pub surface_variant: Color,

    #[serde(with = "serialize_color")]
    pub on_surface_variant: Color,

    #[serde(with = "serialize_color")]
    pub outline: Color,

    #[serde(with = "serialize_color")]
    pub outline_variant: Color,

    #[serde(with = "serialize_color")]
    pub shadow: Color,

    #[serde(with = "serialize_color")]
    pub scrim: Color,

    #[serde(with = "serialize_color")]
    pub inverse_surface: Color,

    #[serde(with = "serialize_color")]
    pub inverse_on_surface: Color,

    #[serde(with = "serialize_color")]
    pub inverse_primary: Color,

    #[serde(with = "serialize_color")]
    pub primary_fixed: Color,

    #[serde(with = "serialize_color")]
    pub on_primary_fixed: Color,

    #[serde(with = "serialize_color")]
    pub primary_fixed_dim: Color,

    #[serde(with = "serialize_color")]
    pub on_primary_fixed_variant: Color,

    #[serde(with = "serialize_color")]
    pub secondary_fixed: Color,

    #[serde(with = "serialize_color")]
    pub on_secondary_fixed: Color,

    #[serde(with = "serialize_color")]
    pub secondary_fixed_dim: Color,

    #[serde(with = "serialize_color")]
    pub on_secondary_fixed_variant: Color,

    #[serde(with = "serialize_color")]
    pub tertiary_fixed: Color,

    #[serde(with = "serialize_color")]
    pub on_tertiary_fixed: Color,

    #[serde(with = "serialize_color")]
    pub tertiary_fixed_dim: Color,

    #[serde(with = "serialize_color")]
    pub on_tertiary_fixed_variant: Color,

    #[serde(with = "serialize_color")]
    pub surface_dim: Color,

    #[serde(with = "serialize_color")]
    pub surface_bright: Color,

    #[serde(with = "serialize_color")]
    pub surface_container_lowest: Color,

    #[serde(with = "serialize_color")]
    pub surface_container_low: Color,

    #[serde(with = "serialize_color")]
    pub surface_container: Color,

    #[serde(with = "serialize_color")]
    pub surface_container_high: Color,

    #[serde(with = "serialize_color")]
    pub surface_container_highest: Color,
}

impl SchemeColors {
    pub fn surface(&self, surface: Surface) -> Color {
        match surface {
            Surface::Background => self.background,
            Surface::Surface => self.surface,
            Surface::InverseSurface => self.inverse_surface,
            Surface::SurfaceVariant => self.surface_variant,
            Surface::SurfaceDim => self.surface_dim,
            Surface::SurfaceBright => self.surface_bright,
        }
    }

    pub fn accent(&self, accent: Accent) -> Color {
        match accent {
            Accent::Primary => self.primary,
            Accent::PrimaryFixed => self.primary_fixed,
            Accent::PrimaryFixedDim => self.primary_fixed_dim,
            Accent::InversePrimary => self.inverse_primary,
            Accent::Secondary => self.secondary,
            Accent::SecondaryFixed => self.secondary_fixed,
            Accent::SecondaryFixedDim => self.secondary_fixed_dim,
            Accent::Tertiary => self.tertiary,
            Accent::TertiaryFixed => self.tertiary_fixed,
            Accent::TertiaryFixedDim => self.tertiary_fixed_dim,
            Accent::Error => self.error,
            Accent::Outline => self.outline,
            Accent::OutlineVariant => self.outline_variant,
            Accent::Shadow => self.shadow,
            Accent::Scrim => self.scrim,
        }
    }

    pub fn container(&self, container: Container) -> Color {
        match container {
            Container::Primary => self.primary_container,
            Container::Secondary => self.secondary_container,
            Container::Tertiary => self.tertiary_container,
            Container::Error => self.error_container,
            Container::SurfaceLowest => self.surface_container_lowest,
            Container::SurfaceLow => self.surface_container_low,
            Container::SurfaceMid => self.surface_container,
            Container::SurfaceHigh => self.surface_container_high,
            Container::SurfaceHighest => self.surface_container_highest,
        }
    }

    pub fn on(&self, on: OnColor) -> Color {
        match on {
            OnColor::Primary => self.on_primary,
            OnColor::PrimaryContainer => self.on_primary_container,
            OnColor::PrimaryFixed => self.on_primary_fixed,
            OnColor::PrimaryFixedVariant => self.on_primary_fixed_variant,
            OnColor::Secondary => self.on_secondary,
            OnColor::SecondaryContainer => self.on_secondary_container,
            OnColor::SecondaryFixed => self.on_secondary_fixed,
            OnColor::SecondaryFixedVariant => self.on_secondary_fixed_variant,
            OnColor::Tertiary => self.on_tertiary,
            OnColor::TertiaryContainer => self.on_tertiary_container,
            OnColor::TertiaryFixed => self.on_tertiary_fixed,
            OnColor::TertiaryFixedVariant => self.on_tertiary_fixed_variant,
            OnColor::Error => self.on_error,
            OnColor::ErrorContainer => self.on_error_container,
            OnColor::Background => self.on_background,
            OnColor::Surface => self.on_surface,
            OnColor::SurfaceVariant => self.on_surface_variant,
            OnColor::InverseSurface => self.inverse_on_surface,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ColorSchemes {
    pub light: SchemeColors,
    pub light_medium_contrast: SchemeColors,
    pub light_high_contrast: SchemeColors,
    pub dark: SchemeColors,
    pub dark_medium_contrast: SchemeColors,
    pub dark_high_contrast: SchemeColors,
}

impl ColorSchemes {
    pub fn light_contrast(&self, contrast: Contrast) -> SchemeColors {
        match contrast {
            Contrast::Standard => self.light,
            Contrast::Medium => self.light_medium_contrast,
            Contrast::High => self.light_high_contrast,
        }
    }

    pub fn dark_contrast(&self, contrast: Contrast) -> SchemeColors {
        match contrast {
            Contrast::Standard => self.dark,
            Contrast::Medium => self.dark_medium_contrast,
            Contrast::High => self.dark_high_contrast,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
pub struct ColorPalette {
    #[serde(rename = "0", with = "serialize_color")]
    pub p_0: Color,

    #[serde(rename = "5", with = "serialize_color")]
    pub p_5: Color,

    #[serde(rename = "10", with = "serialize_color")]
    pub p_10: Color,

    #[serde(rename = "15", with = "serialize_color")]
    pub p_15: Color,

    #[serde(rename = "20", with = "serialize_color")]
    pub p_20: Color,

    #[serde(rename = "25", with = "serialize_color")]
    pub p_25: Color,

    #[serde(rename = "30", with = "serialize_color")]
    pub p_30: Color,

    #[serde(rename = "35", with = "serialize_color")]
    pub p_35: Color,

    #[serde(rename = "40", with = "serialize_color")]
    pub p_40: Color,

    #[serde(rename = "50", with = "serialize_color")]
    pub p_50: Color,

    #[serde(rename = "60", with = "serialize_color")]
    pub p_60: Color,

    #[serde(rename = "70", with = "serialize_color")]
    pub p_70: Color,

    #[serde(rename = "80", with = "serialize_color")]
    pub p_80: Color,

    #[serde(rename = "90", with = "serialize_color")]
    pub p_90: Color,

    #[serde(rename = "95", with = "serialize_color")]
    pub p_95: Color,

    #[serde(rename = "98", with = "serialize_color")]
    pub p_98: Color,

    #[serde(rename = "99", with = "serialize_color")]
    pub p_99: Color,

    #[serde(rename = "100", with = "serialize_color")]
    pub p_100: Color,
}

#[derive(Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ColorPalettes {
    pub primary: ColorPalette,
    pub secondary: ColorPalette,
    pub tertiary: ColorPalette,
    pub neutral: ColorPalette,
    pub neutral_variant: ColorPalette,
}

/// Follows Material3 theme format. For more information (and a web-based theme builder), visit
/// [Material Theme Builder](https://material-foundation.github.io/material-theme-builder/).
#[derive(Asset, Clone, Debug, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    pub description: String,
    // TODO: Generate colors from seed & core colors?
    #[serde(with = "serialize_color")]
    pub seed: Color,
    pub core_colors: CoreColors,
    pub extended_colors: Vec<ExtendedColor>,
    pub schemes: ColorSchemes,
    pub palettes: ColorPalettes,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            description: "Sickle UI Theme".into(),
            seed: Color::Srgba(Srgba::hex("037E90").unwrap()),
            core_colors: CoreColors {
                primary: Color::Srgba(Srgba::hex("BCB4A3").unwrap()),
                secondary: Color::Srgba(Srgba::hex("1A1A1A").unwrap()).into(),
                neutral: Color::Srgba(Srgba::hex("262829").unwrap()).into(),
                ..default()
            },
            extended_colors: Default::default(),
            schemes: ColorSchemes {
                light: SchemeColors {
                    primary: Color::Srgba(Srgba::hex("8B4F24").unwrap()),
                    on_primary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    primary_container: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_primary_container: Color::Srgba(Srgba::hex("311300").unwrap()),
                    secondary: Color::Srgba(Srgba::hex("755846").unwrap()),
                    on_secondary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary_container: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_secondary_container: Color::Srgba(Srgba::hex("2B1709").unwrap()),
                    tertiary: Color::Srgba(Srgba::hex("606134").unwrap()),
                    on_tertiary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary_container: Color::Srgba(Srgba::hex("E6E6AD").unwrap()),
                    on_tertiary_container: Color::Srgba(Srgba::hex("1C1D00").unwrap()),
                    error: Color::Srgba(Srgba::hex("BA1A1A").unwrap()),
                    on_error: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    error_container: Color::Srgba(Srgba::hex("FFDAD6").unwrap()),
                    on_error_container: Color::Srgba(Srgba::hex("410002").unwrap()),
                    background: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    on_background: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    on_surface: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface_variant: Color::Srgba(Srgba::hex("F4DED3").unwrap()),
                    on_surface_variant: Color::Srgba(Srgba::hex("52443C").unwrap()),
                    outline: Color::Srgba(Srgba::hex("FFD200").unwrap()),
                    outline_variant: Color::Srgba(Srgba::hex("D7C3B8").unwrap()),
                    shadow: Color::Srgba(Srgba::hex("000000").unwrap()),
                    scrim: Color::Srgba(Srgba::hex("000000").unwrap()),
                    inverse_surface: Color::Srgba(Srgba::hex("382E29").unwrap()),
                    inverse_on_surface: Color::Srgba(Srgba::hex("FFEDE5").unwrap()),
                    inverse_primary: Color::Srgba(Srgba::hex("FFB688").unwrap()),
                    primary_fixed: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_primary_fixed: Color::Srgba(Srgba::hex("311300").unwrap()),
                    primary_fixed_dim: Color::Srgba(Srgba::hex("FFB688").unwrap()),
                    on_primary_fixed_variant: Color::Srgba(Srgba::hex("6E380F").unwrap()),
                    secondary_fixed: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_secondary_fixed: Color::Srgba(Srgba::hex("2B1709").unwrap()),
                    secondary_fixed_dim: Color::Srgba(Srgba::hex("E5BFA9").unwrap()),
                    on_secondary_fixed_variant: Color::Srgba(Srgba::hex("5B4130").unwrap()),
                    tertiary_fixed: Color::Srgba(Srgba::hex("E6E6AD").unwrap()),
                    on_tertiary_fixed: Color::Srgba(Srgba::hex("1C1D00").unwrap()),
                    tertiary_fixed_dim: Color::Srgba(Srgba::hex("CACA93").unwrap()),
                    on_tertiary_fixed_variant: Color::Srgba(Srgba::hex("48491E").unwrap()),
                    surface_dim: Color::Srgba(Srgba::hex("E7D7CE").unwrap()),
                    surface_bright: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    surface_container_lowest: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    surface_container_low: Color::Srgba(Srgba::hex("FFF1EA").unwrap()),
                    surface_container: Color::Srgba(Srgba::hex("FCEBE2").unwrap()),
                    surface_container_high: Color::Srgba(Srgba::hex("F6E5DC").unwrap()),
                    surface_container_highest: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                },
                light_medium_contrast: SchemeColors {
                    primary: Color::Srgba(Srgba::hex("69350B").unwrap()),
                    on_primary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    primary_container: Color::Srgba(Srgba::hex("A56538").unwrap()),
                    on_primary_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary: Color::Srgba(Srgba::hex("573D2D").unwrap()),
                    on_secondary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary_container: Color::Srgba(Srgba::hex("8D6E5B").unwrap()),
                    on_secondary_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary: Color::Srgba(Srgba::hex("44451B").unwrap()),
                    on_tertiary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary_container: Color::Srgba(Srgba::hex("777748").unwrap()),
                    on_tertiary_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    error: Color::Srgba(Srgba::hex("8C0009").unwrap()),
                    on_error: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    error_container: Color::Srgba(Srgba::hex("DA342E").unwrap()),
                    on_error_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    background: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    on_background: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    on_surface: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface_variant: Color::Srgba(Srgba::hex("F4DED3").unwrap()),
                    on_surface_variant: Color::Srgba(Srgba::hex("4E4038").unwrap()),
                    outline: Color::Srgba(Srgba::hex("FFD200").unwrap()),
                    outline_variant: Color::Srgba(Srgba::hex("88776E").unwrap()),
                    shadow: Color::Srgba(Srgba::hex("000000").unwrap()),
                    scrim: Color::Srgba(Srgba::hex("000000").unwrap()),
                    inverse_surface: Color::Srgba(Srgba::hex("382E29").unwrap()),
                    inverse_on_surface: Color::Srgba(Srgba::hex("FFEDE5").unwrap()),
                    inverse_primary: Color::Srgba(Srgba::hex("FFB688").unwrap()),
                    primary_fixed: Color::Srgba(Srgba::hex("A56538").unwrap()),
                    on_primary_fixed: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    primary_fixed_dim: Color::Srgba(Srgba::hex("884D22").unwrap()),
                    on_primary_fixed_variant: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary_fixed: Color::Srgba(Srgba::hex("8D6E5B").unwrap()),
                    on_secondary_fixed: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary_fixed_dim: Color::Srgba(Srgba::hex("735644").unwrap()),
                    on_secondary_fixed_variant: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary_fixed: Color::Srgba(Srgba::hex("777748").unwrap()),
                    on_tertiary_fixed: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary_fixed_dim: Color::Srgba(Srgba::hex("5E5E32").unwrap()),
                    on_tertiary_fixed_variant: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    surface_dim: Color::Srgba(Srgba::hex("E7D7CE").unwrap()),
                    surface_bright: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    surface_container_lowest: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    surface_container_low: Color::Srgba(Srgba::hex("FFF1EA").unwrap()),
                    surface_container: Color::Srgba(Srgba::hex("FCEBE2").unwrap()),
                    surface_container_high: Color::Srgba(Srgba::hex("F6E5DC").unwrap()),
                    surface_container_highest: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                },

                light_high_contrast: SchemeColors {
                    primary: Color::Srgba(Srgba::hex("3B1800").unwrap()),
                    on_primary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    primary_container: Color::Srgba(Srgba::hex("69350B").unwrap()),
                    on_primary_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary: Color::Srgba(Srgba::hex("321D0F").unwrap()),
                    on_secondary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary_container: Color::Srgba(Srgba::hex("573D2D").unwrap()),
                    on_secondary_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary: Color::Srgba(Srgba::hex("232400").unwrap()),
                    on_tertiary: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary_container: Color::Srgba(Srgba::hex("44451B").unwrap()),
                    on_tertiary_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    error: Color::Srgba(Srgba::hex("4E0002").unwrap()),
                    on_error: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    error_container: Color::Srgba(Srgba::hex("8C0009").unwrap()),
                    on_error_container: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    background: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    on_background: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    on_surface: Color::Srgba(Srgba::hex("000000").unwrap()),
                    surface_variant: Color::Srgba(Srgba::hex("F4DED3").unwrap()),
                    on_surface_variant: Color::Srgba(Srgba::hex("2D211A").unwrap()),
                    outline: Color::Srgba(Srgba::hex("FFD200").unwrap()),
                    outline_variant: Color::Srgba(Srgba::hex("4E4038").unwrap()),
                    shadow: Color::Srgba(Srgba::hex("000000").unwrap()),
                    scrim: Color::Srgba(Srgba::hex("000000").unwrap()),
                    inverse_surface: Color::Srgba(Srgba::hex("382E29").unwrap()),
                    inverse_on_surface: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    inverse_primary: Color::Srgba(Srgba::hex("FFE7DB").unwrap()),
                    primary_fixed: Color::Srgba(Srgba::hex("69350B").unwrap()),
                    on_primary_fixed: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    primary_fixed_dim: Color::Srgba(Srgba::hex("4B2100").unwrap()),
                    on_primary_fixed_variant: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary_fixed: Color::Srgba(Srgba::hex("573D2D").unwrap()),
                    on_secondary_fixed: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    secondary_fixed_dim: Color::Srgba(Srgba::hex("3E2718").unwrap()),
                    on_secondary_fixed_variant: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary_fixed: Color::Srgba(Srgba::hex("44451B").unwrap()),
                    on_tertiary_fixed: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    tertiary_fixed_dim: Color::Srgba(Srgba::hex("2E2E06").unwrap()),
                    on_tertiary_fixed_variant: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    surface_dim: Color::Srgba(Srgba::hex("E7D7CE").unwrap()),
                    surface_bright: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    surface_container_lowest: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    surface_container_low: Color::Srgba(Srgba::hex("FFF1EA").unwrap()),
                    surface_container: Color::Srgba(Srgba::hex("FCEBE2").unwrap()),
                    surface_container_high: Color::Srgba(Srgba::hex("F6E5DC").unwrap()),
                    surface_container_highest: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                },
                dark: SchemeColors {
                    primary: Color::Srgba(Srgba::hex("FFB688").unwrap()),
                    on_primary: Color::Srgba(Srgba::hex("512400").unwrap()),
                    primary_container: Color::Srgba(Srgba::hex("6E380F").unwrap()),
                    on_primary_container: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    secondary: Color::Srgba(Srgba::hex("E5BFA9").unwrap()),
                    on_secondary: Color::Srgba(Srgba::hex("432B1C").unwrap()),
                    secondary_container: Color::Srgba(Srgba::hex("5B4130").unwrap()),
                    on_secondary_container: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    tertiary: Color::Srgba(Srgba::hex("CACA93").unwrap()),
                    on_tertiary: Color::Srgba(Srgba::hex("32320A").unwrap()),
                    tertiary_container: Color::Srgba(Srgba::hex("48491E").unwrap()),
                    on_tertiary_container: Color::Srgba(Srgba::hex("E6E6AD").unwrap()),
                    error: Color::Srgba(Srgba::hex("FFB4AB").unwrap()),
                    on_error: Color::Srgba(Srgba::hex("690005").unwrap()),
                    error_container: Color::Srgba(Srgba::hex("93000A").unwrap()),
                    on_error_container: Color::Srgba(Srgba::hex("FFDAD6").unwrap()),
                    background: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    on_background: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                    surface: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    on_surface: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                    surface_variant: Color::Srgba(Srgba::hex("52443C").unwrap()),
                    on_surface_variant: Color::Srgba(Srgba::hex("D7C3B8").unwrap()),
                    outline: Color::Srgba(Srgba::hex("FFD200").unwrap()),
                    outline_variant: Color::Srgba(Srgba::hex("52443C").unwrap()),
                    shadow: Color::Srgba(Srgba::hex("000000").unwrap()),
                    scrim: Color::Srgba(Srgba::hex("000000").unwrap()),
                    inverse_surface: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                    inverse_on_surface: Color::Srgba(Srgba::hex("382E29").unwrap()),
                    inverse_primary: Color::Srgba(Srgba::hex("8B4F24").unwrap()),
                    primary_fixed: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_primary_fixed: Color::Srgba(Srgba::hex("311300").unwrap()),
                    primary_fixed_dim: Color::Srgba(Srgba::hex("FFB688").unwrap()),
                    on_primary_fixed_variant: Color::Srgba(Srgba::hex("6E380F").unwrap()),
                    secondary_fixed: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_secondary_fixed: Color::Srgba(Srgba::hex("2B1709").unwrap()),
                    secondary_fixed_dim: Color::Srgba(Srgba::hex("E5BFA9").unwrap()),
                    on_secondary_fixed_variant: Color::Srgba(Srgba::hex("5B4130").unwrap()),
                    tertiary_fixed: Color::Srgba(Srgba::hex("E6E6AD").unwrap()),
                    on_tertiary_fixed: Color::Srgba(Srgba::hex("1C1D00").unwrap()),
                    tertiary_fixed_dim: Color::Srgba(Srgba::hex("CACA93").unwrap()),
                    on_tertiary_fixed_variant: Color::Srgba(Srgba::hex("48491E").unwrap()),
                    surface_dim: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    surface_bright: Color::Srgba(Srgba::hex("413731").unwrap()),
                    surface_container_lowest: Color::Srgba(Srgba::hex("140D08").unwrap()),
                    surface_container_low: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface_container: Color::Srgba(Srgba::hex("261E19").unwrap()),
                    surface_container_high: Color::Srgba(Srgba::hex("312823").unwrap()),
                    surface_container_highest: Color::Srgba(Srgba::hex("3D332D").unwrap()),
                },

                dark_medium_contrast: SchemeColors {
                    primary: Color::Srgba(Srgba::hex("FFBC92").unwrap()),
                    on_primary: Color::Srgba(Srgba::hex("290F00").unwrap()),
                    primary_container: Color::Srgba(Srgba::hex("C68051").unwrap()),
                    on_primary_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    secondary: Color::Srgba(Srgba::hex("E9C3AD").unwrap()),
                    on_secondary: Color::Srgba(Srgba::hex("251105").unwrap()),
                    secondary_container: Color::Srgba(Srgba::hex("AC8A75").unwrap()),
                    on_secondary_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    tertiary: Color::Srgba(Srgba::hex("CECE97").unwrap()),
                    on_tertiary: Color::Srgba(Srgba::hex("171700").unwrap()),
                    tertiary_container: Color::Srgba(Srgba::hex("939361").unwrap()),
                    on_tertiary_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    error: Color::Srgba(Srgba::hex("FFBAB1").unwrap()),
                    on_error: Color::Srgba(Srgba::hex("370001").unwrap()),
                    error_container: Color::Srgba(Srgba::hex("FF5449").unwrap()),
                    on_error_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    background: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    on_background: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                    surface: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    on_surface: Color::Srgba(Srgba::hex("FFFAF8").unwrap()),
                    surface_variant: Color::Srgba(Srgba::hex("52443C").unwrap()),
                    on_surface_variant: Color::Srgba(Srgba::hex("DBC7BC").unwrap()),
                    outline: Color::Srgba(Srgba::hex("FFD200").unwrap()),
                    outline_variant: Color::Srgba(Srgba::hex("918076").unwrap()),
                    shadow: Color::Srgba(Srgba::hex("000000").unwrap()),
                    scrim: Color::Srgba(Srgba::hex("000000").unwrap()),
                    inverse_surface: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                    inverse_on_surface: Color::Srgba(Srgba::hex("312823").unwrap()),
                    inverse_primary: Color::Srgba(Srgba::hex("703A10").unwrap()),
                    primary_fixed: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_primary_fixed: Color::Srgba(Srgba::hex("210B00").unwrap()),
                    primary_fixed_dim: Color::Srgba(Srgba::hex("FFB688").unwrap()),
                    on_primary_fixed_variant: Color::Srgba(Srgba::hex("592800").unwrap()),
                    secondary_fixed: Color::Srgba(Srgba::hex("FFDBC7").unwrap()),
                    on_secondary_fixed: Color::Srgba(Srgba::hex("1F0C02").unwrap()),
                    secondary_fixed_dim: Color::Srgba(Srgba::hex("E5BFA9").unwrap()),
                    on_secondary_fixed_variant: Color::Srgba(Srgba::hex("493121").unwrap()),
                    tertiary_fixed: Color::Srgba(Srgba::hex("E6E6AD").unwrap()),
                    on_tertiary_fixed: Color::Srgba(Srgba::hex("121200").unwrap()),
                    tertiary_fixed_dim: Color::Srgba(Srgba::hex("CACA93").unwrap()),
                    on_tertiary_fixed_variant: Color::Srgba(Srgba::hex("37380F").unwrap()),
                    surface_dim: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    surface_bright: Color::Srgba(Srgba::hex("413731").unwrap()),
                    surface_container_lowest: Color::Srgba(Srgba::hex("140D08").unwrap()),
                    surface_container_low: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface_container: Color::Srgba(Srgba::hex("261E19").unwrap()),
                    surface_container_high: Color::Srgba(Srgba::hex("312823").unwrap()),
                    surface_container_highest: Color::Srgba(Srgba::hex("3D332D").unwrap()),
                },
                dark_high_contrast: SchemeColors {
                    primary: Color::Srgba(Srgba::hex("FFFAF8").unwrap()),
                    on_primary: Color::Srgba(Srgba::hex("000000").unwrap()),
                    primary_container: Color::Srgba(Srgba::hex("FFBC92").unwrap()),
                    on_primary_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    secondary: Color::Srgba(Srgba::hex("FFFAF8").unwrap()),
                    on_secondary: Color::Srgba(Srgba::hex("000000").unwrap()),
                    secondary_container: Color::Srgba(Srgba::hex("E9C3AD").unwrap()),
                    on_secondary_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    tertiary: Color::Srgba(Srgba::hex("FFFEC5").unwrap()),
                    on_tertiary: Color::Srgba(Srgba::hex("000000").unwrap()),
                    tertiary_container: Color::Srgba(Srgba::hex("CECE97").unwrap()),
                    on_tertiary_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    error: Color::Srgba(Srgba::hex("FFF9F9").unwrap()),
                    on_error: Color::Srgba(Srgba::hex("000000").unwrap()),
                    error_container: Color::Srgba(Srgba::hex("FFBAB1").unwrap()),
                    on_error_container: Color::Srgba(Srgba::hex("000000").unwrap()),
                    background: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    on_background: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                    surface: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    on_surface: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                    surface_variant: Color::Srgba(Srgba::hex("52443C").unwrap()),
                    on_surface_variant: Color::Srgba(Srgba::hex("FFFAF8").unwrap()),
                    outline: Color::Srgba(Srgba::hex("FFD200").unwrap()),
                    outline_variant: Color::Srgba(Srgba::hex("DBC7BC").unwrap()),
                    shadow: Color::Srgba(Srgba::hex("000000").unwrap()),
                    scrim: Color::Srgba(Srgba::hex("000000").unwrap()),
                    inverse_surface: Color::Srgba(Srgba::hex("F0DFD7").unwrap()),
                    inverse_on_surface: Color::Srgba(Srgba::hex("000000").unwrap()),
                    inverse_primary: Color::Srgba(Srgba::hex("471F00").unwrap()),
                    primary_fixed: Color::Srgba(Srgba::hex("FFE1D0").unwrap()),
                    on_primary_fixed: Color::Srgba(Srgba::hex("000000").unwrap()),
                    primary_fixed_dim: Color::Srgba(Srgba::hex("FFBC92").unwrap()),
                    on_primary_fixed_variant: Color::Srgba(Srgba::hex("290F00").unwrap()),
                    secondary_fixed: Color::Srgba(Srgba::hex("FFE1D0").unwrap()),
                    on_secondary_fixed: Color::Srgba(Srgba::hex("000000").unwrap()),
                    secondary_fixed_dim: Color::Srgba(Srgba::hex("E9C3AD").unwrap()),
                    on_secondary_fixed_variant: Color::Srgba(Srgba::hex("251105").unwrap()),
                    tertiary_fixed: Color::Srgba(Srgba::hex("EBEAB1").unwrap()),
                    on_tertiary_fixed: Color::Srgba(Srgba::hex("000000").unwrap()),
                    tertiary_fixed_dim: Color::Srgba(Srgba::hex("CECE97").unwrap()),
                    on_tertiary_fixed_variant: Color::Srgba(Srgba::hex("171700").unwrap()),
                    surface_dim: Color::Srgba(Srgba::hex("19120D").unwrap()),
                    surface_bright: Color::Srgba(Srgba::hex("413731").unwrap()),
                    surface_container_lowest: Color::Srgba(Srgba::hex("140D08").unwrap()),
                    surface_container_low: Color::Srgba(Srgba::hex("221A15").unwrap()),
                    surface_container: Color::Srgba(Srgba::hex("261E19").unwrap()),
                    surface_container_high: Color::Srgba(Srgba::hex("312823").unwrap()),
                    surface_container_highest: Color::Srgba(Srgba::hex("3D332D").unwrap()),
                },
            },
            palettes: ColorPalettes {
                primary: ColorPalette {
                    p_0: Color::Srgba(Srgba::hex("000000").unwrap()),
                    p_5: Color::Srgba(Srgba::hex("1A0E07").unwrap()),
                    p_10: Color::Srgba(Srgba::hex("261911").unwrap()),
                    p_15: Color::Srgba(Srgba::hex("31231A").unwrap()),
                    p_20: Color::Srgba(Srgba::hex("3C2D24").unwrap()),
                    p_25: Color::Srgba(Srgba::hex("48382F").unwrap()),
                    p_30: Color::Srgba(Srgba::hex("54433A").unwrap()),
                    p_35: Color::Srgba(Srgba::hex("604F45").unwrap()),
                    p_40: Color::Srgba(Srgba::hex("6D5B50").unwrap()),
                    p_50: Color::Srgba(Srgba::hex("877368").unwrap()),
                    p_60: Color::Srgba(Srgba::hex("A28D81").unwrap()),
                    p_70: Color::Srgba(Srgba::hex("BDA79A").unwrap()),
                    p_80: Color::Srgba(Srgba::hex("DAC2B5").unwrap()),
                    p_90: Color::Srgba(Srgba::hex("F7DED0").unwrap()),
                    p_95: Color::Srgba(Srgba::hex("FFEDE4").unwrap()),
                    p_98: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    p_99: Color::Srgba(Srgba::hex("FFFBFF").unwrap()),
                    p_100: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                },
                secondary: ColorPalette {
                    p_0: Color::Srgba(Srgba::hex("000000").unwrap()),
                    p_5: Color::Srgba(Srgba::hex("14100E").unwrap()),
                    p_10: Color::Srgba(Srgba::hex("1F1B18").unwrap()),
                    p_15: Color::Srgba(Srgba::hex("2A2522").unwrap()),
                    p_20: Color::Srgba(Srgba::hex("352F2C").unwrap()),
                    p_25: Color::Srgba(Srgba::hex("403A37").unwrap()),
                    p_30: Color::Srgba(Srgba::hex("4C4542").unwrap()),
                    p_35: Color::Srgba(Srgba::hex("58514E").unwrap()),
                    p_40: Color::Srgba(Srgba::hex("645D5A").unwrap()),
                    p_50: Color::Srgba(Srgba::hex("7D7672").unwrap()),
                    p_60: Color::Srgba(Srgba::hex("978F8B").unwrap()),
                    p_70: Color::Srgba(Srgba::hex("B2A9A5").unwrap()),
                    p_80: Color::Srgba(Srgba::hex("CEC5C0").unwrap()),
                    p_90: Color::Srgba(Srgba::hex("EBE0DC").unwrap()),
                    p_95: Color::Srgba(Srgba::hex("F9EFEA").unwrap()),
                    p_98: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    p_99: Color::Srgba(Srgba::hex("FFFBFF").unwrap()),
                    p_100: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                },
                tertiary: ColorPalette {
                    p_0: Color::Srgba(Srgba::hex("000000").unwrap()),
                    p_5: Color::Srgba(Srgba::hex("11110B").unwrap()),
                    p_10: Color::Srgba(Srgba::hex("1C1C16").unwrap()),
                    p_15: Color::Srgba(Srgba::hex("26261F").unwrap()),
                    p_20: Color::Srgba(Srgba::hex("31312A").unwrap()),
                    p_25: Color::Srgba(Srgba::hex("3C3C34").unwrap()),
                    p_30: Color::Srgba(Srgba::hex("48473F").unwrap()),
                    p_35: Color::Srgba(Srgba::hex("54534B").unwrap()),
                    p_40: Color::Srgba(Srgba::hex("605F56").unwrap()),
                    p_50: Color::Srgba(Srgba::hex("79776F").unwrap()),
                    p_60: Color::Srgba(Srgba::hex("939188").unwrap()),
                    p_70: Color::Srgba(Srgba::hex("AEABA2").unwrap()),
                    p_80: Color::Srgba(Srgba::hex("C9C6BC").unwrap()),
                    p_90: Color::Srgba(Srgba::hex("E6E2D8").unwrap()),
                    p_95: Color::Srgba(Srgba::hex("F4F1E6").unwrap()),
                    p_98: Color::Srgba(Srgba::hex("FDF9EE").unwrap()),
                    p_99: Color::Srgba(Srgba::hex("FFFBFF").unwrap()),
                    p_100: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                },
                neutral: ColorPalette {
                    p_0: Color::Srgba(Srgba::hex("000000").unwrap()),
                    p_5: Color::Srgba(Srgba::hex("121110").unwrap()),
                    p_10: Color::Srgba(Srgba::hex("1D1B1B").unwrap()),
                    p_15: Color::Srgba(Srgba::hex("272525").unwrap()),
                    p_20: Color::Srgba(Srgba::hex("32302F").unwrap()),
                    p_25: Color::Srgba(Srgba::hex("3D3B3A").unwrap()),
                    p_30: Color::Srgba(Srgba::hex("494645").unwrap()),
                    p_35: Color::Srgba(Srgba::hex("545251").unwrap()),
                    p_40: Color::Srgba(Srgba::hex("615E5D").unwrap()),
                    p_50: Color::Srgba(Srgba::hex("7A7675").unwrap()),
                    p_60: Color::Srgba(Srgba::hex("94908F").unwrap()),
                    p_70: Color::Srgba(Srgba::hex("AFAAA9").unwrap()),
                    p_80: Color::Srgba(Srgba::hex("CAC5C4").unwrap()),
                    p_90: Color::Srgba(Srgba::hex("E7E1E0").unwrap()),
                    p_95: Color::Srgba(Srgba::hex("F5F0EE").unwrap()),
                    p_98: Color::Srgba(Srgba::hex("FEF8F6").unwrap()),
                    p_99: Color::Srgba(Srgba::hex("FFFBFF").unwrap()),
                    p_100: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                },
                neutral_variant: ColorPalette {
                    p_0: Color::Srgba(Srgba::hex("000000").unwrap()),
                    p_5: Color::Srgba(Srgba::hex("13100F").unwrap()),
                    p_10: Color::Srgba(Srgba::hex("1E1B1A").unwrap()),
                    p_15: Color::Srgba(Srgba::hex("282524").unwrap()),
                    p_20: Color::Srgba(Srgba::hex("33302E").unwrap()),
                    p_25: Color::Srgba(Srgba::hex("3E3B39").unwrap()),
                    p_30: Color::Srgba(Srgba::hex("4A4644").unwrap()),
                    p_35: Color::Srgba(Srgba::hex("555250").unwrap()),
                    p_40: Color::Srgba(Srgba::hex("625D5C").unwrap()),
                    p_50: Color::Srgba(Srgba::hex("7B7674").unwrap()),
                    p_60: Color::Srgba(Srgba::hex("95908D").unwrap()),
                    p_70: Color::Srgba(Srgba::hex("B0AAA8").unwrap()),
                    p_80: Color::Srgba(Srgba::hex("CBC5C3").unwrap()),
                    p_90: Color::Srgba(Srgba::hex("E8E1DE").unwrap()),
                    p_95: Color::Srgba(Srgba::hex("F6EFED").unwrap()),
                    p_98: Color::Srgba(Srgba::hex("FFF8F5").unwrap()),
                    p_99: Color::Srgba(Srgba::hex("FFFBFF").unwrap()),
                    p_100: Color::Srgba(Srgba::hex("FFFFFF").unwrap()),
                },
            },
        }
    }
}

use std::str::FromStr;

use bevy::prelude::*;
use fluent_langneg::{LanguageIdentifier, LangugeIdentifierParserError};

//-------------------------------------------------------------------------------------------------------------------

/// The currently-requested locale controlling localization.
///
/// This is inserted by `LocalizationPlugin` with a default value [detected](Self::get_system_locale) from the
/// user's system (if possible). If no system locale is detected, the value will be empty, which means to use the
/// default values for all localized content.
///
/// If you want a custom locale, it is recommended to set this resource during app startup. You can further modify
/// the locale at runtime to relocalize everything in the app (e.g. when the user changes language settings).
#[derive(Resource, Clone, Debug, Default)]
pub struct Locale
{
    /// Requested languages in order of preference.
    ///
    /// It is recommended for most use-cases to only put one language here, because otherwise it is possible the
    /// user will see a confusing mix of multiple languages in their app if different requested
    /// languages are only partially translated.
    pub requested: Vec<LanguageIdentifier>,
}

impl Locale
{
    /// Makes a new `Locale` from a single language.
    ///
    /// Errors if the string fails to convert to a [`LanguageIdentifier`].
    pub fn new(locale: impl AsRef<str>) -> Result<Self, LangugeIdentifierParserError>
    {
        Ok(Self::new_from_id(LanguageIdentifier::from_str(locale.as_ref())?))
    }

    /// Makes a new `Locale` from a single language ID.
    pub fn new_from_id(locale: LanguageIdentifier) -> Self
    {
        Self { requested: vec![locale] }
    }

    /// Gets the user's system locale.
    ///
    /// This is a best-effort extraction of the system language. See the
    /// [`sys-locale`](https://crates.io/crates/sys-locale) crate for supported platforms.
    pub fn get_system_locale() -> Option<LanguageIdentifier>
    {
        sys_locale::get_locale()
            .map(|s| LanguageIdentifier::from_str(s.as_str()).ok())
            .flatten()
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LocalePlugin;

impl Plugin for LocalePlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(Locale { requested: Locale::get_system_locale().into_iter().collect() });
    }
}

//-------------------------------------------------------------------------------------------------------------------

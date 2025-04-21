use std::path::PathBuf;
use std::str::FromStr;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use fluent_langneg::{negotiate_languages, LanguageIdentifier, LangugeIdentifierParserError, NegotiationStrategy};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Deref, DerefMut)]
struct CachedRequestedLangs(Vec<LanguageIdentifier>);

//-------------------------------------------------------------------------------------------------------------------

/// This system contains similar logic to `update_negotiated_languages`, but we separate the two for more precise
/// logic flow during app initialization.
fn load_manifest(
    In((default_meta, alt_metas)): In<(LocalizationMeta, Vec<LocalizationMeta>)>,
    mut cached: ResMut<CachedRequestedLangs>,
    mut c: Commands,
    mut manifest: ResMut<LocalizationManifest>,
    locale: Res<Locale>,
)
{
    if **cached != locale.requested {
        **cached = locale.requested.clone();
    }

    manifest.reset(default_meta, alt_metas);
    manifest.negotiate(&locale.requested);
    tracing::info!("app languages set to {:?} from requested {:?}", manifest.negotiated(), locale.requested);
    c.react().broadcast(LocalizationManifestUpdated);
    c.react().broadcast(LanguagesNegotiated);
}

//-------------------------------------------------------------------------------------------------------------------

fn update_negotiated_languages(
    mut cached: ResMut<CachedRequestedLangs>,
    mut c: Commands,
    mut manifest: ResMut<LocalizationManifest>,
    locale: Res<Locale>,
)
{
    if **cached == locale.requested {
        tracing::trace!("ignoring modified Locale whose contents did not change");
        return;
    }
    **cached = locale.requested.clone();

    manifest.negotiate(&locale.requested);
    tracing::info!("app languages set to {:?} from requested {:?}", manifest.negotiated(), locale.requested);
    c.react().broadcast(LanguagesNegotiated);
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when [`LocalizationManifest`] has been updated with languages loaded from file.
pub struct LocalizationManifestUpdated;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when [`LocalizationManifest`] has negotiated a new languages list with [`Locale`].
pub struct LanguagesNegotiated;

//-------------------------------------------------------------------------------------------------------------------

/// Version of [`LocalizationMeta`] that can be reflected.
///
/// Used by [`LoadLocalizationManifest`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct LocalizationMetaReflected
{
    pub id: String,
    #[reflect(default)]
    pub name: Option<String>,
    pub manifest: PathBuf,
    #[reflect(default)]
    pub allow_as_fallback: bool,
}

//-------------------------------------------------------------------------------------------------------------------

/// Metadata that defines a language for use in localization.
///
/// Includes a manifest for the language's [`fluent`](https://projectfluent.org/)-based text localization.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct LocalizationMeta
{
    /// The canonical [`LanguageIdentifier`] for this language.
    pub id: LanguageIdentifier,
    /// The language name for this language.
    ///
    /// This value is optional, and only included as a convenience for generating language lists.
    //todo: Include font so this string can be written in the right language? Is it expensive to have many fonts
    // loaded? Maybe need a truncated multi-language font for writing language names?
    pub name: Option<String>,
    /// The `FtlBundle` file where this language's localization file names are collected.
    ///
    /// This file should have one of the following extensions: `"ftl.ron", "ftl.yaml", "ftl.yml"`. It should
    /// deserialize into a struct like this:
    /*
    ```rust
    struct FtlBundleData {
        locale: LanguageIdentifier,
        resources: Vec<PathBuf>,
    }
    ```
    */
    ///
    /// Each resource referenced by the manifest should be a file with the `.ftl` extension and contain a
    /// [`FluentResource`](fluent::FluentResource). See the [`fluent`](https://projectfluent.org/) docs.
    pub manifest: PathBuf,
    /// Option indicating this language can be used as a fallback if it matches a user's system language.
    ///
    /// Note that if all text and localizable assets in your app are not localized by this language, then the
    /// user may encounter up to three languages in their app (their requested language, their system language,
    /// and the global default language).
    ///
    /// This option does nothing for the `default` field in [`LoadLocalizationManifest`], which will be used as
    /// the global default fallback.
    pub allow_as_fallback: bool,
}

impl LocalizationMeta
{
    /// Gets the display name of this language.
    ///
    /// Falls back to `self.id` if `self.name` is `None`.
    pub fn display_name(&self) -> String
    {
        self.name.clone().unwrap_or_else(|| format!("{}", self.id))
    }
}

impl TryFrom<LocalizationMetaReflected> for LocalizationMeta
{
    type Error = LangugeIdentifierParserError;
    fn try_from(value: LocalizationMetaReflected) -> Result<Self, Self::Error>
    {
        Ok(Self {
            id: LanguageIdentifier::from_str(value.id.as_str())?,
            name: value.name,
            manifest: value.manifest,
            allow_as_fallback: value.allow_as_fallback,
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that tracks [`LocalizationMetas`](LocalizationMeta).
#[derive(Resource, Debug, Default)]
pub struct LocalizationManifest
{
    /// All languages that can potentially be used to localize text or assets in your app.
    ///
    /// The first language is the default.
    languages: Vec<LocalizationMeta>,
    /// The negotiated list of languages to use when localizing text or assets in your app.
    ///
    /// This is updated automatically when the manifest's contents change, or when the user modifies the
    /// [`Locale`] resource.
    negotiated: Vec<LanguageIdentifier>,
}

impl LocalizationManifest
{
    fn reset(&mut self, mut default: LocalizationMeta, alternates: Vec<LocalizationMeta>)
    {
        if self.languages.len() > 0 {
            // Note: this prints spuriously when the manifest is hot-reloaded.
            tracing::warn!("overwriting LocalizationManifest, only one instance of LoadLocalizationManifest may \
                be loaded");
        }

        default.allow_as_fallback = true;

        self.languages.clear();
        self.languages.push(default);
        self.languages.extend(alternates);
    }

    fn negotiate(&mut self, requested: &[LanguageIdentifier])
    {
        // Negotiate the user's 'system default'.
        // - This is the language that best matches the user's system language.
        // - We only look at languages that are allowed to be used as fallbacks, to avoid using partially-localized
        //   languages that would cause a lot of jank for the user.
        let allowed_fallbacks: Vec<&LanguageIdentifier> = self
            .languages()
            .iter()
            .filter_map(|meta| match meta.allow_as_fallback {
                true => Some(&meta.id),
                false => None,
            })
            .collect();
        let system_default = Locale::get_system_locale()
            .map(|system_locale| {
                negotiate_languages(&[system_locale], &allowed_fallbacks, None, NegotiationStrategy::Lookup)
                    .get(0)
                    .cloned()
            })
            .flatten();

        // Negotiate a language list between the user's requested languages and the available languages.
        let all_available: Vec<&LanguageIdentifier> = self.languages().iter().map(|meta| &meta.id).collect();
        let mut negotiated: Vec<LanguageIdentifier> = negotiate_languages(
            requested,
            &all_available,
            system_default,
            NegotiationStrategy::Filtering,
        )
        .iter()
        .map(|l| (**l).clone())
        .collect();

        // Remove languages after the first fallback.
        let first_fallback = negotiated
            .iter()
            .position(|n| {
                self.languages()
                    .iter()
                    .find(|l| l.id == *n)
                    .unwrap()
                    .allow_as_fallback
            })
            .unwrap_or(negotiated.len());
        negotiated.truncate(first_fallback + 1);

        // Append the global default if it doesn't exist in the language list.
        // - We add this as an emergency fallback to minimize the worst-case scenario of some text/asset failing to
        //   localize.
        if let Some(global_default) = self.get_default() {
            if !negotiated.contains(&global_default.id) {
                negotiated.push(global_default.id.clone());
            }
        }

        // Save the final negotiated list.
        self.negotiated = negotiated;
    }

    /// Accesses the full list of languages available for localization.
    ///
    /// The returned slice is ordered based on the order in the [`LoadLocalizationManifest`] that filled this
    /// resource.
    pub fn languages(&self) -> &[LocalizationMeta]
    {
        &self.languages
    }

    /// Accesses the list of languages that should be used to localize text and assets.
    ///
    /// The returned slice is ordered based on localization priority. The first language will ideally be used to
    /// localize everything in the app. All other languages returned are fallbacks.
    ///
    /// The list is built from the following steps (see [`fluent-langneg`][fluent-langneg] for more on
    /// negotiation):
    /// - Filter available languages (see [`Self::languages`]) against the user's requested languages (see
    ///   [`Locale`]). This gives a list of languages prioritized by how well they match the user's request.
    /// - In case the previous list's languages can't fully localize the app, try to add a fallback to the user's
    ///   system language. This entails filtering languages with [`LocalizationMeta::allow_as_fallback`] set to
    ///   `true` against the user's primary system language (which is detected automatically).
    /// - Trim the language list so there is at most one language with [`LocalizationMeta::allow_as_fallback`] set
    ///   to true. This minimizes the chance for jank caused by multiple fallback languages (which should generally
    ///   localize all text/assets) that don't completely overlap their localization coverage. It also reduces the
    ///   number of localization files held in memory.
    /// - Finally, we insert the global default (see [`LoadLocalizationManifest::default`]) at the end of the list
    ///   as a last-resort fallback.
    ///
    /// [fluent-langneg]: https://docs.rs/fluent-langneg/0.14.1/fluent_langneg/negotiate/index.html
    pub fn negotiated(&self) -> &[LanguageIdentifier]
    {
        &self.negotiated
    }

    /// See [`Self::negotiated`].
    pub fn iter_negotiated_metas(&self) -> impl Iterator<Item = &LocalizationMeta> + '_
    {
        self.negotiated.iter().map(|id| self.get(id).unwrap())
    }

    /// Gets a specific [`LocalizationMeta`].
    pub fn get(&self, id: &LanguageIdentifier) -> Option<&LocalizationMeta>
    {
        self.languages.iter().find(|m| m.id == *id)
    }

    /// Gets the default locale.
    ///
    /// This is used as a fallback for localizing text, and is considered the 'primary' language for all
    /// localizable assets (e.g. fonts, images, sounds, etc.).
    ///
    /// Returns `None` if the manifest has not loaded yet.
    pub fn get_default(&self) -> Option<&LocalizationMeta>
    {
        self.languages.get(0)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Command loadable for adding language folders to [`LocalizationManifest`].
///
/// Languages are inserted to the manifest in the order they appear in this loadable. You should organize the
/// manifest according to the order you want languages to appear in [`LocalizationManifest::languages`] (this is
/// useful for automatically generating localization options lists).
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct LoadLocalizationManifest
{
    /// The default language.
    ///
    /// This is used as a fallback for localizing text, and is considered the 'primary' language for all
    /// localizable assets (e.g. fonts, images, sounds, etc.).
    ///
    /// All localized text in your app should have definitions in this language's localization files.
    pub default: LocalizationMetaReflected,
    /// Alternate languages that may have partial or complete translations of text and localization of assets.
    #[reflect(default)]
    pub alts: Vec<LocalizationMetaReflected>,
}

impl Command for LoadLocalizationManifest
{
    fn apply(mut self, world: &mut World)
    {
        let default_meta = match LocalizationMeta::try_from(self.default) {
            Ok(default_meta) => default_meta,
            Err(err) => {
                tracing::error!("failed parsing LoadLocalizationManifest, default language has an invalid id {:?}",
                    err);
                return;
            }
        };

        let alt_metas: Vec<LocalizationMeta> = self
            .alts
            .drain(..)
            .map(LocalizationMeta::try_from)
            .filter_map(|maybe_meta| match maybe_meta {
                Ok(meta) => Some(meta),
                Err(err) => {
                    tracing::error!("ignoring alternate language in LoadLocalizationManifest with invalid language \
                        id {:?}", err);
                    None
                }
            })
            .collect();

        world.syscall((default_meta, alt_metas), load_manifest);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LocalizationManifestPlugin;

impl Plugin for LocalizationManifestPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<CachedRequestedLangs>()
            .init_resource::<LocalizationManifest>()
            .register_type::<LocalizationMetaReflected>()
            .register_type::<Vec<LocalizationMetaReflected>>()
            .register_type::<Option<String>>()
            .register_command_type::<LoadLocalizationManifest>()
            .add_systems(
                PostUpdate,
                update_negotiated_languages
                    .run_if(resource_changed::<Locale>)
                    .in_set(LocalizationSet::Negotiate)
                    .run_if(in_state(LoadState::Done)),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

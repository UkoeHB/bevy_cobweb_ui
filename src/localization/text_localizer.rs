use std::borrow::Borrow;

use bevy::asset::AssetLoadFailedEvent;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use fluent::bundle::FluentBundle;
use fluent::memoizer::MemoizerKind;
use fluent::{FluentArgs, FluentResource};
use fluent_content::Request;
use fluent_langneg::LanguageIdentifier;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Implementation copied from `fluent_content::Content`, but avoids allocating the target string.
fn set_content<'a, T, U, V, W>(
    bundle: &FluentBundle<V, W>,
    template: T,
    template_str: &str,
    target: &mut String,
) -> bool
where
    T: Into<Request<'a, U>>,
    U: Borrow<FluentArgs<'a>>,
    V: Borrow<FluentResource>,
    W: MemoizerKind,
{
    let request = template.into();
    let request = request.borrow();
    let Some(message) = bundle.get_message(request.id) else { return false };
    let pattern = match request.attr {
        Some(key) => {
            let Some(attribute) = message.get_attribute(key) else { return false };
            attribute.value()
        }
        None => {
            let Some(value) = message.value() else { return false };
            value
        }
    };
    let mut errors = Vec::new();
    target.clear();
    bundle
        .write_pattern(target, pattern, request.args.as_ref().map(Borrow::borrow), &mut errors)
        .expect("writing to string failed");
    for error in &errors {
        tracing::warn!("error while localizing template \"{template_str}\", {:?}", error);
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------

fn load_localization(
    mut c: Commands,
    asset_server: Res<AssetServer>,
    manifest: ReactRes<LocalizationManifest>,
    mut localizer: ResMut<TextLocalizer>,
)
{
    localizer.update_localizations(&manifest, &asset_server);

    // Assume the update modified the localizations list in some way, so we need to relocalize all text.
    if !localizer.is_loading() {
        c.react().broadcast(TextLocalizerUpdated);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_localization_data(
    mut c: Commands,
    mut errors: EventReader<AssetLoadFailedEvent<FtlBundle>>,
    mut events: EventReader<AssetEvent<FtlBundle>>,
    mut assets: ResMut<Assets<FtlBundle>>,
    mut localizer: ResMut<TextLocalizer>,
)
{
    let started_loading = localizer.is_loading();
    let mut fail_count = 0;
    let mut refresh_count = 0;

    // Handle errors.
    for error in errors.read() {
        let AssetLoadFailedEvent { id, .. } = error;
        if localizer.remove_failed_load(*id) {
            fail_count += 1;
        }
    }

    // Handle asset events.
    for event in events.read() {
        let id = match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => id,
            _ => {
                tracing::debug!("ignoring localization asset event {:?}", event);
                continue;
            }
        };

        let Some(ftl_bundle) = assets.remove(*id) else {
            tracing::error!("failed to remove localization asset {:?}", id);
            continue;
        };

        // Save the localization data.
        // - We do *not* add formatting fallbacks to the bundle because we don't want localized text sections to
        //   have multiple languages in them (e.g. localized text and a date/time formatted with a different
        //   language). This is because each font section can only have one font, and fonts aren't valid for all
        //   languages.
        if localizer.try_set(*id, ftl_bundle) {
            refresh_count += 1;
        }
    }

    // Update `is_loading` field.
    if fail_count > 0 || refresh_count > 0 {
        localizer.update_is_loading();
    }

    // Check if the text localizer updated and is fully loaded.
    let ended_loading = localizer.is_loading();
    if (started_loading || refresh_count > 0) && !ended_loading {
        c.react().broadcast(TextLocalizerUpdated);
    }
}

//-------------------------------------------------------------------------------------------------------------------

enum TextLocalization
{
    Loading
    {
        id: LanguageIdentifier, handle: Handle<FtlBundle>
    },
    Loaded
    {
        id: LanguageIdentifier,
        asset: FtlBundle,
        // We cache this in case a bundle is hot reloaded.
        handle: Handle<FtlBundle>,
    },
}

impl TextLocalization
{
    fn try_set(&mut self, id: &AssetId<FtlBundle>, asset: FtlBundle) -> Result<(), FtlBundle>
    {
        if self.handle().id() != *id {
            return Err(asset);
        }

        *self = Self::Loaded {
            id: self.lang_id().clone(),
            asset,
            handle: self.handle().clone(),
        };
        Ok(())
    }

    fn lang_id(&self) -> &LanguageIdentifier
    {
        match self {
            Self::Loading { id, .. } => id,
            Self::Loaded { id, .. } => id,
        }
    }

    fn handle(&self) -> &Handle<FtlBundle>
    {
        match self {
            Self::Loading { handle, .. } => handle,
            Self::Loaded { handle, .. } => handle,
        }
    }

    fn is_loading(&self) -> bool
    {
        matches!(*self, Self::Loading{..})
    }

    fn asset(&self) -> Option<(&LanguageIdentifier, &FtlBundle)>
    {
        let Self::Loaded { id, asset, .. } = self else { return None };
        Some((id, asset))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when [`TextLocalizer`] has been updated.
///
/// Fires *after* all localization data has been loaded.
pub struct TextLocalizerUpdated;

//-------------------------------------------------------------------------------------------------------------------

/// Tool for localizing text.
///
/// The current [`Self::language`] is set automatically using the [`LocalizationManifest`] and [`Locale`]
/// resources.
///
/// When this resource has been updated due to a [`LocalizationManifest`] or [`Locale`] change, the
/// [`TextLocalizerUpdated`] reactive event will be broadcasted.
#[derive(Resource)]
pub struct TextLocalizer
{
    is_loading: bool,
    localizations: Vec<TextLocalization>,
}

impl TextLocalizer
{
    /// Returns `true` if any language data is currently loading.
    pub fn is_loading(&self) -> bool
    {
        self.is_loading
    }

    /// Localizes a string containing a localization template.
    ///
    /// Returns the language ID of the language used to set the string, or `None` if localization failed.
    ///
    /// Always returns `None` if `self.is_loading()` is true.
    pub fn localize(&self, template: &str, target: &mut String) -> Option<&LanguageIdentifier>
    {
        if self.is_loading() {
            return None;
        }

        self.localizations
            .iter()
            .filter_map(TextLocalization::asset)
            .find_map(|(lang, bundle)| {
                if set_content(&*bundle, template, template, target) {
                    Some(lang)
                } else {
                    None
                }
            })
    }

    fn update_localizations(&mut self, manifest: &LocalizationManifest, asset_server: &AssetServer)
    {
        let mut new_localizations = Vec::with_capacity(manifest.negotiated().len());

        // Build new localizations list while stealing existing languages from the previous list.
        for negotiated in manifest.negotiated_metas() {
            let next = match self
                .localizations
                .iter()
                .position(|l| *l.lang_id() == negotiated.id)
            {
                Some(idx) => {
                    let removed = self.localizations.swap_remove(idx);

                    #[cfg(feature = "hot_reload")]
                    {
                        // When hot reloading, the language's manifest location may be stale.
                        let path: std::path::PathBuf = removed.handle().path().unwrap().clone().into();
                        if negotiated.manifest != path {
                            TextLocalization::Loading {
                                id: negotiated.id.clone(),
                                handle: asset_server.load(negotiated.manifest.clone()),
                            }
                        } else {
                            removed
                        }
                    }
                    #[cfg(not(feature = "hot_reload"))]
                    {
                        removed
                    }
                }
                None => TextLocalization::Loading {
                    id: negotiated.id.clone(),
                    handle: asset_server.load(negotiated.manifest.clone()),
                },
            };
            new_localizations.push(next);
        }

        self.localizations = new_localizations;

        // Cache the loading state to reduce lookups when localizing text.
        self.update_is_loading();
    }

    fn update_is_loading(&mut self)
    {
        self.is_loading = self.localizations.iter().any(|l| l.is_loading());
    }

    fn remove_failed_load(&mut self, id: AssetId<FtlBundle>) -> bool
    {
        let mut removed_id = false;

        self.localizations.retain(|l| {
            if l.handle().id() == id {
                tracing::warn!("failed loading localization data for {:?}", l.lang_id());
                removed_id = true;
                return false;
            }
            true
        });

        removed_id
    }

    fn try_set(&mut self, id: AssetId<FtlBundle>, mut asset: FtlBundle) -> bool
    {
        for localization in self.localizations.iter_mut() {
            match localization.try_set(&id, asset) {
                Ok(()) => return true,
                Err(returned_asset) => {
                    asset = returned_asset;
                }
            }
        }

        // This shouldn't print unless the user is very rapidly changing language settings.
        tracing::warn!("ignoring stale localization bundle {:?}", asset.locale().to_string());
        false
    }
}

impl Default for TextLocalizer
{
    fn default() -> Self
    {
        Self { is_loading: false, localizations: Vec::default() }
    }
}

impl AssetLoadProgress for TextLocalizer
{
    fn pending_assets(&self) -> usize
    {
        self.localizations.iter().filter(|l| l.is_loading()).count()
    }

    fn total_assets(&self) -> usize
    {
        self.localizations.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct TextLocalizerPlugin;

impl Plugin for TextLocalizerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<TextLocalizer>()
            .register_asset_tracker::<TextLocalizer>()
            .react(|rc| rc.on_persistent(resource_mutation::<LocalizationManifest>(), load_localization))
            .add_systems(First, get_localization_data.after(FileProcessingSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------

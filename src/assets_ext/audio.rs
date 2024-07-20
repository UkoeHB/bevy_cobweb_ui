use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use bevy::asset::{AssetLoadFailedEvent, AssetPath};
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use fluent_langneg::{negotiate_languages, LanguageIdentifier, NegotiationStrategy};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_audio_sources(
    In(loaded): In<Vec<LoadedAudio>>,
    mut c: Commands,
    asset_server: Res<AssetServer>,
    mut audios: ResMut<AudioMap>,
    manifest: Res<LocalizationManifest>,
)
{
    audios.insert_loaded(loaded, &asset_server, &manifest, &mut c);
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_lang_list(
    asset_server: Res<AssetServer>,
    manifest: Res<LocalizationManifest>,
    mut audios: ResMut<AudioMap>,
)
{
    audios.negotiate_languages(&manifest, &asset_server);
}

//-------------------------------------------------------------------------------------------------------------------

fn check_loaded_audios(
    mut c: Commands,
    mut errors: EventReader<AssetLoadFailedEvent<AudioSource>>,
    mut events: EventReader<AssetEvent<AudioSource>>,
    mut audios: ResMut<AudioMap>,
)
{
    for error in errors.read() {
        let AssetLoadFailedEvent { id, .. } = error;
        audios.remove_pending(id);
    }

    for event in events.read() {
        let AssetEvent::Added { id } = event else { continue };
        audios.remove_pending(id);
    }

    audios.try_emit_load_event(&mut c);
}

//-------------------------------------------------------------------------------------------------------------------

/// System that runs when the app needs to replace existing audio sources with updated localized audio sources.
fn relocalize_audios(audios: Res<AudioMap>, mut query: Query<&mut Handle<AudioSource>>)
{
    for mut handle in query.iter_mut() {
        audios.localize_audio(&mut handle);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when [`AudioMap`] has updated and become fully loaded *after* a [`LoadAudio`]
/// instance was applied.
///
/// This event is *not* emitted when audio sources are reloaded due to language renegotiation. Listen for the
/// [`RelocalizeApp`] event instead.
pub struct AudioMapLoaded;

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to loaded audio files and manages audio localization.
///
/// Requested audio handles will be automatically localized based on the currently negotiated languages in
/// [`LocalizationManifest`]. If negotiated languages change, then all audio handles in the app will be
/// automatically re-localized if they have fallbacks for the new language list.
///
/// We assume that all localization fallbacks are globally unique. A fallback should be used as a fallback exactly
/// once and never used as a 'main' audio.
///
/// Audio sources are automatically loaded and unloaded when languages are changed, so that only sources that might
/// be needed are kept in memory.
#[derive(Resource, Default)]
pub struct AudioMap
{
    /// Indicates the current pending audio sources came from `LoadedAudio` entries, rather than from negotiating
    /// languages.
    ///
    /// This is used to emit `AudioMapLoaded` events accurately.
    waiting_for_load: bool,
    /// Audio sources currently loading.
    pending: HashSet<AssetId<AudioSource>>,
    /// Localization fallbacks.
    /// - Strings in this map are 'full asset paths' that can be used to load audio sources.
    /// [ main audio path : (main audio path, [ lang id, fallback audio path ]) ]
    localization_map: HashMap<Arc<str>, (AssetPath<'static>, HashMap<LanguageIdentifier, AssetPath<'static>>)>,
    /// Used when replacing audio sources on language change. Includes main audio AssetPaths in case newly-loaded
    /// audio mappings introduce a new localization so existing main audio handles need to be replaced.
    /// [ audio path : main audio path ]
    localized_audios_id_helper: HashMap<AssetPath<'static>, Arc<str>>,
    /// Contains handles for audio sources that should be displayed for each 'main audio path' based on
    /// currently negotiated languages.
    /// [ main audio path : audio handle ]
    localized_audios: HashMap<Arc<str>, Handle<AudioSource>>,
    /// Audio sources stored permanently.
    cached_audios: HashMap<Arc<str>, Handle<AudioSource>>,
}

impl AudioMap
{
    /// Checks if the map has any audio sources waiting to load.
    pub fn is_loading(&self) -> bool
    {
        !self.pending.is_empty()
    }

    fn try_add_pending(
        handle: &Handle<AudioSource>,
        asset_server: &AssetServer,
        pending: &mut HashSet<AssetId<AudioSource>>,
    )
    {
        match asset_server.load_state(handle) {
            bevy::asset::LoadState::Loaded => (),
            _ => {
                pending.insert(handle.id());
            }
        }
    }

    fn try_emit_load_event(&mut self, c: &mut Commands)
    {
        if self.is_loading() {
            return;
        }
        if !self.waiting_for_load {
            return;
        }

        self.waiting_for_load = false;
        c.react().broadcast(AudioMapLoaded);
    }

    fn negotiate_languages(&mut self, manifest: &LocalizationManifest, asset_server: &AssetServer)
    {
        // We remove `localized_audios` because we assume it might be stale (e.g. if we are negotiating because
        // LoadAudio was hot-reloaded).
        let prev_localized_audios = std::mem::take(&mut self.localized_audios);
        self.localized_audios.reserve(self.localization_map.len());

        let app_negotiated = manifest.negotiated();
        let mut langs_buffer = Vec::default();

        self.localization_map
            .iter()
            .for_each(|(main_path, (main_asset_path, fallbacks))| {
                // Collect fallback langs for this audio.
                langs_buffer.clear();
                langs_buffer.extend(fallbacks.keys());

                // Negotiate the language we should use, then look up its asset path.
                // - Note: `negotiated_languages` may allocate multiple times, but we don't think this is a huge
                //   issue since it's unlikely users will localize a *lot* of audio sources. It *could* be an issue
                //   if a user loads audio sources from many LoadAudio commands, causing this loop to run many
                //   times.
                let asset_path =
                    negotiate_languages(&langs_buffer, app_negotiated, None, NegotiationStrategy::Lookup)
                        .get(0)
                        .map(|lang| {
                            fallbacks
                                .get(lang)
                                .expect("negotiation should only return fallback langs")
                        })
                        .unwrap_or(main_asset_path);

                // Look up or load the handle currently associated with the main audio.
                // - If we found a the handle but it doesn't match the language we want, then load the audio fresh.
                let handle = prev_localized_audios
                    .get(main_path)
                    .or_else(|| self.cached_audios.get(main_path))
                    .filter(|handle| {
                        // Filter based on if the handle has a path that equals the target path.
                        handle.path().filter(|path| *path == asset_path).is_some()
                    })
                    .cloned()
                    .unwrap_or_else(|| {
                        let handle = asset_server.load(asset_path.clone());
                        Self::try_add_pending(&handle, asset_server, &mut self.pending);
                        handle
                    });

                // Now save the localized audio.
                self.localized_audios.insert(main_path.clone(), handle);
            });

        // Note: old audio sources that are no longer needed will be released when `prev_localized_audios` is
        // dropped.
    }

    fn remove_pending(&mut self, id: &AssetId<AudioSource>)
    {
        let _ = self.pending.remove(id);
    }

    /// Adds an audio that should be cached.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the audio to be loaded.
    pub fn insert(&mut self, path: impl AsRef<str>, asset_server: &AssetServer)
    {
        let path = path.as_ref();

        // Check if the audio is cached already.
        if self.cached_audios.contains_key(path) {
            tracing::warn!("ignoring duplicate insert for audio {}", path);
            return;
        }

        // Check if the audio is a localized audio.
        let asset_path = match AssetPath::try_parse(path) {
            Ok(asset_path) => asset_path,
            Err(err) => {
                tracing::error!("failed parsing audio path {:?} on insert to AudioMap: {:?}", path, err);
                return;
            }
        };
        if let Some((key, handle)) = self
            .localized_audios
            .get_key_value(path)
            .filter(|(_, handle)| {
                *handle
                    .path()
                    .expect("handles in localized_audios should have paths")
                    == asset_path
            })
        {
            self.cached_audios.insert(key.clone(), handle.clone());
            return;
        }

        // Add a new cached audio.
        let handle = asset_server.load(asset_path);
        Self::try_add_pending(&handle, asset_server, &mut self.pending);
        self.cached_audios.insert(Arc::from(path), handle);
    }

    /// Adds a new set of [`LoadedAudios`](`LoadedAudio`).
    ///
    /// Will automatically renegotiate languages and emit [`AudioMapLoaded`] if appropriate.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for new audio sources to be loaded.
    pub fn insert_loaded(
        &mut self,
        mut loaded: Vec<LoadedAudio>,
        asset_server: &AssetServer,
        manifest: &LocalizationManifest,
        c: &mut Commands,
    )
    {
        for mut loaded in loaded.drain(..) {
            let main_path = Arc::<str>::from(loaded.audio.as_str());

            let (main_asset_path, fallbacks) = self
                .localization_map
                .entry(main_path.clone())
                .or_insert_with(|| {
                    let main_asset_path = match AssetPath::try_parse(&main_path) {
                        Ok(asset_path) => asset_path.clone_owned(),
                        Err(err) => {
                            tracing::error!("failed parsing audio path {:?} on insert loaded to AudioMap: {:?}",
                                main_path, err);
                            AssetPath::<'static>::default()
                        }
                    };
                    (main_asset_path, HashMap::default())
                });

            // Add helper entry for main audio.
            self.localized_audios_id_helper
                .insert(main_asset_path.clone(), main_path.clone());

            // Save fallbacks.
            #[cfg(not(feature = "hot_reload"))]
            if fallbacks.len() > 0 {
                // This is feature-gated by hot_reload to avoid spam when hot reloading large lists.
                tracing::warn!("overwritting audio fallbacks for main audio {:?}; main audio sources should only appear \
                    in one LoadAudio command per app", main_path);
            }

            fallbacks.clear();
            fallbacks.reserve(loaded.fallbacks.len());

            for LoadedAudioFallback { lang, audio } in loaded.fallbacks.drain(..) {
                // Save fallback.
                let lang_id = match LanguageIdentifier::from_str(lang.as_str()) {
                    Ok(lang_id) => lang_id,
                    Err(err) => {
                        tracing::error!("failed parsing target language id  {:?} for audio fallback {:?} for audio {:?}: \
                            {:?}", lang, audio, main_path, err);
                        continue;
                    }
                };
                let fallback_asset_path = match AssetPath::try_parse(audio.as_str()) {
                    Ok(asset_path) => asset_path.clone_owned(),
                    Err(err) => {
                        tracing::error!("failed parsing fallback audio path {:?} for {:?} on insert loaded to \
                            AudioMap: {:?}", audio, main_path, err);
                        continue;
                    }
                };

                if let Some(prev) = fallbacks.insert(lang_id, fallback_asset_path.clone()) {
                    tracing::warn!("overwriting audio fallback {:?} for audio {:?} for lang {:?}",
                        prev, main_path, lang);
                }

                // Save fallback to helper.
                self.localized_audios_id_helper
                    .insert(fallback_asset_path, main_path.clone());
            }

            // Note: we populate `localized_audios` in `Self::negotiate_languages`.
        }

        // Load audio sources as needed.
        self.waiting_for_load = true;
        self.negotiate_languages(manifest, asset_server);
        self.try_emit_load_event(c);
    }

    /// Updates an audio handle with the correct localized handle.
    ///
    /// Does nothing if the handle is already correctly localized or if there are no localization fallbacks
    /// associated with the audio.
    pub fn localize_audio(&self, handle: &mut Handle<AudioSource>)
    {
        let Some(path) = handle.path().cloned() else {
            tracing::debug!("failed localizing audio handle that doesn't have a path");
            return;
        };

        if let Some(localized_handle) = self
            .localized_audios_id_helper
            .get(&path)
            .and_then(|main_path| self.localized_audios.get(main_path))
        {
            *handle = localized_handle.clone();
        } else {
            tracing::debug!("failed localizing audio handle with {:?} that doesn't have a localization entry", path);
        }
    }

    /// Gets an audio handle for the given path.
    ///
    /// If the given path has a localization fallback for the current [`LocalizationManifest::negotiated`]
    /// languages, then the handle for that fallback will be returned.
    ///
    /// Returns a default handle if the audio was not pre-inserted via [`Self::insert`] or [`Self::insert_loaded`].
    pub fn get(&self, path: impl AsRef<str>) -> Handle<AudioSource>
    {
        let path = path.as_ref();

        self.localized_audios
            .get(path)
            .or_else(|| self.cached_audios.get(path))
            .cloned()
            .unwrap_or_else(|| {
                tracing::error!("failed getting audio {} that was not loaded to AudioMap", path);
                Default::default()
            })
    }

    /// Gets an audio handle for the given path, or loads and caches the audio if it's unknown.
    ///
    /// If the given path has a localization fallback for the current [`LocalizationManifest::negotiated`]
    /// languages, then the handle for that fallback will be returned.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the audio to be loaded.
    pub fn get_or_insert(&mut self, path: impl AsRef<str>, asset_server: &AssetServer) -> Handle<AudioSource>
    {
        let path = path.as_ref();

        // Looks up the audio, otherwise loads it fresh.
        self.localized_audios
            .get(path)
            .or_else(|| self.cached_audios.get(path))
            .cloned()
            .unwrap_or_else(|| {
                let handle = asset_server.load(String::from(path));
                Self::try_add_pending(&handle, asset_server, &mut self.pending);
                self.cached_audios.insert(Arc::from(path), handle.clone());
                handle
            })
    }
}

impl AssetLoadProgress for AudioMap
{
    fn pending_assets(&self) -> usize
    {
        self.pending.len()
    }

    fn total_assets(&self) -> usize
    {
        // This may double-count some audio sources.
        self.localized_audios.len() + self.cached_audios.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Contains information for a audio fallback.
///
/// See [`LoadedAudio`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedAudioFallback
{
    /// The language id for the fallback.
    pub lang: String,
    /// The path to the audio asset.
    pub audio: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`LoadAudio`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadedAudio
{
    /// Path to the audio asset.
    pub audio: String,
    /// Fallback audio sources for specific languages.
    ///
    /// Add fallbacks if `self.audio` cannot be used for all languages. Any reference to `self.audio` will be
    /// automatically localized to the right fallback if you use [`AudioMap::get`].
    #[reflect(default)]
    pub fallbacks: Vec<LoadedAudioFallback>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering audio assets that need to be pre-loaded.
///
/// The loaded audio sources can be accessed via [`AudioMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadAudio(pub Vec<LoadedAudio>);

impl Command for LoadAudio
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_audio_sources);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct AudioLoadPlugin;

impl Plugin for AudioLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<AudioMap>()
            .register_asset_tracker::<AudioMap>()
            .register_command::<LoadAudio>()
            .react(|rc| rc.on_persistent(broadcast::<LanguagesNegotiated>(), handle_new_lang_list))
            .react(|rc| {
                rc.on_persistent(
                    (broadcast::<AudioMapLoaded>(), broadcast::<RelocalizeApp>()),
                    relocalize_audios,
                )
            })
            .add_systems(PreUpdate, check_loaded_audios.in_set(LoadProgressSet::Prepare));
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use bevy::asset::AssetLoadFailedEvent;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use fluent_langneg::LanguageIdentifier;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn load_localized_fonts(
    In(loaded): In<Vec<LocalizedFont>>,
    mut c: Commands,
    asset_server: Res<AssetServer>,
    mut fonts: ResMut<FontMap>,
    manifest: Res<LocalizationManifest>,
)
{
    fonts.insert_localized(loaded, &asset_server, &manifest, &mut c);
}

//-------------------------------------------------------------------------------------------------------------------

fn load_fonts(In(loaded): In<Vec<String>>, asset_server: Res<AssetServer>, mut fonts: ResMut<FontMap>)
{
    for path in loaded {
        fonts.insert(&path, &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_lang_list(
    asset_server: Res<AssetServer>,
    manifest: Res<LocalizationManifest>,
    mut fonts: ResMut<FontMap>,
)
{
    fonts.negotiate_languages(&manifest, &asset_server);
}

//-------------------------------------------------------------------------------------------------------------------

fn check_loaded_fonts(
    mut c: Commands,
    mut events: EventReader<AssetEvent<Font>>,
    mut errors: EventReader<AssetLoadFailedEvent<Font>>,
    mut fonts: ResMut<FontMap>,
)
{
    for event in events.read() {
        let AssetEvent::Added { id } = event else { continue };
        fonts.remove_pending(id);
    }

    for error in errors.read() {
        let AssetLoadFailedEvent { id, .. } = error;
        fonts.remove_pending(id);
    }

    fonts.try_emit_load_event(&mut c);
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when [`FontMap`] has updated and become fully loaded *after* a [`LoadFonts`]
/// instance was applied.
///
/// This event is *not* emitted when fonts are reloaded due to language renegotiation. Listen for the
/// [`RelocalizeApp`] event instead.
pub struct FontMapLoaded;

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to loaded fonts and manages font localization.
///
/// Localization font fallbacks are supported. If you use [`LocalizedText`] then fallbacks will be applied
/// automatically. You only need to insert 'main fonts' to entity text via [`Self::get`] (when initially
/// constructing text sections) or [`TextEditor::set_font`] (when editing existing text).
///
/// Fonts are automatically loaded and unloaded when languages are changed, so that only fonts that might be
/// needed are kept in memory.
#[derive(Resource, Default)]
pub struct FontMap
{
    /// Indicates the current pending fonts came from `LoadedFont` entries, rather than from negotiating
    /// languages.
    ///
    /// This is used to emit `FontMapLoaded` events accurately.
    waiting_for_load: bool,
    /// All font assets that are currently loaded.
    pending: HashSet<AssetId<Font>>,
    /// Fonts that are permanently cached, including main fonts.
    ///
    /// We keep all main fonts loaded because to spawn a new text entity, the user needs to get a
    /// `Handle<Font>` for their main font. Font localization happens separate from the 'insert Text component'
    /// step because when inserting a Text component you don't always know what language each text section
    /// requires.
    ///
    /// When localization *does* occur for a newly-inserted Text component, we use existing font handles on
    /// the text sections to look up font fallbacks.
    ///
    /// This can contain stale 'main fonts' if you hot reload `LoadFonts` and the reloaded version removes some
    /// main fonts. It should not affect users in any way.
    // [ font path : font handle ]
    cached_fonts: HashMap<String, Handle<Font>>,
    /// Map between main fonts and language-specific fallbacks.
    ///
    /// This is updated whenever a new font list is inserted.
    // [ main font : [ lang : fallback font ] ]
    localization_map: HashMap<AssetId<Font>, HashMap<LanguageIdentifier, String>>,
    /// Map between font paths and font handles for all currently-active localization fallbacks.
    ///
    /// This is reconstructed whenever languages are renegotiated.
    // [ font path : font handle ]
    localization_fonts: HashMap<String, Handle<Font>>,
}

impl FontMap
{
    /// Checks if the map has any fonts waiting to load.
    pub fn is_loading(&self) -> bool
    {
        !self.pending.is_empty()
    }

    fn try_add_pending(handle: &Handle<Font>, asset_server: &AssetServer, pending: &mut HashSet<AssetId<Font>>)
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
        c.react().broadcast(FontMapLoaded);
    }

    /// Returns `false` if no localized assets were loaded.
    fn negotiate_languages(&mut self, manifest: &LocalizationManifest, asset_server: &AssetServer) -> bool
    {
        // Skip negotiation of there are no negotiated languages yet.
        // - This avoids spuriously loading assets that will be replaced once the language list is known.
        let negotiated = manifest.negotiated();
        if negotiated.len() == 0 {
            return false;
        }

        let prev_localization_fonts = std::mem::take(&mut self.localization_fonts);
        self.localization_fonts
            .reserve(prev_localization_fonts.len());

        self.localization_map.iter().for_each(|(_main, fallbacks)| {
            // Save fallbacks for currently-negotiated languages.
            for (_lang, font) in fallbacks
                .iter()
                .filter(|(lang, _)| negotiated.iter().any(|n| n == *lang))
            {
                let handle = prev_localization_fonts
                    .get(font)
                    .or_else(|| self.cached_fonts.get(font))
                    .cloned()
                    .unwrap_or_else(|| {
                        let new_handle = asset_server.load(font);
                        Self::try_add_pending(&new_handle, asset_server, &mut self.pending);
                        new_handle
                    });
                self.localization_fonts.insert(font.clone(), handle);
            }
        });

        true
    }

    fn remove_pending(&mut self, id: &AssetId<Font>) -> bool
    {
        self.pending.remove(id)
    }

    /// Adds a font that should be loaded.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the font to be loaded.
    pub fn insert(&mut self, path: impl AsRef<str>, asset_server: &AssetServer)
    {
        self.get_or_insert(path, asset_server);
    }

    /// Adds a new set of [`LoadedFonts`](`LoadedFont`).
    ///
    /// Will automatically renegotiate languages and emit [`FontMapLoaded`] if appropriate.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for new fonts to be loaded.
    pub fn insert_localized(
        &mut self,
        mut loaded: Vec<LocalizedFont>,
        asset_server: &AssetServer,
        manifest: &LocalizationManifest,
        c: &mut Commands,
    )
    {
        for mut loaded in loaded.drain(..) {
            // Add main font.
            let main_handle = self
                .cached_fonts
                .get(&loaded.font)
                .cloned()
                .or_else(|| {
                    self.localization_fonts
                        .get(&loaded.font)
                        .inspect(|handle| {
                            // Main fonts need to be cached.
                            self.cached_fonts
                                .insert(loaded.font.clone(), (*handle).clone());
                        })
                        .cloned()
                })
                .unwrap_or_else(|| {
                    let new_handle = asset_server.load(&loaded.font);
                    self.cached_fonts
                        .insert(String::from(loaded.font.as_str()), new_handle.clone());
                    Self::try_add_pending(&new_handle, asset_server, &mut self.pending);
                    new_handle
                });

            // Add fallbacks.
            let fallbacks = self.localization_map.entry(main_handle.id()).or_default();

            #[cfg(not(feature = "hot_reload"))]
            if fallbacks.len() > 0 {
                // This is feature-gated by hot_reload to avoid spam when hot reloading large lists.
                tracing::warn!("overwritting font fallbacks for main font {:?}; main fonts should only appear in one \
                    LoadFonts command per app", loaded.font);
            }

            fallbacks.clear();
            fallbacks.reserve(loaded.fallbacks.len());

            for LocalizedFontFallback { lang, font } in loaded.fallbacks.drain(..) {
                let lang_id = match LanguageIdentifier::from_str(lang.as_str()) {
                    Ok(lang_id) => lang_id,
                    Err(err) => {
                        tracing::error!("failed parsing target language id for font fallback {:?} for main font \
                            {:?}: {:?}", font, loaded.font, err);
                        continue;
                    }
                };

                if let Some(prev) = fallbacks.insert(lang_id, font) {
                    tracing::warn!("overwriting font fallback {:?} for font {:?} for lang {:?}",
                        prev, loaded.font, lang);
                }
            }
        }

        // Load fallback fonts as needed.
        if self.negotiate_languages(manifest, asset_server) {
            self.waiting_for_load = true;
            self.try_emit_load_event(c);
        }
    }

    /// Gets a font handle for the given path.
    ///
    /// The returned handle will *not* be localized. Use [`Self::get_localized`] or
    /// [`Self::get_or_load_localized`] instead.
    ///
    /// Returns a default handle if the font was not pre-inserted via [`Self::insert`].
    pub fn get(&self, path: impl AsRef<str>) -> Handle<Font>
    {
        // Look in cached map only.
        // - We assume localization fonts are 'invisible' to the user.
        let Some(entry) = self.cached_fonts.get(path.as_ref()) else {
            tracing::error!("failed getting font {} that was not loaded; use LoadFonts command or \
                FontMap::insert", path.as_ref());
            return Default::default();
        };
        entry.clone()
    }

    /// Gets a font handle for the given path, or loads and caches the font if it's unknown.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the font to be loaded.
    pub fn get_or_insert(&mut self, path: impl AsRef<str>, asset_server: &AssetServer) -> Handle<Font>
    {
        let path = path.as_ref();
        self.cached_fonts
            .get(path)
            .cloned()
            .or_else(|| {
                self.localization_fonts
                    .get(path)
                    .inspect(|handle| {
                        // Cache the font because `get_or_insert` access implies the font should be permanently
                        // stored.
                        self.cached_fonts
                            .insert(String::from(path), (*handle).clone());
                    })
                    .cloned()
            })
            .unwrap_or_else(|| {
                let new_handle = asset_server.load(String::from(path));
                self.cached_fonts
                    .insert(String::from(path), new_handle.clone());
                Self::try_add_pending(&new_handle, asset_server, &mut self.pending);
                new_handle
            })
    }

    /// Gets the font localized to `lang_id` for the given `main_font`.
    ///
    /// Returns `None` if there is no font fallback for `lang_id`.
    pub fn get_localized(&self, lang_id: &LanguageIdentifier, main_font: AssetId<Font>) -> Option<Handle<Font>>
    {
        self.localization_map
            .get(&main_font)
            .and_then(|fallbacks| fallbacks.get(lang_id))
            .and_then(|lang_font| {
                self.localization_fonts.get(lang_font).or_else(|| {
                    tracing::error!("font fallback {:?} is missing from loaded fonts, the requested language {:?} \
                        is probably not in the negotiated languages list of LocalizationManifest", lang_font, lang_id);
                    None
                })
            })
            .cloned()
    }

    /// Gets the font localized to `lang_id` for the given `main_font`.
    ///
    /// Will cache the font if it's not already cached, and will load it if it's not loaded.
    ///
    /// This can be used if you need to localize the font of text that should be displayed in another
    /// language (e.g. usernames).
    ///
    /// Will return the requested main font if no language-specific fallback is found.
    pub fn get_or_load_localized(
        &mut self,
        lang_id: &LanguageIdentifier,
        main_font: impl AsRef<str>,
        asset_server: &AssetServer,
    ) -> Handle<Font>
    {
        let main_font = main_font.as_ref();
        let mut to_cache: Option<(String, Handle<Font>)> = None;
        let req_handle = self.cached_fonts
            .get(main_font)
            .map(|main| {
                // Look up the fallback and clone it into the `cached_fonts` map.
                // - If the fallback exists but isn't loaded, then load it.
                // - If there is no fallback, just use the `main` font.
                self.localization_map
                    .get(&main.id())
                    .and_then(|fallbacks| fallbacks.get(lang_id))
                    .map(|lang_font| {
                        self.cached_fonts.get(lang_font)
                            .or_else(|| {
                                self.localization_fonts.get(lang_font).inspect(|handle| {
                                    to_cache = Some((lang_font.clone(), (*handle).clone()));
                                })
                            })
                            .cloned()
                            .unwrap_or_else(|| {
                                let new_handle = asset_server.load(lang_font.clone());
                                to_cache = Some((lang_font.clone(), new_handle.clone()));
                                Self::try_add_pending(&new_handle, asset_server, &mut self.pending);
                                new_handle
                            })
                    })
                    .unwrap_or_else(|| {
                        tracing::debug!("failed get-or-load-localized font, requested main font does not have a fallback \
                            for language {:?}; returning main font {:?} instead", lang_id, main_font);
                        main.clone()
                    })
            })
            .unwrap_or_else(|| {
                tracing::debug!("failed get-or-load-localized font, requested main font {:?} is not registered, loading it \
                    directy instead for language {:?}", main_font, lang_id);

                let new_handle = asset_server.load(String::from(main_font));
                to_cache = Some((String::from(main_font), new_handle.clone()));
                Self::try_add_pending(&new_handle, asset_server, &mut self.pending);
                new_handle
            });

        if let Some((lang, handle)) = to_cache {
            self.cached_fonts.insert(lang, handle);
        }

        req_handle
    }
}

impl AssetLoadProgress for FontMap
{
    fn pending_assets(&self) -> usize
    {
        self.pending.len()
    }

    // This may not be totally accurate if localization fonts and cached fonts overlap.
    fn total_assets(&self) -> usize
    {
        self.localization_fonts.len() + self.cached_fonts.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Contains information for a font fallback.
///
/// See [`LoadedFont`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedFontFallback
{
    /// The language id for the fallback.
    pub lang: String,
    /// The path to the font asset.
    pub font: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`LoadFonts`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedFont
{
    /// Path to the font asset.
    pub font: String,
    /// Fallback fonts for specific languages.
    ///
    /// Add fallbacks if `self.font` cannot be used for all languages. Any reference to `self.font` will be
    /// automatically localized to the right fallback on entities with [`LocalizedText`]. Note that
    /// [`LocalizedText`] keeps track of the main font (`self.font`) for every text section in case the user
    /// changes languages and it is necessary to switch to a different fallback.
    #[reflect(default)]
    pub fallbacks: Vec<LocalizedFontFallback>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering localized font assets that need to be pre-loaded.
///
/// The loaded fonts can be accessed via [`FontMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadLocalizedFonts(pub Vec<LocalizedFont>);

impl Command for LoadLocalizedFonts
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_localized_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering font assets that need to be pre-loaded.
///
/// The loaded fonts can be accessed via [`FontMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadFonts(pub Vec<String>);

impl Command for LoadFonts
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct FontLoadPlugin;

impl Plugin for FontLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<FontMap>()
            .register_asset_tracker::<FontMap>()
            .register_command::<LoadFonts>()
            .register_command::<LoadLocalizedFonts>()
            .react(|rc| rc.on_persistent(broadcast::<LanguagesNegotiated>(), handle_new_lang_list))
            .add_systems(PreUpdate, check_loaded_fonts.in_set(LoadProgressSet::Prepare));
    }
}

//-------------------------------------------------------------------------------------------------------------------

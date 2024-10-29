use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use bevy::asset::AssetLoadFailedEvent;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use fluent_langneg::LanguageIdentifier;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

struct SelectedLocalizedFontFallback
{
    lang: SmolStr,
    font: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// Gets font variants of a specific font family.
///
/// If the attributes list is empty, then all variants are returned. Otherwise only variants equal to requested
/// attributes are returned.
fn get_loaded_font_variants<'a>(
    families: &'a HashMap<SmolStr, Vec<FontVariant>>,
    family: &'a SmolStr,
    attrs: &'a [FontAttributes],
) -> impl Iterator<Item = &'a FontVariant> + 'a
{
    families
        .get(family)
        .or_else(|| {
            tracing::error!("ignoring loaded font ({family:?}, {attrs:?}) with unknown family; this is a bug");
            None
        })
        .into_iter()
        .flat_map(|v| v.iter())
        .filter_map(|v| {
            if attrs.len() > 0 {
                let v_attrs = v.attributes();
                if !attrs.iter().any(|attrs| *attrs == v_attrs) {
                    return None;
                }
            }
            Some(v)
        })
}

//-------------------------------------------------------------------------------------------------------------------

/// Gets localized font fallbacks for a given font variant from the main font.
fn get_dependent_font_variants(
    families: &HashMap<SmolStr, Vec<FontVariant>>,
    reference_font_family: &SmolStr,
    reference_font: FontAttributes,
    loaded_fallbacks: &Vec<LocalizedFontFallback>,
) -> Vec<SelectedLocalizedFontFallback>
{
    let mut fallbacks = Vec::<SelectedLocalizedFontFallback>::default();
    for LocalizedFontFallback { lang, family, attributes } in loaded_fallbacks {
        let Some(variants) = families.get(family) else {
            tracing::error!("failed getting fallback fonts for font family {reference_font_family:?} for language \
                {lang:?}; fallback font family {family:?} is unknown (fallback attrs: {attributes:?}); \
                this is a bug");
            continue;
        };

        let attributes = if attributes.len() == 0 {
            negotiate_eligible_fonts(reference_font, || variants.iter().map(|v| v.attributes()))
        } else {
            negotiate_eligible_fonts(reference_font, || attributes.iter().cloned())
        };

        let Some(attributes) = attributes else {
            tracing::warn!("failed getting fallback fonts for font family {reference_font_family:?} with \
                {reference_font:?} for language {lang:?}; unable to select eligible fallback font for fallback \
                font family {family:?} (fallback attrs: {attributes:?}, registered variants: {variants:?})");
            continue;
        };

        let Some(variant) = variants.iter().find(|v| v.attributes() == attributes) else {
            tracing::warn!("failed getting fallback fonts for font family {reference_font_family:?} with \
                {reference_font:?} for language {lang:?}; selected font attributes don't match any registered \
                font variants for fallback font family {family:?} (selected: {attributes:?}, fallback attrs: \
                {attributes:?}, registered variants: {variants:?})");
            continue;
        };

        fallbacks.push(SelectedLocalizedFontFallback { lang: lang.clone(), font: variant.path.clone() })
    }

    fallbacks
}

//-------------------------------------------------------------------------------------------------------------------

/// See https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm
fn negotiate_eligible_fonts<I>(attrs: FontAttributes, attrs_fn: impl Fn() -> I) -> Option<FontAttributes>
where
    I: Iterator<Item = FontAttributes>,
{
    // Identify the best-fitting font width.
    let width = FontWidth::negotiate(attrs.width, || (attrs_fn)().map(|a| a.width))?;

    // Identify the best-fitting font style using the identified width.
    let style = FontStyle::negotiate(attrs.style, || {
        (attrs_fn)().filter(|a| a.width == width).map(|a| a.style)
    })?;

    // Identify the best-fitting font weight using the identified width and style.
    let weight = FontWeight::negotiate(attrs.weight, || {
        (attrs_fn)()
            .filter(|a| (a.width == width) && (a.style == style))
            .map(|a| a.weight)
    })?;

    Some(FontAttributes { width, style, weight })
}

//-------------------------------------------------------------------------------------------------------------------

fn get_eligible_font<'a>(
    families: &'a HashMap<SmolStr, Vec<FontVariant>>,
    font: &FontRequest,
) -> Option<&'a FontVariant>
{
    let variants = families.get(&**font.family)?;
    let attributes = negotiate_eligible_fonts(font.attributes(), || variants.iter().map(|v| v.attributes()))?;

    // Get the variant that matches what we found.
    let Some(variant) = variants.iter().find(|v| v.attributes() == attributes) else {
        tracing::error!("failed negotiating eligible fonts for request {font:?} and variants {variants:?} \
            even though a variant was presumably found ({attributes:?}); this is a bug");
        return None;
    };
    Some(variant)
}

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

fn load_fonts(In(families): In<Vec<SmolStr>>, asset_server: Res<AssetServer>, mut fonts: ResMut<FontMap>)
{
    for family in families {
        fonts.load(&FontFamily(family), &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn register_font_families(
    In(registrations): In<Vec<RegisterFontFamily>>,
    mut c: Commands,
    asset_server: Res<AssetServer>,
    mut fonts: ResMut<FontMap>,
    manifest: Res<LocalizationManifest>,
)
{
    for registration in registrations {
        fonts.register(registration, &asset_server, &manifest, &mut c);
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
/// Fonts must be pre-registered with [`RegisterFontFamilies`]. Registered fonts are *not* automatically loaded.
/// Use [`LoadFonts`], [`LoadLocalizedFonts`], or [`FontMap::get_or_load`] to trigger asset loads.
///
/// Localization font fallbacks are supported. If you use the [`LocalizedText`] component then fallbacks will be
/// applied automatically. You only need to insert 'main fonts' to entity text via [`Self::get`] (when initially
/// constructing text sections) or [`TextEditor::set_font`] (when editing existing text).
///
/// Fonts are automatically loaded and unloaded when languages are changed, so that only fonts that might be
/// needed are kept in memory. Note that the text backend may cache a copy of all fonts indefinitely. This map
/// only manages Bevy font assets.
#[derive(Resource, Default)]
pub struct FontMap
{
    /// Indicates the current pending fonts came from `LoadFonts`/`LoadLocalizedFonts` entries, rather than from
    /// negotiating languages.
    ///
    /// This is used to emit `FontMapLoaded` events accurately.
    waiting_for_load: bool,
    /// All font assets that are currently loaded.
    pending: HashSet<AssetId<Font>>,

    /// Loaded fonts waiting for font families to be registered.
    loaded_awaiting_families: Vec<FontFamily>,
    localized_awaiting_families: Vec<LocalizedFont>,

    /// Registered font families.
    families: HashMap<SmolStr, Vec<FontVariant>>,

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
    /// [ font path : font handle ]
    cached_fonts: HashMap<String, Handle<Font>>,
    /// Map between main fonts and language-specific fallbacks.
    ///
    /// This is updated whenever a new font list is inserted.
    /// [ main font : [ lang : fallback font ] ]
    localization_map: HashMap<AssetId<Font>, HashMap<LanguageIdentifier, String>>,
    /// Map between font paths and font handles for all currently-active localization fallbacks.
    ///
    /// This is reconstructed whenever languages are renegotiated.
    /// [ font path : font handle ]
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
                        let new_handle = asset_server.load(String::from(font.as_str()));
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

    /// Checks and warns if font requests were unresolved on entering `LoadState::Done`.
    fn check_unresolved_font_requests(map: Res<FontMap>)
    {
        for loaded in &map.loaded_awaiting_families {
            tracing::warn!("failed loading requested font {loaded:?} before LoadState::Done; use RegisterFontFamilies");
        }
        for loaded in &map.localized_awaiting_families {
            tracing::warn!("failed loading localized font {loaded:?} or one of its fallbacks before LoadState::Done; \
                use RegisterFontFamilies");
        }
    }

    /// Registers a font family so its members can be loaded on request.
    pub fn register(
        &mut self,
        family: RegisterFontFamily,
        asset_server: &AssetServer,
        manifest: &LocalizationManifest,
        c: &mut Commands,
    )
    {
        if family.fonts.len() == 0 {
            tracing::warn!("ignoring font registration {family:?} with no variants");
            return;
        }

        if let Some(prev) = self
            .families
            .insert(family.family.deref().clone(), family.fonts)
        {
            tracing::warn!("overwriting font family {:?}; old font variants: {prev:?}", family.family);
        }

        // Insert loaded requests that can be completed now.
        //todo: use extract_if when stabilized
        let mut to_insert = Vec::<FontFamily>::default();
        self.loaded_awaiting_families.retain(|l| {
            if family.family == *l {
                to_insert.push(l.clone());
                false;
            }
            true
        });
        to_insert.drain(..).for_each(|l| {
            self.load(&l, asset_server);
        });

        // Insert loaded localized requests that can be completed now.
        //todo: use extract_if when stabilized
        let mut to_insert = Vec::<LocalizedFont>::default();
        self.localized_awaiting_families.retain(|l| {
            if self.families.contains_key(&l.family)
                && !l
                    .fallbacks
                    .iter()
                    .any(|fallback| !self.families.contains_key(&fallback.family))
            {
                to_insert.push(l.clone());
                false;
            }
            true
        });
        self.insert_localized(to_insert, asset_server, manifest, c);
    }

    /// Adds a font family that should be loaded.
    ///
    /// Returns `false` if there are no eligible fonts. See [`RegisterFontFamilies`]. The requested font
    /// will be cached in case it was loaded out-of-order with `RegisterFontFamilies` insertion.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the font to be loaded.
    pub fn load(&mut self, family: &FontFamily, asset_server: &AssetServer) -> bool
    {
        // Save font requests that can't be loaded yet.
        let Some(variants) = self.families.get(&**family) else {
            self.loaded_awaiting_families.push(family.clone());
            return false;
        };

        variants.iter().for_each(|v| {
            Self::get_or_load_impl(
                &mut self.pending,
                &mut self.cached_fonts,
                &self.localization_fonts,
                v,
                asset_server,
            );
        });

        true
    }

    /// Adds a new set of [`LocalizedFonts`](`LocalizedFont`).
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
        // Extract font requests that can't be loaded yet.
        //todo: use extract_if when stabilized
        let mut loaded2 = Vec::with_capacity(loaded.len());
        for font in loaded.drain(..) {
            if !self.families.contains_key(&font.family)
                || font
                    .fallbacks
                    .iter()
                    .any(|fallback| !self.families.contains_key(&fallback.family))
            {
                self.localized_awaiting_families.push(font);
            } else {
                loaded2.push(font);
            }
        }

        // Load the requested fonts.
        for (font_variant, mut loaded_fallbacks) in loaded2.iter().flat_map(|loaded| {
            // A `LocalizedFont` is a set of fonts that should be localized to other sets of fonts
            // per-language. We first get the set of fonts to localize, then for each of those fonts,
            // we get the font that each language should fall back to.
            get_loaded_font_variants(&self.families, &loaded.family, &loaded.attributes).map(|font_variant| {
                let fallbacks = get_dependent_font_variants(
                    &self.families,
                    &loaded.family,
                    font_variant.attributes(),
                    &loaded.fallbacks,
                );
                (font_variant, fallbacks)
            })
        }) {
            // Add main font.
            let loaded_font = &font_variant.path;
            let main_handle = self
                .cached_fonts
                .get(loaded_font)
                .cloned()
                .or_else(|| {
                    self.localization_fonts
                        .get(loaded_font)
                        .inspect(|handle| {
                            // Main fonts need to be cached.
                            self.cached_fonts
                                .insert(loaded_font.clone(), (*handle).clone());
                        })
                        .cloned()
                })
                .unwrap_or_else(|| {
                    let new_handle = asset_server.load(String::from(loaded_font.as_str()));
                    self.cached_fonts
                        .insert(loaded_font.clone(), new_handle.clone());
                    Self::try_add_pending(&new_handle, asset_server, &mut self.pending);
                    new_handle
                });

            // Add fallbacks.
            let fallbacks = self.localization_map.entry(main_handle.id()).or_default();

            #[cfg(not(feature = "hot_reload"))]
            if fallbacks.len() > 0 {
                // This is feature-gated by hot_reload to avoid spam when hot reloading large lists.
                tracing::warn!("overwritting font fallbacks for main font {:?}; main fonts should only appear in one \
                    LoadFonts command per app", loaded_font);
            }

            fallbacks.clear();
            fallbacks.reserve(loaded_fallbacks.len());

            for SelectedLocalizedFontFallback { lang, font } in loaded_fallbacks.drain(..) {
                let lang_id = match LanguageIdentifier::from_str(lang.as_str()) {
                    Ok(lang_id) => lang_id,
                    Err(err) => {
                        tracing::error!("failed parsing target language id for font fallback {:?} for main font \
                            {:?}: {:?}", font, loaded_font, err);
                        continue;
                    }
                };

                if let Some(prev) = fallbacks.insert(lang_id, font) {
                    tracing::warn!("overwriting font fallback {:?} for font {:?} for lang {:?}",
                        prev, loaded_font, lang);
                }
            }
        }

        // Load fallback fonts as needed.
        if self.negotiate_languages(manifest, asset_server) {
            self.waiting_for_load = true;
            self.try_emit_load_event(c);
        }
    }

    /// Gets the nearest eligible font using CSS font eligibility rules.
    ///
    /// Returns `None` if the requested font's family was not registered. See [`RegisterFontFamilies`].
    pub fn get_eligible_font(&self, font: &FontRequest) -> Option<&FontVariant>
    {
        get_eligible_font(&self.families, font)
    }

    /// Gets a font handle for the requested font.
    ///
    /// The returned handle will *not* be localized. Use [`Self::get_localized`] or
    /// [`Self::get_or_load_localized`] instead.
    ///
    /// Returns a default handle if the font was not successfully pre-inserted via [`Self::load`].
    pub fn get(&self, font: &FontRequest) -> Handle<Font>
    {
        // Get the best eligible font.
        let Some(eligible_font) = self.get_eligible_font(font) else {
            tracing::error!("failed font request {:?} that has no eligible fonts; use RegisterFontFamilies command", font);
            return Default::default();
        };

        // Look in cached map only.
        // - We assume localization fonts are 'invisible' to the user.
        let Some(entry) = self.cached_fonts.get(&eligible_font.path) else {
            tracing::error!("failed getting font {:?} that was not loaded; use LoadFonts command or \
                FontMap::insert", font);
            return Default::default();
        };
        entry.clone()
    }

    /// Gets a font handle for the requested font, or loads and caches the font if it's unloaded.
    ///
    /// Returns a default handle if there are no eligible fonts. See [`RegisterFontFamilies`].
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the font to be loaded.
    pub fn get_or_load(&mut self, font: &FontRequest, asset_server: &AssetServer) -> Handle<Font>
    {
        // Get the best eligible font.
        let Some(eligible_font) = get_eligible_font(&self.families, font) else {
            tracing::error!("failed font request {:?} that has no eligible fonts; use RegisterFontFamilies command", font);
            return Default::default();
        };

        Self::get_or_load_impl(
            &mut self.pending,
            &mut self.cached_fonts,
            &self.localization_fonts,
            eligible_font,
            asset_server,
        )
    }

    fn get_or_load_impl(
        pending: &mut HashSet<AssetId<Font>>,
        cached_fonts: &mut HashMap<String, Handle<Font>>,
        localization_fonts: &HashMap<String, Handle<Font>>,
        eligible_font: &FontVariant,
        asset_server: &AssetServer,
    ) -> Handle<Font>
    {
        let path = &eligible_font.path;
        cached_fonts
            .get(path)
            .cloned()
            .or_else(|| {
                localization_fonts
                    .get(path)
                    .inspect(|handle| {
                        // Cache the font because `get_or_load` access implies the font should be permanently
                        // stored.
                        cached_fonts.insert(path.clone(), (*handle).clone());
                    })
                    .cloned()
            })
            .unwrap_or_else(|| {
                let new_handle = asset_server.load(String::from(path.as_str()));
                cached_fonts.insert(path.clone(), new_handle.clone());
                Self::try_add_pending(&new_handle, asset_server, pending);
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
        main_font: &FontRequest,
        asset_server: &AssetServer,
    ) -> Handle<Font>
    {
        // Get the best eligible font.
        let Some(eligible_font) = get_eligible_font(&self.families, main_font) else {
            tracing::error!("failed get-or-loaded-localized font request {:?} that has no eligible fonts; use \
                RegisterFontFamilies command", main_font);
            return Default::default();
        };

        let main_font = &eligible_font.path;
        let mut to_cache: Option<(&String, Handle<Font>)> = None;
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
                                    to_cache = Some((lang_font, (*handle).clone()));
                                })
                            })
                            .cloned()
                            .unwrap_or_else(|| {
                                let new_handle = asset_server.load(String::from(lang_font.as_str()));
                                to_cache = Some((lang_font, new_handle.clone()));
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

                let new_handle = asset_server.load(String::from(main_font.as_str()));
                to_cache = Some((main_font, new_handle.clone()));
                Self::try_add_pending(&new_handle, asset_server, &mut self.pending);
                new_handle
            });

        if let Some((path, handle)) = to_cache {
            self.cached_fonts.insert(path.clone(), handle);
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
/// See [`LocalizedFont`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedFontFallback
{
    /// The language id for the fallback.
    pub lang: SmolStr,
    /// The font family for the fallback.
    pub family: SmolStr,
    /// The [`FontAttributes`] allowed for the fallback.
    ///
    /// A font variant will be selected from the attributes set that best matches the source font(s). If no
    /// attributes are specified, then the best variant from the font family will be selected.
    #[reflect(default)]
    pub attributes: Vec<FontAttributes>,
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`LoadLocalizedFonts`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedFont
{
    /// Family of the font to localize.
    pub family: SmolStr,
    /// A set of [`FontAttributes`] to target for the fallback.
    ///
    /// If no attributes are specified, then all members of this font family will use the fallbacks specified
    /// here. Note that each 'main font' variant can only be mapped to localization fallbacks *once*. Multiple
    /// mappings will override each other.
    #[reflect(default)]
    pub attributes: Vec<FontAttributes>,
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

/// A member of a font family, with path for loading the asset.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontVariant
{
    /// Path to the font asset.
    pub path: String,
    #[reflect(default)]
    pub width: FontWidth,
    #[reflect(default)]
    pub style: FontStyle,
    #[reflect(default)]
    pub weight: FontWeight,
}

impl FontVariant
{
    /// Gets a [`FontAttributes`] from the variant.
    pub fn attributes(&self) -> FontAttributes
    {
        FontAttributes { width: self.width, style: self.style, weight: self.weight }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A font family with all its font variants.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterFontFamily
{
    pub family: FontFamily,
    pub fonts: Vec<FontVariant>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering localized fonts.
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

/// Loadable command for registering font families that need to be pre-loaded.
///
/// The loaded fonts can be accessed via [`FontMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadFonts(pub Vec<SmolStr>);

impl Command for LoadFonts
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_fonts);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering font family asset maps.
///
/// The font families can be accessed via [`FontMap`].
///
/// Font assets added here are *not* automatically loaded. Use [`LoadFonts`], [`LoadLocalizedFonts`], or
/// [`FontMap::get_or_load`] to actually load the fonts you want to use.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterFontFamilies(pub Vec<RegisterFontFamily>);

impl Command for RegisterFontFamilies
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, register_font_families);
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
            .register_command_type::<LoadFonts>()
            .register_command_type::<LoadLocalizedFonts>()
            .register_command_type::<RegisterFontFamilies>()
            .add_reactor(broadcast::<LanguagesNegotiated>(), handle_new_lang_list)
            .add_systems(OnEnter(LoadState::Done), FontMap::check_unresolved_font_requests)
            .add_systems(PreUpdate, check_loaded_fonts.in_set(LoadProgressSet::Prepare));
    }
}

//-------------------------------------------------------------------------------------------------------------------

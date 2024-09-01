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

fn load_localized_images(
    In(loaded): In<Vec<LocalizedImage>>,
    mut c: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<ImageMap>,
    manifest: Res<LocalizationManifest>,
)
{
    images.insert_localized(loaded, &asset_server, &manifest, &mut c);
}

//-------------------------------------------------------------------------------------------------------------------

fn load_images(In(loaded): In<Vec<String>>, asset_server: Res<AssetServer>, mut images: ResMut<ImageMap>)
{
    for path in loaded {
        images.insert(&path, &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_lang_list(
    asset_server: Res<AssetServer>,
    manifest: Res<LocalizationManifest>,
    mut images: ResMut<ImageMap>,
)
{
    images.negotiate_languages(&manifest, &asset_server);
}

//-------------------------------------------------------------------------------------------------------------------

fn check_loaded_images(
    mut c: Commands,
    mut errors: EventReader<AssetLoadFailedEvent<Image>>,
    mut events: EventReader<AssetEvent<Image>>,
    mut images: ResMut<ImageMap>,
)
{
    for error in errors.read() {
        let AssetLoadFailedEvent { id, .. } = error;
        images.remove_pending(id);
    }

    for event in events.read() {
        let AssetEvent::Added { id } = event else { continue };
        images.remove_pending(id);
    }

    images.try_emit_load_event(&mut c);
}

//-------------------------------------------------------------------------------------------------------------------

/// System that runs when the app needs to replace existing images with updated localized images.
fn relocalize_images(
    images: Res<ImageMap>,
    mut raw_imgs: Query<&mut Handle<Image>>,
    mut ui_imgs: Query<&mut UiImage>,
)
{
    for mut handle in raw_imgs.iter_mut().map(|img| img.into_inner()).chain(
        ui_imgs
            .iter_mut()
            .map(|ui_img| &mut ui_img.into_inner().texture),
    ) {
        images.localize_image(&mut handle);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when [`ImageMap`] has updated and become fully loaded *after* a
/// [`LoadLocalizedImages`] instance was applied.
///
/// This event is *not* emitted when images are reloaded due to language renegotiation. Listen for the
/// [`RelocalizeApp`] event instead.
pub struct ImageMapLoaded;

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to loaded image textures and manages image localization.
///
/// Requested image handles will be automatically localized based on the currently negotiated languages in
/// [`LocalizationManifest`]. If negotiated languages change, then all image handles in the app will be
/// automatically re-localized if they have fallbacks for the new language list.
///
/// We assume that all localization fallbacks are globally unique. A fallback should be used as a fallback exactly
/// once and never used as a 'main' image.
///
/// Images are automatically loaded and unloaded when languages are changed, so that only images that might be
/// needed are kept in memory.
#[derive(Resource, Default)]
pub struct ImageMap
{
    /// Indicates the current pending images came from `LoadImages` and `LoadLocalizedImages` entries, rather than
    /// from negotiating languages.
    ///
    /// This is used to emit `ImageMapLoaded` events accurately.
    waiting_for_load: bool,
    /// Images currently loading.
    pending: HashSet<AssetId<Image>>,
    /// Localization fallbacks.
    /// - Strings in this map are 'full asset paths' that can be used to load images.
    /// [ main image path : (main image path, [ lang id, fallback image path ]) ]
    localization_map: HashMap<Arc<str>, (AssetPath<'static>, HashMap<LanguageIdentifier, AssetPath<'static>>)>,
    /// Used when replacing images on language change. Includes main image AssetPaths in case newly-loaded
    /// image mappings introduce a new localization so existing main image handles need to be replaced.
    /// [ image path : main image path ]
    localized_images_id_helper: HashMap<AssetPath<'static>, Arc<str>>,
    /// Contains handles for images that should be displayed for each 'main image path' based on
    /// currently negotiated languages.
    /// [ main image path : image handle ]
    localized_images: HashMap<Arc<str>, Handle<Image>>,
    /// Images stored permanently.
    cached_images: HashMap<Arc<str>, Handle<Image>>,
}

impl ImageMap
{
    /// Checks if the map has any images waiting to load.
    pub fn is_loading(&self) -> bool
    {
        !self.pending.is_empty()
    }

    fn try_add_pending(handle: &Handle<Image>, asset_server: &AssetServer, pending: &mut HashSet<AssetId<Image>>)
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
        // Note/todo: This waits for both localized and non-localized images to load, even though the loaded
        // event is used for localization.
        if self.is_loading() {
            return;
        }
        if !self.waiting_for_load {
            return;
        }

        self.waiting_for_load = false;
        c.react().broadcast(ImageMapLoaded);
    }

    fn negotiate_languages(&mut self, manifest: &LocalizationManifest, asset_server: &AssetServer) -> bool
    {
        // Skip negotiation of there are no negotiated languages yet.
        // - This avoids spuriously loading assets that will be replaced once the language list is known.
        let app_negotiated = manifest.negotiated();
        if app_negotiated.len() == 0 {
            return false;
        }

        // We remove `localized_images` because we assume it might be stale (e.g. if we are negotiating because
        // LoadImages was hot-reloaded).
        let prev_localized_images = std::mem::take(&mut self.localized_images);
        self.localized_images.reserve(self.localization_map.len());

        let mut langs_buffer = Vec::default();

        self.localization_map
            .iter()
            .for_each(|(main_path, (main_asset_path, fallbacks))| {
                // Collect fallback langs for this image.
                langs_buffer.clear();
                langs_buffer.extend(fallbacks.keys());

                // Negotiate the language we should use, then look up its asset path.
                // - Note: `negotiated_languages` may allocate multiple times, but we don't think this is a huge
                //   issue since it's unlikely users will localize a *lot* of images. It *could* be an issue if a
                //   user loads images from many LoadImages commands, causing this loop to run many times.
                let asset_path =
                    negotiate_languages(&langs_buffer, app_negotiated, None, NegotiationStrategy::Lookup)
                        .get(0)
                        .map(|lang| {
                            fallbacks
                                .get(lang)
                                .expect("negotiation should only return fallback langs")
                        })
                        .unwrap_or(main_asset_path);

                // Look up or load the handle currently associated with the main image.
                // - If we found a the handle but it doesn't match the language we want, then load the image fresh.
                let handle = prev_localized_images
                    .get(main_path)
                    .or_else(|| self.cached_images.get(main_path))
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

                // Now save the localized image.
                self.localized_images.insert(main_path.clone(), handle);
            });

        // Note: old images that are no longer needed will be released when `prev_localized_images` is dropped.

        true
    }

    fn remove_pending(&mut self, id: &AssetId<Image>)
    {
        let _ = self.pending.remove(id);
    }

    /// Adds an image that should be cached.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the image to be loaded.
    pub fn insert(&mut self, path: impl AsRef<str>, asset_server: &AssetServer)
    {
        let path = path.as_ref();

        // Check if the image is cached already.
        if self.cached_images.contains_key(path) {
            tracing::warn!("ignoring duplicate insert for image {}", path);
            return;
        }

        // Check if the image is a localized image.
        let asset_path = match AssetPath::try_parse(path) {
            Ok(asset_path) => asset_path,
            Err(err) => {
                tracing::error!("failed parsing image path {:?} on insert to ImageMap: {:?}", path, err);
                return;
            }
        };
        if let Some((key, handle)) = self
            .localized_images
            .get_key_value(path)
            .filter(|(_, handle)| {
                *handle
                    .path()
                    .expect("handles in localized_images should have paths")
                    == asset_path
            })
        {
            self.cached_images.insert(key.clone(), handle.clone());
            return;
        }

        // Add a new cached image.
        let handle = asset_server.load(asset_path);
        Self::try_add_pending(&handle, asset_server, &mut self.pending);
        self.cached_images.insert(Arc::from(path), handle);
    }

    /// Adds a new set of [`LocalizedImages`](`LocalizedImage`).
    ///
    /// Will automatically renegotiate languages and emit [`ImageMapLoaded`] if appropriate.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for new images to be loaded.
    pub fn insert_localized(
        &mut self,
        mut loaded: Vec<LocalizedImage>,
        asset_server: &AssetServer,
        manifest: &LocalizationManifest,
        c: &mut Commands,
    )
    {
        for mut loaded in loaded.drain(..) {
            let main_path = Arc::<str>::from(loaded.image.as_str());

            let (main_asset_path, fallbacks) = self
                .localization_map
                .entry(main_path.clone())
                .or_insert_with(|| {
                    let main_asset_path = match AssetPath::try_parse(&main_path) {
                        Ok(asset_path) => asset_path.clone_owned(),
                        Err(err) => {
                            tracing::error!("failed parsing image path {:?} on insert loaded to ImageMap: {:?}",
                                main_path, err);
                            AssetPath::<'static>::default()
                        }
                    };
                    (main_asset_path, HashMap::default())
                });

            // Add helper entry for main image.
            self.localized_images_id_helper
                .insert(main_asset_path.clone(), main_path.clone());

            // Save fallbacks.
            #[cfg(not(feature = "hot_reload"))]
            if fallbacks.len() > 0 {
                // This is feature-gated by hot_reload to avoid spam when hot reloading large lists.
                tracing::warn!("overwritting image fallbacks for main image {:?}; main images should only appear in one \
                    LoadImages command per app", main_path);
            }

            fallbacks.clear();
            fallbacks.reserve(loaded.fallbacks.len());

            for LocalizedImageFallback { lang, image } in loaded.fallbacks.drain(..) {
                // Save fallback.
                let lang_id = match LanguageIdentifier::from_str(lang.as_str()) {
                    Ok(lang_id) => lang_id,
                    Err(err) => {
                        tracing::error!("failed parsing target language id  {:?} for image fallback {:?} for image {:?}: \
                            {:?}", lang, image, main_path, err);
                        continue;
                    }
                };
                let fallback_asset_path = match AssetPath::try_parse(image.as_str()) {
                    Ok(asset_path) => asset_path.clone_owned(),
                    Err(err) => {
                        tracing::error!("failed parsing fallback image path {:?} for {:?} on insert loaded to \
                            ImageMap: {:?}", image, main_path, err);
                        continue;
                    }
                };

                if let Some(prev) = fallbacks.insert(lang_id, fallback_asset_path.clone()) {
                    tracing::warn!("overwriting image fallback {:?} for image {:?} for lang {:?}",
                        prev, main_path, lang);
                }

                // Save fallback to helper.
                self.localized_images_id_helper
                    .insert(fallback_asset_path, main_path.clone());
            }

            // Note: we populate `localized_images` in `Self::negotiate_languages`.
        }

        // Load images as needed.
        if self.negotiate_languages(manifest, asset_server) {
            self.waiting_for_load = true;
            self.try_emit_load_event(c);
        }
    }

    /// Updates an image handle with the correct localized handle.
    ///
    /// Does nothing if the handle is already correctly localized or if there are no localization fallbacks
    /// associated with the image.
    pub fn localize_image(&self, handle: &mut Handle<Image>)
    {
        let Some(path) = handle.path().cloned() else {
            tracing::debug!("failed localizing image handle that doesn't have a path");
            return;
        };

        if let Some(localized_handle) = self
            .localized_images_id_helper
            .get(&path)
            .and_then(|main_path| self.localized_images.get(main_path))
        {
            *handle = localized_handle.clone();
        } else {
            tracing::debug!("failed localizing image handle with {:?} that doesn't have a localization entry", path);
        }
    }

    /// Gets an image handle for the given path.
    ///
    /// If the given path has a localization fallback for the current [`LocalizationManifest::negotiated`]
    /// languages, then the handle for that fallback will be returned.
    ///
    /// Returns a default handle if the image was not pre-inserted via [`Self::insert`] or
    /// [`Self::insert_localized`].
    pub fn get(&self, path: impl AsRef<str>) -> Handle<Image>
    {
        let path = path.as_ref();

        self.localized_images
            .get(path)
            .or_else(|| self.cached_images.get(path))
            .cloned()
            .unwrap_or_else(|| {
                tracing::error!("failed getting image {} that was not loaded to ImageMap", path);
                Default::default()
            })
    }

    /// Gets an image handle for the given path, or loads and caches the image if it's unknown.
    ///
    /// If the given path has a localization fallback for the current [`LocalizationManifest::negotiated`]
    /// languages, then the handle for that fallback will be returned.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the image to be loaded.
    pub fn get_or_load(&mut self, path: impl AsRef<str>, asset_server: &AssetServer) -> Handle<Image>
    {
        let path = path.as_ref();

        // Looks up the image, otherwise loads it fresh.
        self.localized_images
            .get(path)
            .or_else(|| self.cached_images.get(path))
            .cloned()
            .unwrap_or_else(|| {
                let handle = asset_server.load(String::from(path));
                Self::try_add_pending(&handle, asset_server, &mut self.pending);
                self.cached_images.insert(Arc::from(path), handle.clone());
                handle
            })
    }
}

impl AssetLoadProgress for ImageMap
{
    fn pending_assets(&self) -> usize
    {
        self.pending.len()
    }

    fn total_assets(&self) -> usize
    {
        // This may double-count some images.
        self.localized_images.len() + self.cached_images.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Contains information for an image fallback.
///
/// See [`LocalizedImage`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedImageFallback
{
    /// The language id for the fallback.
    pub lang: String,
    /// The path to the image asset.
    pub image: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`LoadImages`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedImage
{
    /// Path to the image asset.
    pub image: String,
    /// Fallback images for specific languages.
    ///
    /// Add fallbacks if `self.image` cannot be used for all languages. Any reference to `self.image` will be
    /// automatically localized to the right fallback if you use [`ImageMap::get`].
    #[reflect(default)]
    pub fallbacks: Vec<LocalizedImageFallback>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering localized image assets that need to be pre-loaded.
///
/// The loaded images can be accessed via [`ImageMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadLocalizedImages(pub Vec<LocalizedImage>);

impl Command for LoadLocalizedImages
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_localized_images);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering image assets that need to be pre-loaded.
///
/// The loaded images can be accessed via [`ImageMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadImages(pub Vec<String>);

impl Command for LoadImages
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_images);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ImageLoadPlugin;

impl Plugin for ImageLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<ImageMap>()
            .register_asset_tracker::<ImageMap>()
            .register_command::<LoadImages>()
            .register_command::<LoadLocalizedImages>()
            .react(|rc| rc.on_persistent(broadcast::<LanguagesNegotiated>(), handle_new_lang_list))
            .react(|rc| {
                rc.on_persistent(
                    (broadcast::<ImageMapLoaded>(), broadcast::<RelocalizeApp>()),
                    relocalize_images,
                )
            })
            .add_systems(PreUpdate, check_loaded_images.in_set(LoadProgressSet::Prepare));
    }
}

//-------------------------------------------------------------------------------------------------------------------

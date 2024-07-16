use std::io;
use std::path::PathBuf;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext, LoadDirectError};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use fluent::bundle::FluentBundle;
use fluent::FluentResource;
use intl_memoizer::concurrent::IntlLangMemoizer;
use ron::error::SpannedError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unic_langid::LanguageIdentifier;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FtlBundleData
{
    locale: LanguageIdentifier,
    resources: Vec<PathBuf>,
}

//-------------------------------------------------------------------------------------------------------------------

async fn load_ftl_bundle_contents(
    data: FtlBundleData,
    load_context: &mut LoadContext<'_>,
) -> Result<FtlBundle, FtlLoadError>
{
    let mut bundle = FluentBundle::new_concurrent(vec![data.locale]);
    for mut path in data.resources {
        if path.is_relative() {
            if let Some(parent) = load_context.path().parent() {
                path = parent.join(path);
            }
        }
        let loaded = load_context.loader().direct().untyped().load(path).await?;
        let resource = loaded.take::<FtlResource>().unwrap();
        if let Err(errors) = bundle.add_resource(resource.0) {
            warn_span!("add_ftl_resource").in_scope(|| {
                for error in errors {
                    warn!(%error);
                }
            });
        }
    }

    //TODO: Bevy does not support directional isolates until v0.15 w/ cosmic-text integration
    // - https://github.com/projectfluent/fluent-rs/issues/172
    // - https://docs.rs/fluent-bundle/0.15.3/fluent_bundle/bundle/struct.FluentBundle.html#method.set_use_isolating
    // - https://unicode.org/reports/tr9/#Explicit_Directional_Isolates
    bundle.set_use_isolating(false);
    tracing::debug!("loaded FluentBundle with .set_use_isolating(false), directional isolates not supported by \
        bevy_text");

    Ok(FtlBundle(bundle))
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Asset, TypePath)]
struct FtlResource(FluentResource);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
struct FtlResourceLoader;

impl AssetLoader for FtlResourceLoader
{
    type Asset = FtlResource;
    type Settings = ();
    type Error = FtlLoadError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _: &'a Self::Settings,
        _: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error>
    {
        let mut content = String::new();
        reader.read_to_string(&mut content).await?;

        let resource = match FluentResource::try_new(content) {
            Ok(fluent_resource) => fluent_resource,
            Err((fluent_resource, errors)) => {
                warn_span!("load_ftl_resource").in_scope(|| {
                    for error in errors {
                        warn!(%error);
                    }
                });
                fluent_resource
            }
        };

        Ok(FtlResource(resource))
    }

    fn extensions(&self) -> &[&str]
    {
        &["ftl"]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// [`AssetLoader`](bevy::asset::AssetLoader) implementation for [`FtlBundle`].
#[derive(Default)]
struct FtlBundleLoader;

impl AssetLoader for FtlBundleLoader
{
    type Asset = FtlBundle;
    type Settings = ();
    type Error = FtlLoadError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error>
    {
        let path = load_context.path();
        let mut content = String::new();
        reader.read_to_string(&mut content).await?;
        match path.extension() {
            Some(extension) if extension == "ron" => {
                load_ftl_bundle_contents(ron::de::from_str(&content)?, load_context).await
            }
            Some(extension) if extension == "yaml" || extension == "yml" => {
                load_ftl_bundle_contents(serde_yaml::from_str(&content)?, load_context).await
            }
            _ => unreachable!("We already checked all the supported extensions."),
        }
    }

    fn extensions(&self) -> &[&str]
    {
        &["ftl.ron", "ftl.yaml", "ftl.yml"]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Error that can be emitted when [`FtlBundle`] files or their internal [`FluentResource`] files fail to load.
#[derive(Debug, Error)]
enum FtlLoadError
{
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    LoadDirect(#[from] LoadDirectError),
    #[error(transparent)]
    Ron(#[from] SpannedError),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

//-------------------------------------------------------------------------------------------------------------------

/// Wrapper for [`FluentBundle`](fluent::bundle::FluentBundle).
///
/// Collection of [`FluentResources`](`FluentResource`) for a single locale.
#[derive(Asset, TypePath, Deref)]
pub(crate) struct FtlBundle(FluentBundle<FluentResource, IntlLangMemoizer>);

impl FtlBundle
{
    /// Gets this bundle's locale.
    pub(crate) fn locale(&self) -> &LanguageIdentifier
    {
        &self.locales[0]
    }

    /// Adds a locale for use as a fallback when doing formatting (e.g. date/time formatting).
    fn _add_formatting_fallback(&mut self, locale: LanguageIdentifier)
    {
        self.0.locales.push(locale);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Plugin to load [`FtlBundle`] assets.
pub(crate) struct FtlBundleAssetLoaderPlugin;

impl Plugin for FtlBundleAssetLoaderPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_asset::<FtlResource>()
            .init_asset::<FtlBundle>()
            .register_asset_loader(FtlResourceLoader)
            .register_asset_loader(FtlBundleLoader);
    }
}

//-------------------------------------------------------------------------------------------------------------------

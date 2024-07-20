Localization is the process of swapping out text, fonts, images, sounds, etc. based on the language preferences of a user.

At the center of localization is *language negotiation*, which takes the user's requested languages and compares them with languages available in the app to determine what languages should be prioritized when setting text/images/etc. This crate uses an opinionated negotiation [algorithm](bevy_cobweb_ui::prelude::LocalizationManifest::negotiated) that minimizes the number of selected languages to avoid excessive language mixing when your app is only partially localized in some languages.


## Locale

Users can define an arbitrary requested language list in the [`Locale`](bevy_cobweb_ui::prelude::Locale) resource. Every time `Locale` is updated, the app will automatically process it and re-localize text and assets as needed.

It is recommended to set an initial value for `Locale` during `Startup` (e.g. if you have saved user settings).

For example:
```rust
fn setup(mut locale: ResMut<Locale>)
{
    *locale = Locale::new("fr-FR");
}
```


## LocalizationManifest

In order to coordinate localization across different assets, you need to specify the available languages in a [`LoadLocalizationManifest`](bevy_cobweb_ui::prelude::LoadLocalizationManifest) command. The command can be inserted to a cobweb asset file, or added to the app manually as a normal [`Command`](bevy::ecs::world::Command).

For convenience, the [`LocalizationManifest`](bevy_cobweb_ui::prelude::LocalizationManifest) resource stores manifests of `fluent` resources for localizing text.

For example:
```rust
fn setup(mut c: Commands)
{
    c.add(LoadLocalizationManifest{
        default: LocalizationMetaReflected{
            id: "en-US".into(),
            name: Some("English".into()),
            manifest: "locales/en-US/main.ftl.ron".into(),
        },
        alts: vec![
            LocalizationMetaReflected{
                id: "fr-FR".into(),
                name: Some("French".into()),
                manifest: "locales/fr-FR/main.ftl.ron".into(),
                allow_as_fallback: true,
            }
        ]
    });
}
```

The default language should be your app's primary language which can be treated as a global fallback if an asset is only partially localized.

### Localization events

This crate is designed to only load localization resources/assets on-demand. This means there is a potential delay between negotiating languages and the app being ready to localize existing text and assets.

To coordinate localization across multiple asset managers, we broadcast reactive events at specific points in the workflow.

- [`LocalizationManifestUpdated`](bevy_cobweb_ui::prelude::LocalizationManifestUpdated): Emitted when `LocalizationManifest` is loaded with a fresh `LoadLocalizationManifest` value.
- [`LanguagesNegotiated`](bevy_cobweb_ui::prelude::LanguagesNegotiated): Emitted when `LocalizationManifest` has negotiated languages (either because of a fresh `LoadLocalizationManifest`, or because `Locale` changed). This is a signal for asset managers to load/unload their internally-tracked assets based on the new negotiated language list.
- [`RelocalizeApp`](bevy_cobweb_ui::prelude::RelocalizeApp): Emitted after `LanguagesNegotiated` when the app has finished any asset loads triggered by `LanguagesNegotiated`. This is a signal for the app to relocalize existing text and assets with the updated localized assets.
- Individual asset managers emit their own events when loaded. For example see [`TextLocalizerLoaded`](bevy_cobweb_ui::prelude::TextLocalizerLoaded).

The first `RelocalizeApp` event will occur immediately before entering [`LoadState::Done`](bevy_cobweb_ui::prelude::LoadState::Done), assuming you add a `LoadLocalizationManifest` command at startup or using a cobweb asset file.

We avoid relocalizing the app until all assets have loaded to avoid jank from assets loading asynchronously. This means there may be a delay between the user selecting a new language and that language being applied. You can use the `LanguagesNegotiated -> RelocalizeApp` event sequence to display a 'loading' indicator to users.

**Caveat**: A weakness of the current design is that until the first `RelocalizeApp` event is emitted, localization is not ready. This means initial loading screens *cannot* be localized using this crate, since the first `RelocalizeApp` event won't be emitted until all initial assets are done loading.


## Text/font localization

Text localization is somewhat complicated because text can contain dynamic content (e.g. numbers derived from app state). To address this we integrate the [`fluent`](https://projectfluent.org/) localization framework. All localized text should be written as `fluent` strings that reference `fluent` resources.

Text also has the issue that not all fonts support all languages. We allow specifying font fallbacks when loading fonts to [`FontMap`](bevy_cobweb_ui::prelude::FontMap), which will be automatically applied to text sections using the language used to localize those sections.

### Fluent resources

`Fluent` resources for a given language should be listed in a `fluent` manifest (which needs to be added to `LoadLocalizationManifest`).

For example (a manifest at `assets/locales/en-US/main.ftl.ron`):
```ron
(
    locale: "en-US",
    resources: [
        "menu.ftl",
        "characters.ftl"
    ]
)
```

Then each `fluent` resource will contain primarily key-value sequences for text content.

For example (a resource at `assets/locales/en-US/menu.ftl`)
```
menu-home = Home
menu-settings = Settings
```

**Note**: When `fluent` resources are extracted and bundled, we *do not* add any date/time/etc. formatting [fallback languages](https://docs.rs/fluent/0.16.1/fluent/bundle/struct.FluentBundle.html#locale-fallback-chain). This is because we cannot display multiple fonts in a single text section, in the case where a different language is used to format a data/time from the surrounding text and the formatting language isn't supported by the text's font. If a text section fails to localize due to formatting issues, then it should just fall back within the negotiated languages list.

### Localizing text

Adding text localization is as simple as adding a [`LocalizedText`](bevy_cobweb_ui::prelude::LocalizedText) component to your enity, and writing a `fluent` key into the text on that entity.

For example (with a `fluent` resource that has a `hello-world = Hello, World!` entry):
```rust
fn spawn_text(mut c: Commands)
{
    c.spawn((
        LocalizedText::default(),
        Text::from_section(TextSection::from_text("hello-world"));
    ));
}
```

When `LocalizedText` is inserted the first time to an entity, text on the entity will be auto-localized. After that, you need to use the [`TextEditor`](bevy_cobweb_ui::prelude::TextEditor) system parameter to edit text if you want your edits to be localized. We don't auto-localize in a system because we want to allow animating text color, which will trigger change detection on `Text` components.

For example (with a `fluent` resource that has a `game-score = Score: {$score}` entry):
```rust
fn text_editing(mut e: TextEditor, score: Res<GameScore>)
{
    let score_text = score.text_entity(); // For brevity let's assume the entity is in here.
    write_text!(e, score_entity, "game-score?score={}", score.score());
}
```

Note that `TextEditor` is optimized to avoid allocations when writing to dynamic text. It will also perform font localization automatically.


## Asset localization

Asset localization is designed to be mostly invisible to users. You simply 'get' handles from asset managers, and the returned handle will be localized appropriately (e.g. see [`ImageMap::get`](bevy_cobweb_ui::prelude::ImageMap::get)).

For example:
```rust
fn add_image(mut c: Commands, images: Res<ImageMap>)
{
    c.spawn(ImageBundle{
        // The returned handle may point to a different image if
        // there is an appropriate localization fallback.
        image: images.get("images/title.png"),
        ..default()
    });
}
```

In general, asset managers are optimized to only keep in memory what you need. When languages are changed, existing components on entities will be updated with new handles as needed.

See the [`assets_ext`](bevy_cobweb_ui::assets_ext) module for currently-implemented asset managers with localization support.

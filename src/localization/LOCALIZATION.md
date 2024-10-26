Localization is the process of swapping out text, fonts, images, sounds, etc. based on the language preferences of a user.

Since a user's language preference may not match available languages, there must be *language negotiation* to identify what languages should be prioritized when setting text/images/etc. This crate uses an opinionated negotiation [algorithm](bevy_cobweb_ui::prelude::LocalizationManifest::negotiated) that minimizes the number of selected languages to avoid excessive language mixing when your app is only partially localized in some languages.


## Locale

You can set preferred languages in the [`Locale`](bevy_cobweb_ui::prelude::Locale) resource.

For example:
```rust
fn setup(mut locale: ResMut<Locale>)
{
    *locale = Locale::new("fr-FR");
}
```

Every time `Locale` is updated, a new negotiated language list will be generated, and text and assets will be re-localized as needed.

Negotiated languages are stored in the [`LocalizationManifest`](bevy_cobweb_ui::prelude::LocalizationManifest) resource, which is integrated with text localization.


## Text localization

Text is localized using the [`fluent`](https://projectfluent.org/) localization framework. This framework requires 'fluent resources', which are asset files containing key:string pairs of text snippets to localize. Only the strings are translated. Users will use the `fluent` keys to request localized text on their entities.

If the user's requested language is only 'partially translated' (meaning it only has translated strings for some keys), then a fallback language from the negotiated language list will be used as needed.

### Fluent resources

As mentioned above, a `fluent` resource is a file containing key:string pairs for a language.

For example, here is a resource `menu.ftl` for English at `assets/locales/en-US/menu.ftl`:
```
menu-home = Home
menu-settings = Settings
```

A language can have many resource files, so they are managed by `fluent` bundles.

Here is a bundle `main.ftl.ron` at `assets/locales/en-US/main.ftl.ron`:
```ron
(
    locale: "en-US",
    resources: [
        "menu.ftl",
        "characters.ftl"
    ]
)
```

### Localization manifest

You can add `fluent` bundles to your app using the [`LoadLocalizationManifest`](bevy_cobweb_ui::prelude::LoadLocalizationManifest) command.

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

This command will reset the [`LocalizationManifest`](bevy_cobweb_ui::prelude::LocalizationManifest) resource, which has two important roles:
1. It stores `fluent` bundles for localizing text.
2. It manages the negotiated language list for localizing all assets. The 'available languages' of an app are equal to the languages provided by `LoadLocalizationManifest`. Those languages are negotiated against the languages in [`Locale`](bevy_cobweb_ui::prelude::Locale) to get the negotiated language list.

The default language should be your app's primary language which can be treated as a global fallback if an asset is only partially localized.

**Note**: When `fluent` resources are extracted, we *do not* add any date/time/etc. formatting [fallback languages](https://docs.rs/fluent/0.16.1/fluent/bundle/struct.FluentBundle.html#locale-fallback-chain). This is because we cannot display multiple fonts in a single text section, in the case where a different language is used to format a data/time from the surrounding text and the formatting language isn't supported by the text's font. If a text section fails to localize due to formatting issues, then it should just fall back within the negotiated languages list.

### Localizing text on entities

To localize text, add a [`LocalizedText`](bevy_cobweb_ui::prelude::LocalizedText) component to your entity and store a string with a `fluent` key in the `Text` component.

For example, assuming a `fluent` resource with a `hello-world = Hello, World!` entry is loaded:
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

For example, assuming a `fluent` resource with a `game-score = Score: {$score}` entry is loaded:
```rust
fn text_editing(mut e: TextEditor, score: Res<GameScore>)
{
    let score_text = score.text_entity(); // For brevity let's assume the entity is in here.
    write_text!(e, score_entity, "game-score?score={}", score.score());
}
```

Note that `TextEditor` is optimized to avoid allocations when writing to dynamic text.

### Font localization

Since most fonts don't support all languages, it is necessary to add font fallbacks for different languages. This can be done with the [`LoadLocalizedFonts`](bevy_cobweb_ui::prelude::LoadLocalizedFonts) command, which will update the [`FontMap`](bevy_cobweb_ui::prelude::FontMap) resource.

Fonts are auto-localized when `LocalizedText` is inserted to an entity, when text is updated with `TextEditor`, or when using [`TextEditor::set_font`](bevy_cobweb_ui::prelude::TextEditor::set_font). This ensures font localization is mostly invisible to app code, which just needs to pass primary fonts around.


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


## Localization events

This crate is designed to only load localization resources/assets on-demand. This means there is a potential delay between negotiating languages and the app being ready to localize existing text and assets.

To coordinate localization across multiple asset managers, we broadcast reactive events at specific points in the workflow.

- [`LocalizationManifestUpdated`](bevy_cobweb_ui::prelude::LocalizationManifestUpdated): Emitted when `LocalizationManifest` is loaded with a fresh `LoadLocalizationManifest` value.
- [`LanguagesNegotiated`](bevy_cobweb_ui::prelude::LanguagesNegotiated): Emitted when `LocalizationManifest` has negotiated languages (either because of a fresh `LoadLocalizationManifest`, or because `Locale` changed). This is a signal for asset managers to load/unload their internally-tracked assets based on the new negotiated language list.
- [`RelocalizeApp`](bevy_cobweb_ui::prelude::RelocalizeApp): Emitted after `LanguagesNegotiated` when the app has finished any asset loads triggered by `LanguagesNegotiated`. This is a signal for the app to relocalize existing text and assets with the updated localized assets.
- Individual asset managers emit their own events when loaded. For example see [`TextLocalizerLoaded`](bevy_cobweb_ui::prelude::TextLocalizerLoaded).

The first `RelocalizeApp` event will occur immediately before entering [`LoadState::Done`](bevy_cobweb_ui::prelude::LoadState::Done), assuming you add a `LoadLocalizationManifest` command at startup or are using a cobweb asset file.

We avoid relocalizing the app until all assets have loaded to avoid jank from assets loading asynchronously. This means there may be a delay between the user selecting a new language and that language being applied. You can use the `LanguagesNegotiated -> RelocalizeApp` event sequence to display a 'loading' indicator to users.

**Caveat**: A weakness of the current design is that until the first `RelocalizeApp` event is emitted, localization is not ready. This means initial loading screens *cannot* be localized using this crate, since the first `RelocalizeApp` event won't be emitted until all initial assets are done loading.

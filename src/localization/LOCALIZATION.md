TODO

Locale
LocalizationManifest
LoadLocalizationManifest

LocalizedText
TextLocalizer
TextEditor

FontLocalizer ?
ImageLocalizer ?


- Localizing text is as simple as A) adding a `LocalizedText` component to localize all text sections on an entity (existing `Text` is auto-localized on insert of `LocalizedText`), B) using the `TextEditor` system param for editing `Text` on entities. Using `TextEditor` optimizes text updates by writing directly to text components and buffered localization templates. You can also freely animate text color without worrying about spurious localization work due to change detection on `Text`.
- Users can define an arbitrary requested language list in the `Locale` resource, which will be negotiated against registered languages.
- Language negotiation with a best-effort fallback to the user's system language (and app devs can decide what languages are eligible as fallbacks). This way, for example, someone in the US can set their preferred lang to Spanish, and if Spanish is not completely translated then the first fallback will be `en-US` (if their system language is `en-US`, and if `en-US` is designated as an allowed fallback), even if the app's default language is German (because the dev is German).
- Automatic font, image, audio, etc. localization with a complete asset management framework.
- Optimized localization data management - only keep in memory what you need.
- Startup-load state tracking integrated with localization so localization data will be ready in your desired language(s) for UI construction/etc. post-load screen.
- Change preferred languages at runtime without needing to restart your app or rebuild any entities. (the big detail here is properly remapping fonts, since font fallbacks don't necessarily map 1:1)

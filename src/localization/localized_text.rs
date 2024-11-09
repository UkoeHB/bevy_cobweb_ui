use bevy::prelude::*;
use bevy::ui::UiSystem;
use bevy_cobweb::prelude::*;
use fluent_langneg::LanguageIdentifier;
use smallvec::SmallVec;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// System that runs whenever `TextLocalizer` is reloaded or when the `RelocalizeApp` event is emitted.
///
/// Handles newly-loaded or updated languages when there is existing text.
///
/// Note that this may redundantly relocalize text that was spawned during startup.
fn relocalize_text(
    localizer: Res<TextLocalizer>,
    fonts: Res<FontMap>,
    mut localized_text: Query<(Entity, &mut LocalizedText)>,
    mut writer: TextUiWriter,
)
{
    // Re-localize all text and fonts.
    for (entity, mut localized) in localized_text.iter_mut() {
        let mut idx = 0;

        writer.for_each(entity, |_, _, mut text, mut font, _| {
            localized.localize_span(&localizer, &fonts, &mut *text, &mut font.font, idx);
            idx += 1;
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System that runs whenever `FontMap` is reloaded via `LoadFonts`.
///
/// Handles changes to font fallbacks when there is existing text.
fn handle_font_refresh(
    fonts: Res<FontMap>,
    mut localized_text: Query<(Entity, &mut LocalizedText)>,
    mut writer: TextUiWriter,
)
{
    // Re-localize all fonts.
    for (entity, mut localized) in localized_text.iter_mut() {
        let mut idx = 0;

        writer.for_each_font(entity, |mut font| {
            let this_idx = idx;
            idx += 1;
            let Some(loc_span) = localized.localization_for_span_mut(this_idx) else { return };
            loc_span.update_font(&fonts, &mut font.font);
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System for auto-localizing text and font when `LocalizedText` is inserted.
///
/// This allows users to insert text when spawining an entity without needing to do a roundabout access to
/// `TextEditor` just to write a single permanent text value (and also set the font).
fn handle_new_localized_text(
    localizer: Res<TextLocalizer>,
    fonts: Res<FontMap>,
    mut localized_text: Query<(Entity, &mut LocalizedText), Added<LocalizedText>>,
    mut writer: TextUiWriter,
)
{
    for (entity, mut localized) in localized_text.iter_mut() {
        let mut idx = 0;

        writer.for_each(entity, |_, _, mut text, mut font, _| {
            let this_idx = idx;
            idx += 1;

            // Ignore empty spans.
            // - We assume if empty spans need to be localized, it will be handled by `TextEditor` when the user
            //   writes to those spans.
            if text.is_empty() {
                return;
            }

            // Ignore spans if the associated localization template is non-empty.
            // - These text spans have already been localized.
            // - NOTE: Normally we don't repair partially-localized text, but in this case we look through all the
            //   spans regardless of if other spans have been localized.
            if let Some(span_loc) = localized.localization_for_span(this_idx) {
                if span_loc.lang().is_some() {
                    return;
                }
            }

            // Now localize the span.
            localized.set_localization_for_span(text.as_str(), this_idx);
            localized.localize_span(&localizer, &fonts, &mut *text, &mut font.font, this_idx);
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Result returned by [`LocalizedTextspan::localize`].
pub enum TextLocalizationResult
{
    /// The text was localized to a different language from what it had before.
    NewLang,
    /// The text was localized to the same language it had before.
    SameLang,
    /// Localization failed.
    Fail,
}

//-------------------------------------------------------------------------------------------------------------------

/// Localization templates for a specific [`Textspan`] in a [`Text`] component on an entity.
///
/// Includes the language currently loaded to each span, which can be used to accurately set fallback
/// fonts.
#[derive(Reflect, Clone, Default, Debug, PartialEq)]
pub struct LocalizedTextspan
{
    #[reflect(ignore)]
    id: Option<LanguageIdentifier>,
    #[reflect(ignore)]
    font_backup: Option<Handle<Font>>,
    /// The localization template that will be used to generate localized text strings.
    #[reflect(ignore)]
    pub template: String,
}

impl LocalizedTextspan
{
    /// Gets the language currently applied to the associated text span on the entity.
    ///
    /// This will be `None` if the text span has not been localized yet. See
    /// [`LocalizedText::localize_span`].
    pub fn lang(&self) -> &Option<LanguageIdentifier>
    {
        &self.id
    }

    /// Gets the main font associated with this text span.
    ///
    /// If this is `None` then the current font on the text span is associated with the app's default
    /// language. See [`LocalizedText::localize_span`].
    ///
    /// Used to coordinate switching to a new fallback font if the language changes.
    pub fn font_backup(&self) -> &Option<Handle<Font>>
    {
        &self.font_backup
    }

    /// Localizes this text span.
    pub fn localize(&mut self, localizer: &TextLocalizer, target: &mut String) -> TextLocalizationResult
    {
        let Some(lang) = localizer.localize(&self.template, target) else { return TextLocalizationResult::Fail };
        if self.id.as_ref() == Some(lang) {
            return TextLocalizationResult::SameLang;
        }
        self.id = Some(lang.clone());
        TextLocalizationResult::NewLang
    }

    /// Sets the font backup, which is used to coordinate font lookups when negotiated languages change.
    pub fn set_font_backup(&mut self, backup: Handle<Font>)
    {
        self.font_backup = Some(backup);
    }

    /// Updates the text span's font to use a fallback if the current font is not supported by the
    /// text span's currently-set language.
    ///
    /// If there is no current font backup, then the initial value in `target` will be set as the backup.
    pub fn update_font(&mut self, fonts: &FontMap, target: &mut Handle<Font>)
    {
        // If no font backup is selected, then treat the target as a 'main font'.
        if self.font_backup.is_none() {
            self.font_backup = Some(target.clone());
        }

        // Get the correct localized font derived from the main font.
        let Some(lang_id) = &self.id else {
            tracing::warn!("failed setting localized font on a text span, the \
                span has not been localized yet; current font is {:?}", target.path());
            return;
        };
        let backup = self.font_backup.as_ref().unwrap();
        let new_handle = fonts
            .get_localized(lang_id, backup.id())
            .unwrap_or_else(|| backup.clone());
        *target = new_handle;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component for localizing text, compatible with [`Text`] components.
///
/// To localize text, first call [`LocalizedText::set_localization`] with the localization template, then call
/// [`LocalizedText::localize`] to update the target string. See [`fluent_content::Request`] for the template
/// syntax.
///
/// When this component is inserted on an entity, existing text will be automatically localized.
/// Then to update localization templates on entities you should use the [`TextEditor`] helper, which uses this
/// component to auto-localize text.
///
/// **NOTE**: Automatic directional isolation of parameters is supported
/// See [here][fluent-isolation] and [here][directional-isolates].
///
/// [fluent-isolation](https://docs.rs/fluent-bundle/0.15.3/fluent_bundle/bundle/struct.FluentBundle.html#method.set_use_isolating)
/// [directional-isolates](https://unicode.org/reports/tr9/#Explicit_Directional_Isolates)
#[derive(Component, Reflect, Clone, Debug, PartialEq)]
pub struct LocalizedText
{
    /// Localization templates for each [`Textspan`] in the [`Text`] component on this entity.
    ///
    /// This should be updated before calling [`LocalizedText::localize`] if the localization template has
    /// changed.
    #[reflect(ignore, default = "LocalizedText::default_loc")]
    localization: SmallVec<[LocalizedTextspan; 1]>,
}

impl LocalizedText
{
    /// Sets the cached localization template for the first span in the entity's [`Text`].
    pub fn set_localization(&mut self, data: impl AsRef<str>)
    {
        self.set_localization_for_span(data, 0);
    }

    /// Sets the cached localization template for a specific span in the entity's [`Text`].
    pub fn set_localization_for_span(&mut self, data: impl AsRef<str>, span: usize)
    {
        if self.localization.len() <= span {
            self.localization
                .resize(span + 1, LocalizedTextspan::default());
        }
        let localized_span = &mut self.localization[span];
        localized_span.template.clear();
        localized_span.template.push_str(data.as_ref());
    }

    /// Gets a reference to the cached localization template for the first span in the entity's [`Text`].
    pub fn localization(&self) -> &LocalizedTextspan
    {
        // Safe unwrap: struct can't be constructed with 0 spans.
        self.localization_for_span(0).unwrap()
    }

    /// Gets a mutable reference to the cached localization template for the first span in the entity's
    /// [`Text`].
    pub fn localization_mut(&mut self) -> &mut LocalizedTextspan
    {
        // Safe unwrap: struct can't be constructed with 0 spans.
        self.localization_for_span_mut(0).unwrap()
    }

    /// Gets a reference to the cached localization template for the first span in the entity's [`Text`].
    pub fn localization_for_span(&self, span: usize) -> Option<&LocalizedTextspan>
    {
        self.localization.get(span)
    }

    /// Gets a mutable reference to the cached localization template for the first span in the entity's
    /// [`Text`].
    pub fn localization_for_span_mut(&mut self, span: usize) -> Option<&mut LocalizedTextspan>
    {
        self.localization.get_mut(span)
    }

    /// Localizes the first text span on an entity.
    ///
    /// See [`Self::localize_span`].
    pub fn localize(
        &mut self,
        localizer: &TextLocalizer,
        fonts: &FontMap,
        target: &mut String,
        font: &mut Handle<Font>,
    ) -> bool
    {
        self.localize_span(localizer, fonts, target, font, 0)
    }

    /// Uses the localization template in `span` to set `target` with a string localized via [`TextLocalizer`].
    ///
    /// Will update the text's font if the text's language changes (including when localization is initialized).
    ///
    /// Returns `false` if localization failed, which can happen if no language is loaded yet.
    pub fn localize_span(
        &mut self,
        localizer: &TextLocalizer,
        fonts: &FontMap,
        target: &mut String,
        font: &mut Handle<Font>,
        span: usize,
    ) -> bool
    {
        let Some(loc_span) = self.localization_for_span_mut(span) else {
            tracing::warn!("tried to localize text span {span} of an entity, but no localization template is \
                available for this span");
            return false;
        };

        // Localize it.
        match loc_span.localize(localizer, target) {
            TextLocalizationResult::Fail => {
                tracing::warn!("failed localizing {:?} template for text span {span} on an entity",
                    loc_span.template);
                return false;
            }
            TextLocalizationResult::NewLang => {
                // Update font.
                // - We assume we only need to update the font when the language changes.
                loc_span.update_font(fonts, font);
            }
            TextLocalizationResult::SameLang => (),
        }

        true
    }

    fn default_loc() -> SmallVec<[LocalizedTextspan; 1]>
    {
        SmallVec::from_buf([LocalizedTextspan::default()])
    }
}

impl Default for LocalizedText
{
    fn default() -> Self
    {
        Self { localization: Self::default_loc() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LocalizedTextPlugin;

impl Plugin for LocalizedTextPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<LocalizedText>()
            .register_bundle::<LocalizedText>()
            .react(|rc| rc.on_persistent(broadcast::<RelocalizeApp>(), relocalize_text))
            .react(|rc| rc.on_persistent(broadcast::<TextLocalizerLoaded>(), relocalize_text))
            .react(|rc| rc.on_persistent(broadcast::<FontMapLoaded>(), handle_font_refresh))
            .configure_sets(PostUpdate, LocalizationSet::Update.before(UiSystem::Prepare))
            .add_systems(PostUpdate, handle_new_localized_text.in_set(LocalizationSet::Update));
    }
}

//-------------------------------------------------------------------------------------------------------------------

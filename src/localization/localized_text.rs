use bevy::prelude::*;
use bevy::ui::UiSystem;
use bevy_cobweb::prelude::*;
use fluent_langneg::LanguageIdentifier;
use serde::{Deserialize, Serialize};
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
    mut text: Query<(&mut LocalizedText, &mut Text)>,
)
{
    // Re-localize all text and fonts.
    for (mut localized, mut text) in text.iter_mut() {
        for (idx, section) in text.sections.iter_mut().enumerate() {
            localized.localize_section(&localizer, &fonts, &mut section.value, &mut section.style.font, idx);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System that runs whenever `FontMap` is reloaded via `LoadFonts`.
///
/// Handles changes to font fallbacks when there is existing text.
fn handle_font_refresh(fonts: Res<FontMap>, mut text: Query<(&mut LocalizedText, &mut Text)>)
{
    // Re-localize all fonts.
    for (mut localized, mut text) in text.iter_mut() {
        for (idx, section) in text.sections.iter_mut().enumerate() {
            let Some(loc_section) = localized.localization_for_section_mut(idx) else { continue };
            loc_section.update_font(&fonts, &mut section.style.font);
        }
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
    mut text: Query<(&mut LocalizedText, &mut Text), Added<LocalizedText>>,
)
{
    for (mut localized, mut text) in text.iter_mut() {
        for (idx, section) in text.sections.iter_mut().enumerate() {
            // Ignore empty sections.
            // - We assume if empty sections need to be localized, it will be handled by `TextEditor` when the user
            //   writes to those sections.
            if section.value.is_empty() {
                continue;
            }

            // Ignore sections if the associated localization template is non-empty.
            // - These text sections have already been localized.
            // - NOTE: Normally we don't repair partially-localized text, but in this case we look through all the
            //   sections regardless of if other sections have been localized.
            if let Some(section_loc) = localized.localization_for_section(idx) {
                if section_loc.lang().is_some() {
                    continue;
                }
            }

            // Now localize the section.
            localized.set_localization_for_section(&section.value, idx);
            localized.localize_section(&localizer, &fonts, &mut section.value, &mut section.style.font, idx);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Result returned by [`LocalizedTextSection::localize`].
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

/// Localization templates for a specific [`TextSection`] in a [`Text`] component on an entity.
///
/// Includes the language currently loaded to each section, which can be used to accurately set fallback
/// fonts.
#[derive(Reflect, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalizedTextSection
{
    #[reflect(ignore)]
    #[serde(skip)]
    id: Option<LanguageIdentifier>,
    #[reflect(ignore)]
    #[serde(skip)]
    font_backup: Option<Handle<Font>>,
    /// The localization template that will be used to generate localized text strings.
    #[reflect(ignore)]
    #[serde(skip)]
    pub template: String,
}

impl LocalizedTextSection
{
    /// Gets the language currently applied to the associated text section on the entity.
    ///
    /// This will be `None` if the text section has not been localized yet. See
    /// [`LocalizedText::localize_section`].
    pub fn lang(&self) -> &Option<LanguageIdentifier>
    {
        &self.id
    }

    /// Gets the main font associated with this text section.
    ///
    /// If this is `None` then the current font on the text section is associated with the app's default
    /// language. See [`LocalizedText::localize_section`].
    ///
    /// Used to coordinate switching to a new fallback font if the language changes.
    pub fn font_backup(&self) -> &Option<Handle<Font>>
    {
        &self.font_backup
    }

    /// Localizes this text section.
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

    /// Updates the text section's font to use a fallback if the current font is not supported by the
    /// text section's currently-set language.
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
            tracing::warn!("failed setting localized font on a text section, the \
                section has not been localized yet; current font is {:?}", target.path());
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
/// **NOTE**: Automatic directional isolation of parameters is not supported until `bevy` v0.15 when `cosmic-text`
/// will be integrated. See [here][fluent-isolation] and [here][directional-isolates].
///
/// [fluent-isolation](https://docs.rs/fluent-bundle/0.15.3/fluent_bundle/bundle/struct.FluentBundle.html#method.set_use_isolating)
/// [directional-isolates](https://unicode.org/reports/tr9/#Explicit_Directional_Isolates)
#[derive(Component, Reflect, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalizedText
{
    /// Localization templates for each [`TextSection`] in the [`Text`] component on this entity.
    ///
    /// This should be updated before calling [`LocalizedText::localize`] if the localization template has
    /// changed.
    #[reflect(ignore, default = "LocalizedText::default_loc")]
    #[serde(skip, default = "LocalizedText::default_loc")]
    localization: SmallVec<[LocalizedTextSection; 1]>,
}

impl LocalizedText
{
    /// Sets the cached localization template for the first section in the entity's [`Text`].
    pub fn set_localization(&mut self, data: impl AsRef<str>)
    {
        self.set_localization_for_section(data, 0);
    }

    /// Sets the cached localization template for a specific section in the entity's [`Text`].
    pub fn set_localization_for_section(&mut self, data: impl AsRef<str>, section: usize)
    {
        if self.localization.len() <= section {
            self.localization
                .resize(section + 1, LocalizedTextSection::default());
        }
        let localized_section = &mut self.localization[section];
        localized_section.template.clear();
        localized_section.template.push_str(data.as_ref());
    }

    /// Gets a reference to the cached localization template for the first section in the entity's [`Text`].
    pub fn localization(&self) -> &LocalizedTextSection
    {
        // Safe unwrap: struct can't be constructed with 0 sections.
        self.localization_for_section(0).unwrap()
    }

    /// Gets a mutable reference to the cached localization template for the first section in the entity's
    /// [`Text`].
    pub fn localization_mut(&mut self) -> &mut LocalizedTextSection
    {
        // Safe unwrap: struct can't be constructed with 0 sections.
        self.localization_for_section_mut(0).unwrap()
    }

    /// Gets a reference to the cached localization template for the first section in the entity's [`Text`].
    pub fn localization_for_section(&self, section: usize) -> Option<&LocalizedTextSection>
    {
        self.localization.get(section)
    }

    /// Gets a mutable reference to the cached localization template for the first section in the entity's
    /// [`Text`].
    pub fn localization_for_section_mut(&mut self, section: usize) -> Option<&mut LocalizedTextSection>
    {
        self.localization.get_mut(section)
    }

    /// Localizes the first text section on an entity.
    ///
    /// See [`Self::localize_section`].
    pub fn localize(
        &mut self,
        localizer: &TextLocalizer,
        fonts: &FontMap,
        target: &mut String,
        font: &mut Handle<Font>,
    ) -> bool
    {
        self.localize_section(localizer, fonts, target, font, 0)
    }

    /// Uses the localization template in `section` to set `target` with a string localized via [`TextLocalizer`].
    ///
    /// Will update the text's font if the text's language changes (including when localization is initialized).
    ///
    /// Returns `false` if localization failed, which can happen if no language is loaded yet.
    pub fn localize_section(
        &mut self,
        localizer: &TextLocalizer,
        fonts: &FontMap,
        target: &mut String,
        font: &mut Handle<Font>,
        section: usize,
    ) -> bool
    {
        let Some(loc_section) = self.localization_for_section_mut(section) else {
            tracing::warn!("tried to localize text section {section} of an entity, but no localization template is \
                available for this section");
            return false;
        };

        // Localize it.
        match loc_section.localize(localizer, target) {
            TextLocalizationResult::Fail => {
                tracing::warn!("failed localizing {:?} template for text section {section} on an entity",
                    loc_section.template);
                return false;
            }
            TextLocalizationResult::NewLang => {
                // Update font.
                // - We assume we only need to update the font when the language changes.
                loc_section.update_font(fonts, font);
            }
            TextLocalizationResult::SameLang => (),
        }

        true
    }

    fn default_loc() -> SmallVec<[LocalizedTextSection; 1]>
    {
        SmallVec::from_buf([LocalizedTextSection::default()])
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
            .register_loadable::<LocalizedText>()
            .react(|rc| rc.on_persistent(broadcast::<RelocalizeApp>(), relocalize_text))
            .react(|rc| rc.on_persistent(broadcast::<TextLocalizerLoaded>(), relocalize_text))
            .react(|rc| rc.on_persistent(broadcast::<FontMapLoaded>(), handle_font_refresh))
            .configure_sets(
                PostUpdate,
                LocalizationSet::Update
                    //todo: .before(UiSystem::Prepare) in bevy v0.15, see https://github.com/bevyengine/bevy/pull/14228
                    .before(bevy::ui::widget::measure_text_system)
                    .before(UiSystem::Layout),
            )
            .add_systems(PostUpdate, handle_new_localized_text.in_set(LocalizationSet::Update));
    }
}

//-------------------------------------------------------------------------------------------------------------------

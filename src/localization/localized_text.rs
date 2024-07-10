use bevy::prelude::*;
use bevy::ui::UiSystem;
use bevy_cobweb::prelude::*;
use fluent_langneg::LanguageIdentifier;
use smallvec::SmallVec;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// System that runs whenever `TextLocalizer` updates.
///
/// Handles newly-loaded languages when there is existing text.
///
/// Note that this may redundantly relocalize text that was spawned and edited with [`TextEditor`] during startup.
//todo: swap out fonts with FontTextLocalizer? w/ cached main fonts in LocalizedText
fn relocalize_all_text(localizer: Res<TextLocalizer>, mut text: Query<(&mut LocalizedText, &mut Text)>)
{
    // Don't do anything if a language is currently loading.
    if localizer.is_loading() {
        return;
    }

    // Re-localize all text.
    for (mut localized, mut text) in text.iter_mut() {
        for (idx, section) in text.sections.iter_mut().enumerate() {
            localized.localize_section(&localizer, &mut section.value, idx);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System for auto-localizing text when `LocalizedText` is inserted.
///
/// This allows users to insert static text on entity spawn without needing to do a roundabout access to
/// `TextEditor` just to write a single permanent text value.
fn handle_new_localized_text(
    localizer: Res<TextLocalizer>,
    mut text: Query<(&mut LocalizedText, &mut Text), Added<LocalizedText>>,
)
{
    for (mut localized, mut text) in text.iter_mut() {
        for (idx, section) in text.sections.iter_mut().enumerate() {
            // Ignore empty sections.
            if section.value.is_empty() {
                continue;
            }

            // Ignore sections if the associated localization template is non-empty.
            // - These text sections have already been localized.
            // - NOTE: There is a small edge case here where only *some* sections have already been localized, and
            //   the entity is in a partially-localized state. It's a small gotcha because it's the *only* case
            //   where we auto-repair partially-localized text.
            if let Some(section_loc) = localized.localization_for_section(idx) {
                if section_loc.id.is_some() {
                    continue;
                }
            }

            // Now localize the section.
            localized.set_localization_for_section(&section.value, idx);
            localized.localize_section(&localizer, &mut section.value, idx);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Localization templates for a specific [`TextSection`] in a [`Text`] component on an entity.
///
/// Includes the language currently loaded to each section, which can be used to accurately set fallback
/// fonts.
//toto: incorporate fallback fonts (need to store main font for font recovery)
#[derive(Debug, Clone, Default)]
pub struct LocalizedTextSection
{
    /// The language currently applied to the associated text section on the entity.
    ///
    /// This will be `None` if the text section has not been localized yet. See
    /// [`LocalizedText::localize_section`].
    pub id: Option<LanguageIdentifier>,
    /// The localization template that can be used to generate localized text strings.
    pub template: String,
}

impl LocalizedTextSection
{
    /// Localizes this text section.
    ///
    /// Returns `false` if localization failed, which can happen if no language is loaded yet.
    pub fn localize(&mut self, localizer: &TextLocalizer, target: &mut String) -> bool
    {
        let Some(lang) = localizer.localize(&self.template, target) else { return false };
        self.id = Some(lang.clone());
        true
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
/// Then to update localization templates on entities you must use the [`TextEditor`] helper, which uses this
/// component to auto-localize text.
///
/// **NOTE**: Automatic directional isolation of parameters is not supported until `bevy` v0.15. See
/// [here][fluent-isolation] and [here][directional-isolates].
///
/// [fluent-isolation](https://docs.rs/fluent-bundle/0.15.3/fluent_bundle/bundle/struct.FluentBundle.html#method.set_use_isolating)
/// [directional-isolates](https://unicode.org/reports/tr9/#Explicit_Directional_Isolates)
#[derive(Component, Debug)]
pub struct LocalizedText
{
    /// Localization templates for each [`TextSection`] in the [`Text`] component on this entity.
    ///
    /// This should be updated before calling [`LocalizedText::localize`] if the localization template has
    /// changed.
    //todo: #[reflect(skip, default = "LocalizedText::default_loc")]
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

    /// Uses the localization template in section 0 to set `target` with a string localized via [`TextLocalizer`].
    ///
    /// Returns `false` if localization failed, which can happen if no language is loaded yet.
    pub fn localize(&mut self, localizer: &TextLocalizer, target: &mut String) -> bool
    {
        self.localize_section(localizer, target, 0)
    }

    /// Uses the localization template in `section` to set `target` with a string localized via [`TextLocalizer`].
    ///
    /// Returns `false` if localization failed, which can happen if no language is loaded yet.
    pub fn localize_section(&mut self, localizer: &TextLocalizer, target: &mut String, section: usize) -> bool
    {
        let Some(loc_section) = self.localization_for_section_mut(section) else {
            tracing::warn!("tried to localize text section {section} of an entity, but no localization template is \
                available for this section");
            return false;
        };

        // Localize it.
        if !loc_section.localize(localizer, target) {
            tracing::warn!("failed localizing \"{:?}\" template for text section {section} on an entity",
                loc_section.template);
            return false;
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
        app.react(|rc| rc.on_persistent(broadcast::<TextLocalizerUpdated>(), relocalize_all_text))
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

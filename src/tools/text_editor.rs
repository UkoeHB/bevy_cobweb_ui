// Re-export this for `TextEditor::write`.
use std::fmt::Debug;
pub use std::fmt::Write;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::prelude::{FontMap, LocalizedText, TextLocalizer};

//-------------------------------------------------------------------------------------------------------------------

/// Helper system param for modifying [`Text`] components.
///
/// Includes automatic text and font localization when writing text or setting a new font.
///
/// **NOTE**: `TextEditor` uses a query internally, so text can't be edited in the same system where it is
/// inserted.
#[derive(SystemParam)]
pub struct TextEditor<'w, 's>
{
    text: Query<'w, 's, (&'static mut Text, Option<&'static mut LocalizedText>)>,
    localizer: Res<'w, TextLocalizer>,
    fonts: Res<'w, FontMap>,
}

impl<'w, 's> TextEditor<'w, 's>
{
    /// Gets a [`TextSection`] on an entity.
    ///
    /// Returns `Err` if the text section could not be found or the text is empty.
    pub fn section(&self, text_entity: Entity, section: usize) -> Option<&TextSection>
    {
        let Ok((text, _)) = self.text.get(text_entity) else { return None };
        text.sections.get(section)
    }

    /// Gets the style on the first text section on an entity.
    ///
    /// Returns `None` if the text section could not be found or the text is empty.
    pub fn style(&self, text_entity: Entity) -> Option<&TextStyle>
    {
        self.section_style(text_entity, 0)
    }

    /// Gets the style on a text section on an entity.
    ///
    /// Returns `None` if the text section could not be found or the text is empty.
    pub fn section_style(&self, text_entity: Entity, section: usize) -> Option<&TextStyle>
    {
        let text = self.section(text_entity, section)?;
        Some(&text.style)
    }

    /// Overwrites the text on the first text section on an entity.
    ///
    /// See [`Self::write_section`].
    ///
    /// This is used in the [`write_text`] helper macro.
    pub fn write<E: Debug>(
        &mut self,
        text_entity: Entity,
        writer: impl FnOnce(&mut String) -> Result<(), E>,
    ) -> bool
    {
        self.write_section(text_entity, 0, writer)
    }

    /// Overwrites the text on a text section on an entity.
    ///
    /// This will automatically localize the text and its font if the entity has the [`LocalizedText`] component.
    ///
    /// Returns `false` if the text section could not be accessed, if the writer fails, or if localization fails.
    ///
    /// This is used in the [`write_text_section`] helper macro.
    pub fn write_section<E: Debug>(
        &mut self,
        text_entity: Entity,
        section: usize,
        writer: impl FnOnce(&mut String) -> Result<(), E>,
    ) -> bool
    {
        let Ok((text, maybe_localized)) = self.text.get_mut(text_entity) else {
            tracing::warn!("failed writing to text section {section} of {text_entity:?}, entity not found");
            return false;
        };
        let Some(text_section) = text.into_inner().sections.get_mut(section) else {
            tracing::warn!("failed writing to text section {section} of {text_entity:?}, section doesn't exist");
            return false;
        };
        let text = &mut text_section.value;
        let font = &mut text_section.style.font;

        if let Some(mut localized) = maybe_localized {
            // Clear the localization string then write to it.
            localized.set_localization_for_section("", section);
            let localization_section = localized.localization_for_section_mut(section).unwrap();
            let result = match (writer)(&mut localization_section.template) {
                Ok(()) => true,
                Err(err) => {
                    tracing::warn!("failed writing to localized text section {section} of {text_entity:?}, \
                        write callback error {err:?}");
                    false
                }
            };
            // Localize the target string and its font.
            let result = result && localized.localize_section(&self.localizer, &self.fonts, text, font, section);
            result
        } else {
            text.clear();
            match (writer)(text) {
                Ok(()) => true,
                Err(err) => {
                    tracing::warn!("failed writing to text section {section} of {text_entity:?}, \
                        write callback error {err:?}");
                    false
                }
            }
        }
    }

    /// Sets the font on the first text section of an entity.
    ///
    /// See [`set_font_section`].
    ///
    /// Returns `false` if the text section could not be accessed or if the font was not registered in [`FontMap`].
    pub fn set_font(&mut self, entity: Entity, font: impl AsRef<str>) -> bool
    {
        self.set_section_font(entity, 0, font)
    }

    /// Sets the font on a text section of an entity.
    ///
    /// If the entity has [`LocalizedText`] then the font will be automatically localized.
    ///
    /// Returns `false` if the text section could not be accessed or if the font was not registered in [`FontMap`].
    pub fn set_section_font(&mut self, entity: Entity, section: usize, font: impl AsRef<str>) -> bool
    {
        let font = font.as_ref();
        let Ok((text, maybe_localized)) = self.text.get_mut(entity) else {
            tracing::warn!("failed setting font {font:?} on text section {section} of {entity:?}, entity \
                not found");
            return false;
        };
        let Some(text) = text.into_inner().sections.get_mut(section) else {
            tracing::warn!("failed setting font {font:?} on text section {section} of {entity:?}, section \
                doesn't exist");
            return false;
        };
        let font = self.fonts.get(font);
        if font == Handle::default() {
            tracing::warn!("failed setting font {font:?} on text section {section} of {entity:?}, font \
                not found in FontMap");
            return false;
        }
        let text_font = &mut text.style.font;

        // Set the requested font.
        *text_font = font.clone();

        // Handle localization.
        if let Some(mut localized) = maybe_localized {
            // Prep text section if missing.
            if localized.localization_for_section_mut(section).is_none() {
                localized.set_localization_for_section("", section);
            }

            // Update the font backup to the requested font.
            let loc_section = localized.localization_for_section_mut(section).unwrap();
            loc_section.set_font_backup(font.clone());

            // If text has not been localized yet, then early-out since we don't know what language is needed.
            // - We assume that when the text is eventually localized, the font will be properly updated via
            //   `LocalizedText::localize_section`.
            if loc_section.lang().is_none() {
                return true;
            }

            // Update the font.
            loc_section.update_font(&self.fonts, text_font);
        }

        true
    }

    /// Sets the font size on the first text section of an entity.
    pub fn set_font_size(&mut self, entity: Entity, size: f32)
    {
        self.set_section_font_size(entity, 0, size);
    }

    /// Sets the font size on a text section of an entity.
    pub fn set_section_font_size(&mut self, entity: Entity, section: usize, size: f32)
    {
        self.text
            .get_mut(entity)
            .ok()
            .and_then(|(text, _)| text.into_inner().sections.get_mut(section))
            .map(|section| {
                section.style.font_size = size;
            });
    }

    /// Sets the font color on the first text section of an entity.
    pub fn set_font_color(&mut self, entity: Entity, color: Color)
    {
        self.set_section_font_color(entity, 0, color);
    }

    /// Sets the font color on a text section of an entity.
    pub fn set_section_font_color(&mut self, entity: Entity, section: usize, color: Color)
    {
        self.text
            .get_mut(entity)
            .ok()
            .and_then(|(text, _)| text.into_inner().sections.get_mut(section))
            .map(|section| {
                section.style.color = color;
            });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper for writing text with a [`TextEditor`].
///
/// Example
/*
```rust
fn example(mut commands: Commands, mut text_editor: TextEditor)
{
    let entity = commands.spawn(TextBundle::default()).ie();

    // Macro call:
    write_text!(text_editor, entity, "Count: {}", 42);

    // Expands to:
    text_editor.write(entity, |text| write!(text, "Count: {}", 42));
}
```
*/
#[macro_export]
macro_rules! write_text {
    ($editor: ident, $entity: expr, $($arg:tt)*) => {{
        $editor.write($entity, |text| write!(text, $($arg)*))
    }};
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper for writing text to a text section with a [`TextEditor`].
///
/// Example
/*
```rust
fn example(mut commands: Commands, mut text_editor: TextEditor)
{
    let entity = commands.spawn(TextBundle::default()).ie();

    // Macro call:
    write_text_section!(text_editor, entity, 0, "Count: {}", 42);

    // Expands to:
    text_editor.write_section(entity, 0, |text| write!(text, "Count: {}", 42));
}
```
*/
#[macro_export]
macro_rules! write_text_section {
    ($editor: ident, $entity: expr, $section: expr, $($arg:tt)*) => {{
        $editor.write_section($entity, $section, |text| write!(text, $($arg)*))
    }};
}

//-------------------------------------------------------------------------------------------------------------------

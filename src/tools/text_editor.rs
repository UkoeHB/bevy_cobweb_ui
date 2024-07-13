// Re-export this for `TextEditor::write`.
use std::fmt::Debug;
pub use std::fmt::Write;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::prelude::{LocalizedText, TextLocalizer};

//-------------------------------------------------------------------------------------------------------------------

//todo: add set_font()/set_section_font() methods, which will detect font changes and localize them? then remove
// mutability from other methods
// - what about hot-reloaded text info? just use the editor API?

/// Helper system param for modifying [`Text`] components.
///
/// Includes automatic text localization when writing text.
///
/// **NOTE**: `TextEditor` uses a query internally, so text can't be edited in the same system where it is
/// inserted.
#[derive(SystemParam)]
pub struct TextEditor<'w, 's>
{
    text: Query<'w, 's, (&'static mut Text, Option<&'static mut LocalizedText>)>,
    localizer: Res<'w, TextLocalizer>,
}

impl<'w, 's> TextEditor<'w, 's>
{
    /// Gets a [`TextSection`] on an entity.
    ///
    /// Returns `Err` if the text section could not be found or the text is empty.
    pub fn section(&mut self, text_entity: Entity, section: usize) -> Option<&mut TextSection>
    {
        let Ok((text, _)) = self.text.get_mut(text_entity) else { return None };
        text.into_inner().sections.get_mut(section)
    }

    /// Gets the style on the first text section on an entity.
    ///
    /// Returns `None` if the text section could not be found or the text is empty.
    pub fn style(&mut self, text_entity: Entity) -> Option<&mut TextStyle>
    {
        self.section_style(text_entity, 0)
    }

    /// Gets the style on a text section on an entity.
    ///
    /// Returns `None` if the text section could not be found or the text is empty.
    pub fn section_style(&mut self, text_entity: Entity, section: usize) -> Option<&mut TextStyle>
    {
        let text = self.section(text_entity, section)?;
        Some(&mut text.style)
    }

    /// Overwrites the text on the first text section on an entity.
    ///
    /// This will automatically localize the text if the entity has the [`LocalizedText`] component.
    ///
    /// Returns `false` if the text section could not be accessed, if the writer fails, or if localization fails.
    ///
    /// See the [`write_text`] helper macro.
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
    /// This will automatically localize the text if the entity has the [`LocalizedText`] component.
    ///
    /// Returns `false` if the text section could not be accessed, if the writer fails, or if localization fails.
    ///
    /// See the [`write_text_section`] helper macro.
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
        let Some(text) = text.into_inner().sections.get_mut(section) else {
            tracing::warn!("failed writing to text section {section} of {text_entity:?}, section doesn't exist");
            return false;
        };
        let text = &mut text.value;

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
            // Localize the target string.
            let result = result && localized.localize_section(&self.localizer, text, section);
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

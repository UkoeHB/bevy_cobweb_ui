use bevy::{ecs::system::SystemParam, prelude::*};

// Re-export this for `TextEditor::write`.
pub use std::fmt::Write;

//-------------------------------------------------------------------------------------------------------------------

/// Helper system param for modifying [`Text`] components.
#[derive(SystemParam)]
pub struct TextEditor<'w, 's>
{
    text: Query<'w, 's, &'static mut Text>,
}

impl<'w, 's> TextEditor<'w, 's>
{
    /// Gets a [`TextSection`] on an entity.
    ///
    /// Returns `Err` if the text section could not be found or the text is empty.
    pub fn section(&mut self, text_entity: Entity, section: usize) -> Option<&mut TextSection>
    {
        let Ok(text) = self.text.get_mut(text_entity) else { return None };
        text.into_inner().sections.get_mut(section)
    }

    /// Overwrites the text on the first text section on an entity.
    ///
    /// Returns `false` if the text section could not be accessed or if the writer fails.
    pub fn write<E>(
        &mut self,
        text_entity: Entity,
        writer: impl FnOnce(&mut String) -> Result<(), E>
    ) -> bool
    {
        self.write_section(text_entity, 0, writer)
    }

    /// Gets the style on the first text section on an entity.
    ///
    /// Returns `None` if the text section could not be found or the text is empty.
    pub fn style(&mut self, text_entity: Entity) -> Option<&mut TextStyle>
    {
        self.section_style(text_entity, 0)
    }

    /// Overwrites the text on a text section on an entity.
    ///
    /// Returns `false` if the text section could not be accessed or if the writer fails.
    pub fn write_section<E>(
        &mut self,
        text_entity: Entity,
        section: usize,
        writer: impl FnOnce(&mut String) -> Result<(), E>
    ) -> bool
    {
        let Some(text) = self.section(text_entity, section) else { return false };
        let text = &mut text.value;
        text.clear();
        (writer)(text).is_ok()
    }

    /// Gets the style on a text section on an entity.
    ///
    /// Returns `None` if the text section could not be found or the text is empty.
    pub fn section_style(&mut self, text_entity: Entity, section: usize) -> Option<&mut TextStyle>
    {
        let Some(text) = self.section(text_entity, section) else { return None };
        Some(&mut text.style)
    }
}

//-------------------------------------------------------------------------------------------------------------------

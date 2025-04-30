use std::fmt::Debug;
pub use std::fmt::Write; // Re-export this for `TextEditor::write`.
use std::ops::DerefMut;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::prelude::{FontMap, FontRequest, LocalizedText, TextLocalizer};

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
    localized: Query<'w, 's, &'static mut LocalizedText>,
    writer: TextUiWriter<'w, 's>,
    localizer: Res<'w, TextLocalizer>,
    fonts: Res<'w, FontMap>,
}

impl<'w, 's> TextEditor<'w, 's>
{
    /// Gets information for the first text span in a text block.
    ///
    /// Returns `Err` if the text block could not be found.
    pub fn root(&mut self, root_entity: impl Into<Entity>) -> Option<(&mut String, &mut TextFont, &mut Color)>
    {
        let root_entity: Entity = root_entity.into();
        self.span(root_entity, 0).map(|(_, t, f, c)| (t, f, c))
    }

    /// Gets information for a text span in a text block.
    ///
    /// Returns `Err` if the text span could not be found.
    pub fn span(
        &mut self,
        root_entity: impl Into<Entity>,
        span: usize,
    ) -> Option<(Entity, &mut String, &mut TextFont, &mut Color)>
    {
        let root_entity: Entity = root_entity.into();
        self.writer
            .get(root_entity, span)
            .map(|(e, _, t, f, c)| (e, t.into_inner(), f.into_inner(), c.into_inner().deref_mut()))
    }

    /// Overwrites the text on the first text span in a text block.
    ///
    /// See [`Self::write_span`].
    ///
    /// This is used in the [`write_text`](crate::write_text) helper macro.
    pub fn write<E: Debug>(
        &mut self,
        root_entity: impl Into<Entity>,
        writer: impl FnOnce(&mut String) -> Result<(), E>,
    ) -> bool
    {
        self.write_span(root_entity, 0, writer)
    }

    /// Overwrites the text on a text span in a text block.
    ///
    /// This will automatically localize the text and its font if the entity has the [`LocalizedText`] component.
    ///
    /// Returns `false` if the text span could not be accessed, if the writer fails, or if localization fails.
    ///
    /// This is used in the [`write_text_span`](crate::write_text_span) helper macro.
    pub fn write_span<E: Debug>(
        &mut self,
        root_entity: impl Into<Entity>,
        span: usize,
        writer: impl FnOnce(&mut String) -> Result<(), E>,
    ) -> bool
    {
        let root_entity: Entity = root_entity.into();
        let Some((_, _, mut text, mut text_font, _)) = self.writer.get(root_entity, span) else {
            tracing::warn!("failed writing to text span {span} of text block {root_entity:?}, entity not found");
            return false;
        };

        if let Ok(mut localized) = self.localized.get_mut(root_entity) {
            // Clear the localization string then write to it.
            localized.set_localization_for_span("", span);
            let localization_span = localized.localization_for_span_mut(span).unwrap();
            let result = match (writer)(&mut localization_span.template) {
                Ok(()) => true,
                Err(err) => {
                    tracing::warn!("failed writing to localized text span {span} of text block {root_entity:?}, \
                        write callback error {err:?}");
                    false
                }
            };
            // Localize the target string and its font.
            result && localized.localize_span(&self.localizer, &self.fonts, &mut text, &mut text_font.font, span)
        } else {
            text.clear();
            match (writer)(&mut *text) {
                Ok(()) => true,
                Err(err) => {
                    tracing::warn!("failed writing to text span {span} of text block {root_entity:?}, \
                        write callback error {err:?}");
                    false
                }
            }
        }
    }

    /// Sets the font on the first text span of a text block.
    ///
    /// See [`Self::set_span_font`].
    ///
    /// Returns `false` if the text span could not be accessed or if the font was not registered in [`FontMap`].
    pub fn set_font(&mut self, entity: impl Into<Entity>, font: impl Into<FontRequest>) -> bool
    {
        self.set_span_font(entity, 0, font)
    }

    /// Sets the font on a text span of a text block.
    ///
    /// If the entity has [`LocalizedText`] then the font will be automatically localized.
    ///
    /// Returns `false` if the text span could not be accessed or if the font was not registered in [`FontMap`].
    pub fn set_span_font(
        &mut self,
        root_entity: impl Into<Entity>,
        span: usize,
        font: impl Into<FontRequest>,
    ) -> bool
    {
        let root_entity: Entity = root_entity.into();
        let font = font.into();
        let Some((_, _, _, mut text_font, _)) = self.writer.get(root_entity, span) else {
            tracing::warn!("failed setting font {font:?} on text span {span} of text block {root_entity:?}, \
                root entity not found");
            return false;
        };
        let font = self.fonts.get(&font);
        if font == Handle::default() {
            tracing::warn!("failed setting font {font:?} on text span {span} of text block {root_entity:?}, font \
                not found in FontMap");
            return false;
        }

        // Set the requested font.
        text_font.font = font.clone();

        // Handle localization.
        if let Ok(mut localized) = self.localized.get_mut(root_entity) {
            // Prep text span if missing.
            if localized.localization_for_span_mut(span).is_none() {
                localized.set_localization_for_span("", span);
            }

            // Update the font backup to the requested font.
            let loc_span = localized.localization_for_span_mut(span).unwrap();
            loc_span.set_font_backup(font.clone());

            // If text has not been localized yet, then early-out since we don't know what language is needed.
            // - We assume that when the text is eventually localized, the font will be properly updated via
            //   `LocalizedText::localize_span`.
            if loc_span.lang().is_none() {
                return true;
            }

            // Update the font.
            loc_span.update_font(&self.fonts, &mut text_font.font);
        }

        true
    }

    /// Sets the font size on the first text span of a text block.
    pub fn set_font_size(&mut self, entity: impl Into<Entity>, size: f32)
    {
        self.set_span_font_size(entity, 0, size);
    }

    /// Sets the font size on a text span of a text block.
    pub fn set_span_font_size(&mut self, root_entity: impl Into<Entity>, span: usize, size: f32)
    {
        let root_entity: Entity = root_entity.into();
        let Some((_, _, _, mut text_font, _)) = self.writer.get(root_entity, span) else {
            tracing::warn!("failed setting font size {size:?} on text span {span} of text block {root_entity:?}, \
                root entity not found");
            return;
        };
        text_font.font_size = size;
    }

    /// Sets the font color on the first text span of a text block.
    pub fn set_font_color(&mut self, root_entity: impl Into<Entity>, color: Color)
    {
        self.set_span_font_color(root_entity, 0, color);
    }

    /// Sets the font color on a text span of a text block.
    pub fn set_span_font_color(&mut self, root_entity: impl Into<Entity>, span: usize, color: Color)
    {
        let root_entity: Entity = root_entity.into();
        let Some((_, _, _, _, mut text_color)) = self.writer.get(root_entity, span) else {
            tracing::warn!("failed setting font color {color:?} on text span {span} of text block {root_entity:?}, \
                root entity not found");
            return;
        };
        *text_color = TextColor(color);
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

/// Helper for writing text to a text span with a [`TextEditor`].
///
/// Example
/*
```rust
fn example(mut commands: Commands, mut text_editor: TextEditor)
{
    let entity = commands.spawn(TextBundle::default()).id();

    // Macro call:
    write_text_span!(text_editor, entity, 0, "Count: {}", 42);

    // Expands to:
    text_editor.write_span(entity, 0, |text| write!(text, "Count: {}", 42));
}
```
*/
#[macro_export]
macro_rules! write_text_span {
    ($editor: ident, $entity: expr, $span: expr, $($arg:tt)*) => {{
        $editor.write_span($entity, $span, |text| write!(text, $($arg)*))
    }};
}

//-------------------------------------------------------------------------------------------------------------------

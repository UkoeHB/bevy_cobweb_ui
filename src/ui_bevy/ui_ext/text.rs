use bevy::prelude::*;
use bevy::text::{BreakLineOn, TextLayoutInfo};
use bevy::ui::widget::TextFlags;
use bevy::ui::ContentSize;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

const TEXT_LINE_DEFAULT_TEXT: &str = "[text line]";

//-------------------------------------------------------------------------------------------------------------------

fn insert_text_line(
    In((entity, mut line)): In<(Entity, TextLine)>,
    mut commands: Commands,
    localizer: Res<TextLocalizer>,
    font_map: Res<FontMap>,
    color: Query<&TextLineColor>,
    mut localized: Query<&mut LocalizedText>,
)
{
    // Prep color.
    let color = color
        .get(entity)
        .map(|c| c.0)
        .unwrap_or_else(|_| TextLine::default_font_color());

    // Get font.
    let mut font = line.font.map(|f| font_map.get(&f)).unwrap_or_default();

    // Prep localization.
    // - We need to manually localize inserted text in case the text line is hot reloaded into an entity that
    //   already has Text (i.e. because auto-localization won't occur).
    if line.text.as_str() != TEXT_LINE_DEFAULT_TEXT {
        if let Ok(mut localized) = localized.get_mut(entity) {
            localized.set_localization(line.text.as_str());
            //todo: what happens if line.font is None? it should use bevy's default font
            localized.localization_mut().set_font_backup(font.clone());
            localized.localize(&localizer, &font_map, &mut line.text, &mut font);
        }
    }

    // Set up text.
    let mut text = Text::from_section(line.text, TextStyle { font, font_size: line.size, color });
    text.justify = line.justify;
    text.linebreak_behavior = line.linebreak;

    // Add text to entity.
    let mut ec = commands.entity(entity);
    ec.try_insert((
        text,
        TextLayoutInfo::default(),
        TextFlags::default(),
        ContentSize::default(),
    ));
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up an entity with a [`Text`] component and one text section.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct TextLine
{
    /// The starting text string.
    #[reflect(default = "TextLine::default_text")]
    pub text: String,
    /// The font handle.
    ///
    /// Defaults to `sickle_ui`'s built-in `FiraSans-Medium` font.
    #[reflect(default = "TextLine::default_font")]
    pub font: Option<FontRequest>,
    /// The desired font size.
    #[reflect(default = "TextLine::default_font_size")]
    pub size: f32,
    /// The line's [`BreakLineOn`] behavior.
    ///
    /// Defaults to [`BreakLineOn::NoWrap`].
    #[reflect(default = "TextLine::default_line_break")]
    pub linebreak: BreakLineOn,
    /// The line's [`JustifyText`] behavior.
    ///
    /// Defaults to [`JustifyText::Left`].
    #[reflect(default = "TextLine::default_justify_text")]
    pub justify: JustifyText,
}

impl TextLine
{
    pub fn from_text(text: impl Into<String>) -> Self
    {
        Self { text: text.into(), ..default() }
    }

    pub fn with_font(mut self, font: impl Into<FontRequest>) -> Self
    {
        self.font = Some(font.into());
        self
    }

    fn default_text() -> String
    {
        TEXT_LINE_DEFAULT_TEXT.into()
    }

    fn default_font() -> Option<FontRequest>
    {
        Some(FontRequest::new("Fira Sans").medium())
    }

    fn default_font_size() -> f32
    {
        25.
    }

    fn default_font_color() -> Color
    {
        Color::WHITE
    }

    fn default_line_break() -> BreakLineOn
    {
        BreakLineOn::NoWrap
    }

    fn default_justify_text() -> JustifyText
    {
        JustifyText::Left
    }
}

impl Instruction for TextLine
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self), insert_text_line);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<(Text, TextLayoutInfo, TextFlags, ContentSize)>();
        });
    }
}

impl Default for TextLine
{
    fn default() -> Self
    {
        Self {
            text: Self::default_text(),
            font: Self::default_font(),
            size: Self::default_font_size(),
            linebreak: Self::default_line_break(),
            justify: Self::default_justify_text(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for setting the font size of a [`TextLine`] on an entity.
//todo: hook this up to TextLine or find a better abstraction
#[derive(Reflect, Component, Default, Debug, Copy, Clone, PartialEq)]
pub struct TextLineSize(pub f32);

impl Instruction for TextLineSize
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(
            (entity, self.0),
            |In((id, size)): In<(Entity, f32)>, mut editor: TextEditor| {
                editor.set_font_size(id, size);
            },
        );
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        Instruction::apply(Self(TextLine::default_font_size()), entity, world);
    }
}

impl ThemedAttribute for TextLineSize
{
    type Value = f32;
    fn construct(value: Self::Value) -> Self
    {
        TextLineSize(value)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for setting the color of a [`TextLine`] on an entity.
#[derive(Reflect, Component, Default, Debug, Copy, Clone, PartialEq)]
pub struct TextLineColor(pub Color);

impl Instruction for TextLineColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(
            (entity, self.0),
            |In((id, color)): In<(Entity, Color)>, mut editor: TextEditor| {
                editor.set_font_color(id, color);
            },
        );
        world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        Instruction::apply(Self(TextLine::default_font_color()), entity, world);
    }
}

impl ThemedAttribute for TextLineColor
{
    type Value = Color;
    fn construct(value: Self::Value) -> Self
    {
        TextLineColor(value)
    }
}

impl ResponsiveAttribute for TextLineColor {}
impl AnimatableAttribute for TextLineColor {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiTextExtPlugin;

impl Plugin for UiTextExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<TextLine>()
            // IMPORTANT: This must be added after TextLine so the line size will overwrite TextLine defaults.
            .register_themed::<TextLineSize>()
            // IMPORTANT: This must be added after TextLine so the line color will overwrite TextLine defaults.
            .register_animatable::<TextLineColor>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

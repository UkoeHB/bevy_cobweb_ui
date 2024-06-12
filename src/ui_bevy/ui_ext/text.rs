use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use bevy::ui::widget::TextFlags;
use bevy::ui::ContentSize;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn insert_text_line(
    In((entity, line)): In<(Entity, TextLine)>,
    mut commands: Commands,
    mut font_map: ResMut<FontMap>,
    color: Query<&TextLineColor>,
)
{
    let color = color
        .get(entity)
        .map(|c| c.0)
        .unwrap_or_else(|_| TextLine::default_font_color());
    let mut ec = commands.entity(entity);
    ec.try_insert((
        line.as_text(&mut font_map, color),
        TextLayoutInfo::default(),
        TextFlags::default(),
        ContentSize::default(),
    ));
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up an entity with a [`Text`] component and one text section.
#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine
{
    /// The starting text string.
    #[reflect(default = "TextLine::default_text")]
    pub text: String,
    /// The font handle.
    #[reflect(default)]
    pub font: Option<String>,
    /// The desired font size.
    #[reflect(default = "TextLine::default_font_size")]
    pub size: f32,
}

impl TextLine
{
    fn default_text() -> String
    {
        "[text line]".into()
    }

    fn default_font_size() -> f32
    {
        25.
    }

    fn default_font_color() -> Color
    {
        Color::WHITE
    }

    fn as_text(self, font_map: &mut FontMap, color: Color) -> Text
    {
        Text::from_section(
            self.text,
            TextStyle {
                font: self.font.map(|f| font_map.get(&f)).unwrap_or_default(),
                font_size: self.size,
                color,
            },
        )
        .with_no_wrap()
    }
}

impl ApplyLoadable for TextLine
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let entity = ec.id();
        ec.commands().syscall((entity, self), insert_text_line);
    }
}

impl Default for TextLine
{
    fn default() -> Self
    {
        Self {
            text: Self::default_text(),
            font: None,
            size: Self::default_font_size(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable for setting the font size of a [`TextLine`] on an entity.
//todo: hook this up to TextLine or find a better abstraction
#[derive(Reflect, Component, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLineSize(pub f32);

impl ApplyLoadable for TextLineSize
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall(
            (id, self.0),
            |In((id, size)): In<(Entity, f32)>, mut editor: TextEditor| {
                let Some(style) = editor.style(id) else { return };
                style.font_size = size;
            },
        );
        ec.try_insert(self);
    }
}

impl ThemedAttribute for TextLineSize
{
    type Value = f32;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        TextLineSize(value).apply(ec);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable for setting the color of a [`TextLine`] on an entity.
#[derive(Reflect, Component, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLineColor(pub Color);

impl ApplyLoadable for TextLineColor
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall(
            (id, self.0),
            |In((id, color)): In<(Entity, Color)>, mut editor: TextEditor| {
                let Some(style) = editor.style(id) else { return };
                style.color = color;
            },
        );
        ec.try_insert(self);
    }
}

impl ThemedAttribute for TextLineColor
{
    type Value = Color;
    fn update(ec: &mut EntityCommands, value: Self::Value)
    {
        TextLineColor(value).apply(ec);
    }
}

impl ResponsiveAttribute for TextLineColor
{
    type Interactive = Interactive;
}
impl AnimatableAttribute for TextLineColor
{
    type Interactive = Interactive;
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiTextExtPlugin;

impl Plugin for UiTextExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_derived::<TextLine>()
            // IMPORTANT: This must be added after TextLine so the line size will overwrite TextLine defaults.
            .register_themed::<TextLineSize>()
            // IMPORTANT: This must be added after TextLine so the line color will overwrite TextLine defaults.
            .register_animatable::<TextLineColor>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

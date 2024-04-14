use std::collections::HashMap;

use bevy::{ecs::system::EntityCommands, prelude::*, text::TextLayoutInfo, ui::{widget::TextFlags, ContentSize}};
use bevy_cobweb::prelude::*;
use serde::{Serialize, Deserialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn insert_text_line(
    In((entity, line)): In<(Entity, TextLine)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_map: ResMut<FontMap>
){
    let mut ec = commands.entity(entity);
    ec.try_insert((
        line.as_text(&asset_server, &mut font_map),
        TextLayoutInfo::default(),
        TextFlags::default(),
        ContentSize::default(),
    ));
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to loaded fonts.
#[derive(Resource, Default)]
pub struct FontMap
{
    map: HashMap<String, Handle<Font>>,
}

impl FontMap
{
    fn get(&mut self, font: Option<String>, asset_server: &AssetServer) -> Handle<Font>
    {
        let Some(font) = font else { return Default::default() };
        let Some(entry) = self.map.get(&font)
        else
        {
            let entry = asset_server.load(&font);
            self.map.insert(font, entry.clone());
            return entry;
        };
        entry.clone()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up an entity with a [`Text`] component and one text section.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// The text color.
    #[reflect(default = "TextLine::default_font_color")]
    pub color: Color,
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

    fn as_text(self, asset_server: &AssetServer, font_map: &mut FontMap) -> Text
    {
        Text::from_section(
            self.text,
            TextStyle{
                font: font_map.get(self.font, asset_server),
                font_size: self.size,
                color: self.color
            }
        ).with_no_wrap()
    }
}

impl StyleToBevy for TextLine
{
    fn to_bevy(self, ec: &mut EntityCommands)
    {
        let entity = ec.id();
        ec.commands().syscall((entity, self), insert_text_line);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiTextExtPlugin;

impl Plugin for UiTextExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .init_resource::<FontMap>()
            .register_type::<TextLine>()
            .register_derived_style::<TextLine>()
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------
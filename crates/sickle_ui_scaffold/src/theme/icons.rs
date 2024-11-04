use std::char;

use bevy::prelude::*;

#[derive(Clone, Debug, Default, Reflect)]
pub enum IconData
{
    #[default]
    None,
    Image(String, Color),
    FontCodepoint(String, char, Color, f32),
    // TODO: add texture atlas config
}

impl IconData
{
    pub fn is_none(&self) -> bool
    {
        matches!(self, Self::None)
    }

    pub fn is_image(&self) -> bool
    {
        matches!(self, Self::Image(_, _))
    }

    pub fn is_codepoint(&self) -> bool
    {
        matches!(self, Self::FontCodepoint(_, _, _, _))
    }

    pub fn with_color(&self, color: Color) -> Self
    {
        match self {
            IconData::None => IconData::None,
            IconData::Image(path, _) => Self::Image(path.clone(), color),
            IconData::FontCodepoint(path, codepoint, _, size) => {
                Self::FontCodepoint(path.clone(), codepoint.clone(), color, size.clone())
            }
        }
    }

    pub fn with_size(&self, size: f32) -> Self
    {
        match self {
            IconData::None => IconData::None,
            IconData::Image(_, _) => self.clone(),
            IconData::FontCodepoint(path, codepoint, color, _) => {
                Self::FontCodepoint(path.clone(), codepoint.clone(), color.clone(), size)
            }
        }
    }

    pub fn with(&self, color: Color, size: f32) -> Self
    {
        match self {
            IconData::None => IconData::None,
            IconData::Image(path, _) => Self::Image(path.clone(), color),
            IconData::FontCodepoint(path, codepoint, _, _) => {
                Self::FontCodepoint(path.clone(), codepoint.clone(), color, size)
            }
        }
    }
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct CustomIconData
{
    pub name: String,
    pub data: IconData,
}

#[derive(Clone, Debug, Reflect)]
pub struct Icons
{
    pub arrow_right: IconData,
    pub bug_report: IconData,
    pub checkmark: IconData,
    pub chevron_left: IconData,
    pub chevron_right: IconData,
    pub close: IconData,
    pub error: IconData,
    pub exit_to_app: IconData,
    pub expand_less: IconData,
    pub expand_more: IconData,
    pub info: IconData,
    pub open_in_new: IconData,
    pub query_stats: IconData,
    pub radio_button_checked: IconData,
    pub radio_button_unchecked: IconData,
    pub redo: IconData,
    pub refresh: IconData,
    pub sync: IconData,
    pub troubleshoot: IconData,
    pub undo: IconData,
    pub warning: IconData,
    pub custom: Vec<CustomIconData>,
}

// TODO: create codepoint parser?
impl Default for Icons
{
    fn default() -> Self
    {
        Self {
            arrow_right: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5DF}',
                Color::WHITE,
                12.,
            ),
            bug_report: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E868}',
                Color::WHITE,
                12.,
            ),
            checkmark: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CA}',
                Color::WHITE,
                12.,
            ),
            chevron_left: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CB}',
                Color::WHITE,
                12.,
            ),
            chevron_right: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CC}',
                Color::WHITE,
                12.,
            ),
            close: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CD}',
                Color::WHITE,
                12.,
            ),
            error: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E000}',
                Color::WHITE,
                12.,
            ),
            exit_to_app: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E879}',
                Color::WHITE,
                12.,
            ),
            expand_less: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CE}',
                Color::WHITE,
                12.,
            ),
            expand_more: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5CF}',
                Color::WHITE,
                12.,
            ),
            info: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E88E}',
                Color::WHITE,
                12.,
            ),
            open_in_new: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E89E}',
                Color::WHITE,
                12.,
            ),
            query_stats: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E4FC}',
                Color::WHITE,
                12.,
            ),
            radio_button_checked: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E837}',
                Color::WHITE,
                12.,
            ),
            radio_button_unchecked: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E836}',
                Color::WHITE,
                12.,
            ),
            redo: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E15A}',
                Color::WHITE,
                12.,
            ),
            refresh: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E5D5}',
                Color::WHITE,
                12.,
            ),
            sync: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E627}',
                Color::WHITE,
                12.,
            ),
            troubleshoot: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E1D2}',
                Color::WHITE,
                12.,
            ),
            undo: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E166}',
                Color::WHITE,
                12.,
            ),
            warning: IconData::FontCodepoint(
                "embedded://bevy_cobweb_ui/fonts/MaterialIcons-Regular.ttf".into(),
                '\u{E002}',
                Color::WHITE,
                12.,
            ),
            custom: Vec::new(),
        }
    }
}

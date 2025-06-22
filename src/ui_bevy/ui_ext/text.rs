use bevy::prelude::*;
use bevy::text::{ComputedTextBlock, LineBreak, TextLayoutInfo};
use bevy::ui::widget::TextNodeFlags;
use bevy::ui::ContentSize;
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

const TEXT_LINE_DEFAULT_TEXT: &str = "[[text line]]";

//-------------------------------------------------------------------------------------------------------------------

fn insert_text_line(
    In((entity, mut line)): In<(Entity, TextLine)>,
    mut commands: Commands,
    localizer: Res<TextLocalizer>,
    default_font: Res<DefaultFont>,
    font_map: Res<FontMap>,
    mut localized: Query<&mut LocalizedText>,
)
{
    // Get font.
    let request = line.font.as_ref().unwrap_or(&*default_font);
    let mut font = font_map.get(request);

    // Prep localization.
    // - We need to manually localize inserted text in case the text line is hot reloaded into an entity that
    //   already has Text (i.e. because auto-localization won't occur).
    // TODO: future localization rework should make this no longer necessary
    if line.text.as_str() != TEXT_LINE_DEFAULT_TEXT {
        if let Ok(mut localized) = localized.get_mut(entity) {
            localized.set_localization(line.text.as_str());
            //todo: what happens if line.font is None? it should use bevy's default font
            localized.localization_mut().set_font_backup(font.clone());
            localized.localize(&localizer, &font_map, &mut line.text, &mut font);
        }
    }

    // Add text to entity.
    let Ok(mut ec) = commands.get_entity(entity) else { return };
    ec.try_insert((
        Text(line.text),
        TextLayout { justify: line.justify, linebreak: line.linebreak },
        TextFont { font, font_size: line.size, ..default() },
    ));
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets up an entity with a [`Text`] component and one text span.
///
/// The default font is "Fira Sans Medium" with size `25.0`.
#[derive(Reflect, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextLine
{
    /// The starting text string.
    #[reflect(default = "TextLine::default_text")]
    pub text: String,
    /// The font handle.
    ///
    /// See [`DefaultFont`] for the font used if this is not set.
    #[reflect(default)]
    pub font: Option<FontRequest>,
    /// The desired font size.
    ///
    /// Defaults to `25.0`.
    #[reflect(default = "TextLine::default_font_size")]
    pub size: f32,
    /// The line's [`LineBreak`] behavior.
    ///
    /// Defaults to [`LineBreak::NoWrap`].
    #[reflect(default = "TextLine::default_line_break")]
    pub linebreak: LineBreak,
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

    fn default_font_size() -> f32
    {
        25.
    }

    fn default_line_break() -> LineBreak
    {
        LineBreak::NoWrap
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
        // NOTE: Can't use remove_with_requires because removing and reinserting ComputedNodeTarget breaks UI.
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<(
                Text,
                TextLayout,
                TextFont,
                TextColor,
                TextNodeFlags,
                ContentSize,
                ComputedTextBlock,
                TextLayoutInfo,
            )>();
        });
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
            linebreak: Self::default_line_break(),
            justify: Self::default_justify_text(),
        }
    }
}

impl StaticAttribute for TextLine
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for setting the font size of a [`TextLine`] on an entity.
///
/// If ordered before a [`TextLine`] instruction, then this instruction will be overridden by the
/// [`TextLine::size`] field.
//todo: hook this up to TextLine or find a better abstraction
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct TextLineSize(pub f32);

impl Instruction for TextLineSize
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(
            (entity, self.0),
            |In((id, size)): In<(Entity, f32)>, mut c: Commands, mut editor: TextEditor| {
                let Some((_, text_font, _)) = editor.root(id) else {
                    if let Ok(mut ec) = c.get_entity(id) {
                        ec.try_insert((
                            Text(TextLine::default_text()),
                            TextFont { font_size: size, ..default() },
                        ));
                    }
                    return;
                };
                text_font.font_size = size;
            },
        );
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // NOTE: Can't use remove_with_requires because removing and reinserting ComputedNodeTarget breaks UI.
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<(
                Text,
                TextLayout,
                TextFont,
                TextColor,
                TextNodeFlags,
                ContentSize,
                ComputedTextBlock,
                TextLayoutInfo,
            )>();
        });
    }
}

impl StaticAttribute for TextLineSize
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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct TextLineColor(pub Color);

impl Instruction for TextLineColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall(
            (entity, self.0),
            |In((id, color)): In<(Entity, Color)>, mut c: Commands, mut editor: TextEditor| {
                let Some((_, _, text_color)) = editor.root(id) else {
                    if let Ok(mut ec) = c.get_entity(id) {
                        ec.try_insert((Text(TextLine::default_text()), TextColor(color)));
                    }
                    return;
                };
                *text_color = color;
            },
        );
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // NOTE: Can't use remove_with_requires because removing and reinserting ComputedNodeTarget breaks UI.
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<(
                Self,
                Text,
                TextLayout,
                TextFont,
                TextColor,
                TextNodeFlags,
                ContentSize,
                ComputedTextBlock,
                TextLayoutInfo,
            )>();
        });
    }
}

impl StaticAttribute for TextLineColor
{
    type Value = Color;
    fn construct(value: Self::Value) -> Self
    {
        TextLineColor(value)
    }
}

impl ResponsiveAttribute for TextLineColor {}
impl AnimatedAttribute for TextLineColor
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let color = world.get::<Self>(entity)?;
        Some(color.0)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reimplementation of [`TextShadow`].
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct TextShadowImpl
{
    pub offset: Vec2,
    pub color: Color,
}

impl cob_sickle_math::Lerp for TextShadowImpl
{
    fn lerp(&self, to: Self, t: f32) -> Self
    {
        Self {
            offset: self.offset.lerp(to.offset, t),
            color: self.color.lerp(to.color, t),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for setting a group of text shadows on an entity.
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Deref, DerefMut)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct TextShadowGroup(pub SmallVec<[TextShadowImpl; 4]>);

impl TextShadowGroup
{
    pub fn new(shadow: TextShadowImpl) -> Self
    {
        Self(SmallVec::from_elem(shadow, 1))
    }

    pub fn from_slice(shadows: &[TextShadowImpl]) -> Self
    {
        Self(SmallVec::from_slice(shadows))
    }
}

impl Instruction for TextShadowGroup
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

impl cob_sickle_math::Lerp for TextShadowGroup
{
    fn lerp(&self, to: Self, t: f32) -> Self
    {
        let mut group = self.clone();
        group.iter_mut().zip(to.iter()).for_each(|(grp, to)| {
            *grp = grp.lerp(*to, t);
        });
        group
    }
}

impl StaticAttribute for TextShadowGroup
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

impl ResponsiveAttribute for TextShadowGroup {}
impl AnimatedAttribute for TextShadowGroup
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let shadow = world.get::<Self>(entity).cloned()?;
        Some(shadow)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Re-exports [`TextOutline`].
///
/// Implements instruction for adding an outline around text.
///
/// Re-expo
pub use bevy_slow_text_outline::prelude::TextOutline;

impl Instruction for TextOutline
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

impl StaticAttribute for TextOutline
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

impl ResponsiveAttribute for TextOutline {}
impl AnimatedAttribute for TextOutline
{
    fn get_value(entity: Entity, world: &World) -> Option<Self::Value>
    {
        let outline = world.get::<Self>(entity).copied()?;
        Some(outline)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiTextExtPlugin;

impl Plugin for UiTextExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_static::<TextLine>()
            .register_static::<TextLineSize>()
            .register_animatable::<TextLineColor>()
            .register_animatable::<TextShadowGroup>()
            .register_animatable::<TextOutline>()
            .register_type::<TextShadowImpl>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

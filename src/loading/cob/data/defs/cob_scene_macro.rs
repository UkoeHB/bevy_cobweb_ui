use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{recognize, value};
use nom::multi::many0_count;
use nom::sequence::{terminated, tuple};
use nom::{IResult, Parser};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn try_parse_scene_group(
    opener: char,
    closer: char,
    layer_indent: usize,
    start_fill: CobFill,
    content: Span,
) -> Result<(Option<(CobFill, Vec<CobSceneLayerEntry>, CobFill)>, CobFill, Span), SpanError>
{
    let Ok((remaining, _)) = char::<_, ()>(opener).parse(content) else { return Ok((None, start_fill, content)) };

    // Get content indent from first item_fill.
    let (mut item_fill, mut remaining) = CobFill::parse(remaining);
    let Some(content_indent) = item_fill.ends_newline_then_num_spaces() else {
        // Check if there is in fact no content.
        let Ok((remaining, _)) = char::<_, ()>(closer).parse(remaining) else {
            tracing::warn!("failed parsing scene group at {}; group opener '{opener}' not on a separate line from the first item",
                get_location(remaining));
            return Err(span_verify_error(remaining));
        };
        let (post_fill, remaining) = CobFill::parse(remaining);
        return Ok((Some((start_fill, vec![], item_fill)), post_fill, remaining));
    };
    if content_indent == 0 {
        tracing::warn!("failed parsing scene group at {}; content indent is zero",
            get_location(remaining));
        return Err(span_verify_error(remaining));
    }

    // Collect entries.
    let mut entries = vec![];
    let end_fill = loop {
        // Note: this will properly handle the case where content_indent <= layer_indent.
        match rc(remaining, move |rm| {
            CobSceneLayerEntry::try_parse(layer_indent, content_indent, item_fill, rm)
        })? {
            (Some(entry), next_fill, after_entry) => {
                entries.push(entry);
                item_fill = next_fill;
                remaining = after_entry;
            }
            (None, end_fill, after_end) => {
                remaining = after_end;
                break end_fill;
            }
        }
    };

    let (remaining, _) = char(closer).parse(remaining)?;
    let (post_fill, remaining) = CobFill::parse(remaining);
    Ok((Some((start_fill, entries, end_fill)), post_fill, remaining))
}

//-------------------------------------------------------------------------------------------------------------------

/// Command that can be used in scene macro invocations to rearrange loadables in the macro's scene content.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum CobSceneMacroCommandType
{
    /// E.g. `^BorderColor`
    #[default]
    MoveToTop,
    /// E.g. `!BorderColor`
    MoveToBottom,
    /// E.g. `-BorderColor`
    Remove,
}

impl CobSceneMacroCommandType
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::MoveToTop => {
                writer.write_bytes("^".as_bytes())?;
            }
            Self::MoveToBottom => {
                writer.write_bytes("!".as_bytes())?;
            }
            Self::Remove => {
                writer.write_bytes("-".as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn parse_nomlike(content: Span) -> IResult<Span, Self>
    {
        alt((
            value(Self::MoveToTop, char('^')),
            value(Self::MoveToBottom, char('!')),
            value(Self::Remove, char('-')),
        ))
        .parse(content)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CobSceneMacroCommand
{
    pub start_fill: CobFill,
    pub command_type: CobSceneMacroCommandType,
    pub id: CobLoadableIdentifier,
}

impl CobSceneMacroCommand
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        self.command_type.write_to(writer)?;
        self.id.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((remaining, command_type)) = CobSceneMacroCommandType::parse_nomlike(content) else {
            return Ok((None, start_fill, content));
        };
        let (id, remaining) = match CobLoadableIdentifier::parse(remaining) {
            Ok((id, remaining)) => (id, remaining),
            Err(err) => {
                tracing::warn!("failed parsing cob scene macro command at {}; id is invalid: {err:?}",
                    get_location(content).as_str());
                return Err(span_verify_error(content));
            }
        };
        let (post_fill, remaining) = CobFill::parse(remaining);
        Ok((Some(Self { start_fill, command_type, id }), post_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        // No fill in the command type
        self.id.recover_fill(&other.id);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Scene macro name must be `+` followed by a loadable identifier. Names do not include `a::b::` path segments.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CobSceneMacroName
{
    pub name: SmolStr,
}

impl CobSceneMacroName
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("+".as_bytes())?;
        writer.write_bytes(self.name.as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (post_symbol, _) = char('+').parse(content)?;
        recognize(anything_identifier)
            .parse(post_symbol)
            .map(|(r, k)| (Self { name: SmolStr::from(*k.fragment()) }, r))
    }

    pub fn as_str(&self) -> &str
    {
        self.name.as_str()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Scene macro paths must be a series of snake-case identifiers separated by `::`. E.g. `+a::b::my_constant`.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CobSceneMacroPath
{
    pub path: SmolStr,
}

impl CobSceneMacroPath
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("+".as_bytes())?;
        writer.write_bytes(self.path.as_bytes())?;
        Ok(())
    }

    pub fn parse(content: Span) -> Result<(Self, Span), SpanError>
    {
        let (post_symbol, _) = char('+').parse(content)?;
        recognize(tuple((
            // Extensions
            many0_count(terminated(snake_identifier, tag("::"))),
            // Constant name
            anything_identifier,
        )))
        .parse(post_symbol)
        .map(|(r, k)| (Self { path: SmolStr::from(*k.fragment()) }, r))
    }

    pub fn as_str(&self) -> &str
    {
        self.path.as_str()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Scene group for scene macro definitions.
#[derive(Debug, Clone, PartialEq)]
pub struct CobSceneMacroValue
{
    /// Fill before opening `\`.
    pub start_fill: CobFill,
    pub entries: Vec<CobSceneLayerEntry>,
    /// Fill before ending `\`.
    pub end_fill: CobFill,
}

impl CobSceneMacroValue
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("\\".as_bytes())?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        self.end_fill.write_to(writer)?;
        writer.write_bytes("\\".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        try_parse_scene_group('\\', '\\', 0, start_fill, content).map(|(r, post_fill, remaining)| {
            (
                r.map(|(start_fill, entries, end_fill)| Self { start_fill, entries, end_fill }),
                post_fill,
                remaining,
            )
        })
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        for (entry, other_entry) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other_entry);
        }
        self.end_fill.recover(&other.end_fill);
    }

    pub fn resolve(&mut self, resolver: &mut CobResolver, resolve_mode: SceneResolveMode) -> Result<(), String>
    {
        CobSceneLayer::resolve_entries_impl("", &mut self.entries, resolver, resolve_mode)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Scene group for scene macro invocations.
#[derive(Debug, Clone, PartialEq)]
pub struct CobSceneMacroContainer
{
    pub entries: Vec<CobSceneLayerEntry>,
    /// Fill before ending `}`.
    pub end_fill: CobFill,
}

impl CobSceneMacroContainer
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("{".as_bytes())?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        self.end_fill.write_to(writer)?;
        writer.write_bytes("}".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(layer_indent: usize, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        try_parse_scene_group('{', '}', layer_indent, CobFill::default(), content).map(
            |(r, post_fill, remaining)| {
                (
                    r.map(|(_, entries, end_fill)| Self { entries, end_fill }),
                    post_fill,
                    remaining,
                )
            },
        )
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        for (entry, other_entry) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other_entry);
        }
        self.end_fill.recover(&other.end_fill);
    }

    pub fn resolve(&mut self, resolver: &mut CobResolver, resolve_mode: SceneResolveMode) -> Result<(), String>
    {
        CobSceneLayer::resolve_entries_impl("", &mut self.entries, resolver, resolve_mode)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobSceneMacroDef
{
    pub start_fill: CobFill,
    pub name: CobSceneMacroName,
    pub pre_eq_fill: CobFill,
    /// The value is expected to handle its own fill.
    pub value: CobSceneMacroValue,
}

impl CobSceneMacroDef
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        self.name.write_to(writer)?;
        self.pre_eq_fill.write_to(writer)?;
        writer.write_bytes("=".as_bytes())?;
        self.value.write_to(writer)?;

        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((name, remaining)) = rc(content, |c| CobSceneMacroName::parse(c)) else {
            return Ok((None, start_fill, content));
        };
        let (pre_eq_fill, remaining) = CobFill::parse(remaining);
        let (remaining, _) = char('=').parse(remaining)?;
        let (value_fill, remaining) = CobFill::parse(remaining);
        let (Some(value), end_fill, remaining) = CobSceneMacroValue::try_parse(value_fill, remaining)? else {
            tracing::warn!("scene macro definition is invalid at {}", get_location(content).as_str());
            return Err(span_verify_error(content));
        };

        let def = Self { start_fill, name, pre_eq_fill, value };
        Ok((Some(def), end_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        // Name has no fill
        self.pre_eq_fill.recover(&other.pre_eq_fill);
        self.value.recover_fill(&other.value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobSceneMacroCall
{
    pub start_fill: CobFill,
    pub path: CobSceneMacroPath,
    // No fill between path and container.
    pub container: CobSceneMacroContainer,
}

impl CobSceneMacroCall
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        self.path.write_to(writer)?;
        self.container.write_to(writer)?;

        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((path, remaining)) = rc(content, |c| CobSceneMacroPath::parse(c)) else {
            return Ok((None, start_fill, content));
        };
        let (pre_container_fill, remaining) = CobFill::parse(remaining);

        // Scene macro invocations may not have fill before the opening brace.
        if pre_container_fill.len() != 0 {
            tracing::warn!("failed parsing scene macro invocation at {}; there is whitespace between the macro name and \
                its container",
                get_location(remaining));
            return Err(span_verify_error(remaining));
        }

        let layer_indent = start_fill.ends_newline_then_num_spaces().unwrap_or(0);

        let (Some(container), end_fill, remaining) = CobSceneMacroContainer::try_parse(layer_indent, remaining)?
        else {
            tracing::warn!("scene macro invocation is invalid at {}", get_location(content).as_str());
            return Err(span_verify_error(content));
        };

        let def = Self { start_fill, path, container };
        Ok((Some(def), end_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        // Name has no fill.
        self.container.recover_fill(&other.container);
    }

    pub fn resolve(
        &mut self,
        resolver: &mut CobResolver,
        resolve_mode: SceneResolveMode,
    ) -> Result<Vec<CobSceneLayerEntry>, String>
    {
        // Resolve the content.
        self.container.resolve(resolver, resolve_mode)?;

        // Expand the macro.
        resolver.scenes.scene_macros.expand(self)
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::Deref;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, success};
use nom::sequence::terminated;
use nom::Parser;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CobSceneNodeName(pub SmolStr);

impl CobSceneNodeName
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("\"".as_bytes())?;
        writer.write_bytes(self.as_bytes())?;
        writer.write_bytes("\"".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(content: Span) -> Result<(Option<Self>, Span), SpanError>
    {
        let Ok((remaining, _)) = char::<_, ()>('\"').parse(content) else { return Ok((None, content)) };
        // Allows snake identifiers that may start with numbers and empty identifiers.
        let Ok((remaining, name)) = terminated(
            alt((map(numerical_snake_identifier, |i| *i.fragment()), success(""))),
            tag("\""),
        )
        .parse(remaining) else {
            tracing::warn!("failed parsing scene node name at {}; name is not snake-case (e.g. a_b_c)",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        };

        Ok((Some(Self(SmolStr::from(name))), remaining))
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Full loadable.
#[derive(Debug, Clone, PartialEq)]
pub enum CobSceneLayerEntry
{
    LoadableMacroCall(CobLoadableMacroCall),
    Loadable(CobLoadable),
    SceneMacroCall(CobSceneMacroCall),
    Layer(CobSceneLayer),
    /// This is the `..'node_name'` and `..*` syntax.
    SceneMacroParam(CobSceneMacroParam),
}

impl CobSceneLayerEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::LoadableMacroCall(entry) => {
                entry.write_to(writer)?;
            }
            Self::Loadable(entry) => {
                entry.write_to(writer)?;
            }
            Self::SceneMacroCall(entry) => {
                entry.write_to(writer)?;
            }
            Self::Layer(entry) => {
                entry.write_to(writer)?;
            }
            Self::SceneMacroParam(entry) => {
                entry.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(
        parent_indent: usize,
        expected_indent: usize,
        fill: CobFill,
        content: Span,
    ) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        // If no indent but not eof, warn and error, else return none.
        let Some(indent) = fill.ends_newline_then_num_spaces() else {
            if content.fragment().len() == 0 {
                // End-of-file
                return Ok((None, fill, content));
            }
            tracing::warn!("failed parsing scene at {}; encountered something that isn't on a separate line",
                get_location(content));
            return Err(span_verify_error(content));
        };

        // The next item isn't on the active layer.
        if indent <= parent_indent {
            return Ok((None, fill, content));
        }

        // Warn and allow if indent is != expected
        if indent != expected_indent {
            tracing::warn!("encountered scene item that isn't aligned with other items in the same layer at {}; \
                item indent: {}, expected: {}", get_location(content).as_str(), indent, expected_indent);
        }

        // Parse item.
        // - Parse loadable macro calls before loadables to avoid conflicts.
        let fill = match rc(content, move |c| CobLoadableMacroCall::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::LoadableMacroCall(item)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobLoadable::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::Loadable(item)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobSceneMacroCall::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::SceneMacroCall(item)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobSceneLayer::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::Layer(item)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobSceneMacroParam::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::SceneMacroParam(item)), fill, remaining)),
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::LoadableMacroCall(entry), Self::LoadableMacroCall(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::Loadable(entry), Self::Loadable(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::SceneMacroCall(entry), Self::SceneMacroCall(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::Layer(entry), Self::Layer(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::SceneMacroParam(entry), Self::SceneMacroParam(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobSceneLayer
{
    /// Fill before the layer name.
    ///
    /// Whitespace between the name and most recent newline is used to control scene layer depth.
    pub name_fill: CobFill,
    pub name: CobSceneNodeName,
    pub entries: Vec<CobSceneLayerEntry>,
}

impl CobSceneLayer
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.name_fill.write_to_or_else(writer, "\n")?;
        self.name.write_to(writer)?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    /// `indent` should be the indent of this layer's id
    pub fn try_parse(name_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let (Some(name), remaining) = CobSceneNodeName::try_parse(content)? else {
            return Ok((None, name_fill, content));
        };

        // Extract layer indent
        let Some(layer_indent) = name_fill.ends_newline_then_num_spaces() else {
            tracing::warn!("failed parsing scene at {}; node name is not on a separate line from the previous item",
                get_location(remaining));
            return Err(span_verify_error(content));
        };

        // Get content indent from first item_fill.
        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let Some(content_indent) = item_fill.ends_newline_then_num_spaces() else {
            if remaining.fragment().len() == 0 {
                // End-of-file
                return Ok((Some(Self { name_fill, name, entries: vec![] }), item_fill, remaining));
            }
            tracing::warn!("failed parsing scene at {}; first item after a node name isn't on a separate line",
                get_location(remaining));
            return Err(span_verify_error(remaining));
        };

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

        Ok((Some(Self { name_fill, name, entries }), end_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.name_fill.recover(&other.name_fill);
        for (entry, other) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobScenes
{
    pub start_fill: CobFill,
    pub scenes: Vec<CobSceneLayer>,
}

impl CobScenes
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("#scenes".as_bytes())?;
        for layer in self.scenes.iter() {
            layer.write_to(writer)?;
        }
        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((remaining, _)) = tag::<_, _, ()>("#scenes").parse(content) else {
            return Ok((None, start_fill, content));
        };

        if start_fill.len() != 0 && !start_fill.ends_with_newline() {
            tracing::warn!("failed parsing scenes section at {} that doesn't start on newline",
                get_location(remaining).as_str());
            return Err(span_verify_error(remaining));
        }

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut scenes = vec![];

        let end_fill = loop {
            let item_depth = item_fill.ends_newline_then_num_spaces();
            match rc(remaining, move |rm| CobSceneLayer::try_parse(item_fill, rm))? {
                (Some(entry), next_fill, after_entry) => {
                    if item_depth != Some(0) {
                        tracing::warn!("failed parsing scene at {}; scene is assessed to be on base layer \
                            but doesn't start with a newline", get_location(remaining).as_str());
                        return Err(span_verify_error(remaining));
                    }
                    scenes.push(entry);
                    item_fill = next_fill;
                    remaining = after_entry;
                }
                (None, end_fill, after_end) => {
                    remaining = after_end;
                    break end_fill;
                }
            }
        };

        let scenes = CobScenes { start_fill, scenes };
        Ok((Some(scenes), end_fill, remaining))
    }
}

// Parsing: layers cannot contain scene macro params, and layer entries cannot contain macro params.
// - TODO: evaluate if this is useful, the perf cost to validate is non-negligible if done by re-traversing the
//   data

//-------------------------------------------------------------------------------------------------------------------

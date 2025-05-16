use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, success};
use nom::sequence::terminated;
use nom::Parser;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobSceneNodeName(pub SmolStr);

impl CobSceneNodeName
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("\"".as_bytes())?;
        writer.write_bytes(self.0.as_bytes())?;
        writer.write_bytes("\"".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(content: Span) -> Result<(Option<Self>, Span), SpanError>
    {
        let Ok((remaining, _)) = char::<_, ()>('\"').parse(content) else { return Ok((None, content)) };
        // Allows arbitrary identifiers and empty identifiers.
        let Ok((remaining, name)) = terminated(
            alt((map(anything_identifier, |i| *i.fragment()), success(""))),
            tag("\""),
        )
        .parse(remaining) else {
            tracing::warn!("failed parsing scene node name at {}; name is not snake-case (e.g. a_b_c)",
                get_location(content).as_str());
            return Err(span_verify_error(content));
        };

        Ok((Some(Self(SmolStr::from(name))), remaining))
    }

    pub fn as_str(&self) -> &str
    {
        self.0.as_str()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Full loadable.
#[derive(Debug, Clone, PartialEq)]
pub enum CobSceneLayerEntry
{
    Loadable(CobLoadable),
    #[cfg(feature = "full")]
    SceneMacroCall(CobSceneMacroCall),
    #[cfg(feature = "full")]
    SceneMacroCommand(CobSceneMacroCommand),
    Layer(CobSceneLayer),
}

impl CobSceneLayerEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Loadable(entry) => {
                entry.write_to(writer)?;
            }
            #[cfg(feature = "full")]
            Self::SceneMacroCall(entry) => {
                entry.write_to(writer)?;
            }
            #[cfg(feature = "full")]
            Self::SceneMacroCommand(entry) => {
                entry.write_to(writer)?;
            }
            Self::Layer(entry) => {
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
        let fill = match rc(content, move |c| CobLoadable::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::Loadable(item)), fill, remaining)),
            (None, fill, _) => fill,
        };
        #[cfg(feature = "full")]
        let fill = match rc(content, move |c| CobSceneMacroCall::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::SceneMacroCall(item)), fill, remaining)),
            (None, fill, _) => fill,
        };
        #[cfg(feature = "full")]
        let fill = match rc(content, move |c| CobSceneMacroCommand::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::SceneMacroCommand(item)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobSceneLayer::try_parse(fill, c))? {
            (Some(item), fill, remaining) => return Ok((Some(Self::Layer(item)), fill, remaining)),
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Loadable(entry), Self::Loadable(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            #[cfg(feature = "full")]
            (Self::SceneMacroCall(entry), Self::SceneMacroCall(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            #[cfg(feature = "full")]
            (Self::SceneMacroCommand(entry), Self::SceneMacroCommand(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::Layer(entry), Self::Layer(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            _ => (),
        }
    }

    #[cfg(feature = "full")]
    pub fn resolve(
        &mut self,
        resolver: &mut CobResolver,
        resolve_mode: SceneResolveMode,
    ) -> Result<Option<Vec<CobSceneLayerEntry>>, String>
    {
        match self {
            Self::Loadable(entry) => match resolve_mode {
                SceneResolveMode::OneLayerSceneOnly | SceneResolveMode::SceneOnly => (),
                SceneResolveMode::Full => {
                    entry.resolve(&resolver.loadables)?;
                }
            },
            Self::SceneMacroCall(entry) => {
                // Upgrade resolve mode to ensure macro call gets resolved properly.
                let resolve_mode = if resolve_mode == SceneResolveMode::OneLayerSceneOnly {
                    SceneResolveMode::SceneOnly
                } else {
                    resolve_mode
                };
                return entry.resolve(resolver, resolve_mode).map(|e| Some(e));
            }
            // These can be skipped over when resolving a scene macro. They will be used when expanding the macro.
            Self::SceneMacroCommand(_) => (),
            Self::Layer(entry) => match resolve_mode {
                SceneResolveMode::OneLayerSceneOnly => (),
                SceneResolveMode::SceneOnly | SceneResolveMode::Full => {
                    entry.resolve(resolver, resolve_mode)?;
                }
            },
        }

        Ok(None)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SceneResolveMode
{
    /// Only resolve scene structure in the current scene layer.
    ///
    /// Scene macros will be fully expanded.
    OneLayerSceneOnly,
    SceneOnly,
    /// Resolve everything, including loadables and child scene layers.
    Full,
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

    #[cfg(feature = "full")]
    pub fn resolve(&mut self, resolver: &mut CobResolver, resolve_mode: SceneResolveMode) -> Result<(), String>
    {
        Self::resolve_entries_impl(self.name.as_str(), &mut self.entries, resolver, resolve_mode)
    }

    #[cfg(feature = "full")]
    pub fn resolve_entries_impl(
        name: &str,
        entries: &mut Vec<CobSceneLayerEntry>,
        resolver: &mut CobResolver,
        resolve_mode: SceneResolveMode,
    ) -> Result<(), String>
    {
        let mut idx = 0;
        while idx < entries.len() {
            // If resolving the entry returns a group of entries, they need to be flattened into this layer.
            let Some(mut group) = entries[idx].resolve(resolver, resolve_mode)? else {
                idx += 1;
                continue;
            };

            // Remove the old entry.
            entries.remove(idx);

            // Flatten the group into the layer.
            for entry in group.drain(..) {
                match entry {
                    CobSceneLayerEntry::Loadable(_) | CobSceneLayerEntry::Layer(_) => {
                        entries.insert(idx, entry);
                        idx += 1;
                    }
                    CobSceneLayerEntry::SceneMacroCall(_) => {
                        return Err(format!("failed resolving scene layer named {}; scene macro call unexpectedly not resolved",
                            name));
                    }
                    CobSceneLayerEntry::SceneMacroCommand(_) => {
                        return Err(
                            format!("failed resolving scene layer named {}; unexpected scene macro command",
                            name),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

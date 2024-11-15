use nom::character::complete::char;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobValueGroupEntry
{
    KeyValue(CobMapKeyValue),
    Value(CobValue),
}

impl CobValueGroupEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::KeyValue(key_value) => {
                key_value.write_to(writer)?;
            }
            Self::Value(value) => {
                value.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        // Check for key-value first in case a key is a CobValue.
        let fill = match rc(content, |c| CobMapKeyValue::try_parse(fill, c))? {
            (CobMapKVParseResult::Success(kv), next_fill, remaining) => {
                return Ok((Some(Self::KeyValue(kv)), next_fill, remaining));
            }
            (CobMapKVParseResult::KeyNoValue(CobMapKey::Value(value)), next_fill, remaining) => {
                return Ok((Some(Self::Value(value)), next_fill, remaining));
            }
            (CobMapKVParseResult::KeyNoValue(CobMapKey::FieldName { name, .. }), _, _) => {
                tracing::warn!("failed parsing value group entry at {}; found field name without value: {}",
                    get_location(content).as_str(), name.as_str());
                return Err(span_verify_error(content));
            }
            (CobMapKVParseResult::Failure, fill, _) => fill,
        };

        Ok((None, fill, content))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::KeyValue(key_value), Self::KeyValue(other_key_value)) => {
                key_value.recover_fill(other_key_value);
            }
            (Self::Value(value), Self::Value(other_value)) => {
                value.recover_fill(other_value);
            }
            _ => (),
        }
    }

    pub fn resolve<'a>(
        &mut self,
        constants: &'a ConstantsBuffer,
    ) -> Result<Option<&'a [CobValueGroupEntry]>, String>
    {
        match self {
            Self::KeyValue(kv) => kv.resolve(constants).map(|()| None),
            Self::Value(value) => value.resolve(constants),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobValueGroup
{
    /// Fill before opening `\`.
    pub start_fill: CobFill,
    pub entries: Vec<CobValueGroupEntry>,
    /// Fill before ending `\`.
    pub end_fill: CobFill,
}

impl CobValueGroup
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
        let Ok((remaining, _)) = char::<_, ()>('\\').parse(content) else {
            return Ok((None, start_fill, content));
        };

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            let fill_len = item_fill.len();
            match rc(remaining, |rm| CobValueGroupEntry::try_parse(item_fill, rm))? {
                (Some(entry), next_fill, after_entry) => {
                    if entries.len() > 0 {
                        if fill_len == 0 {
                            tracing::warn!("failed parsing value group at {}; entry #{} is not preceded by fill/whitespace",
                                get_location(content), entries.len() + 1);
                            return Err(span_verify_error(content));
                        }
                    }
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

        let (remaining, _) = char('\\').parse(remaining)?;
        let (post_fill, remaining) = CobFill::parse(remaining);
        Ok((Some(Self { start_fill, entries, end_fill }), post_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        for (entry, other_entry) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other_entry);
        }
        self.end_fill.recover(&other.end_fill);
    }

    pub fn resolve(&mut self, constants: &ConstantsBuffer) -> Result<(), String>
    {
        let mut idx = 0;
        while idx < self.entries.len() {
            // If resolving the entry returns a group of values, they need to be flattened into this outer group.
            let Some(group) = self.entries[idx].resolve(constants)? else {
                idx += 1;
                continue;
            };

            // Remove the old entry.
            self.entries.remove(idx);

            // Flatten the group into the outer group.
            for val in group.iter() {
                self.entries.insert(idx, val.clone());
                idx += 1;
            }
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

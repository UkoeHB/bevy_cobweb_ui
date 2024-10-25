use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafValueGroupEntry
{
    KeyValue(CafMapKeyValue),
    Value(CafValue),
}

impl CafValueGroupEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Value(value) => {
                value.write_to(writer)?;
            }
            Self::KeyValue(key_value) => {
                key_value.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        // Check for key-value first in case a key is a CafValue.
        let fill = match CafMapKeyValue::try_parse(fill, content)? {
            (Some(kv), next_fill, remaining) => return Ok((Some(Self::KeyValue(kv)), next_fill, remaining)),
            (_, fill, _) => fill
        };
        let fill = match CafValue::try_parse(fill, content)? {
            (Some(value), next_fill, remaining) => return Ok((Some(Self::Value(value)), next_fill, remaining)),
            (_, fill, _) => fill
        };

        Ok((None, fill, content))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Value(value), Self::Value(other_value)) => {
                value.recover_fill(other_value);
            }
            (Self::KeyValue(key_value), Self::KeyValue(other_key_value)) => {
                key_value.recover_fill(other_key_value);
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafValueGroup
{
    /// Fill before opening `\`.
    pub start_fill: CafFill,
    pub entries: Vec<CafValueGroupEntry>,
    /// Fill before ending `\`.
    pub end_fill: CafFill,
}

impl CafValueGroup
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

    pub fn try_parse(start_fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let Ok((remaining, _)) = char('\\').parse(content) else { return Ok((None, start_fill, content ))};

        let (mut item_fill, mut remaining) = CafFill::parse(remaining);
        let mut entries = vec![];

        let close_fill = loop {
            let fill_len = item_fill.len();
            match CafValueGroupEntry::try_parse(item_fill, remaining)? {
                (Some(entry), next_fill, after_entry) => {
                    if entries.len() > 0 {
                        if fill_len == 0 {
                            tracing::warn!("failed parsing array at {}; entry #{} is not preceded by fill/whitespace",
                                get_location(content), entries.len() + 1);
                            return Err(span_verify_error(content));
                        }
                    }
                    entries.push(entry);
                    item_fill = next_fill;
                    remaining = after_entry;
                }
                (None, close_fill, after_end) => {
                    remaining = after_end;
                    break close_fill;
                }
            }
        };

        let (remaining, _) = char('\\').parse(remaining)?;
        let (post_fill, remaining) = CafFill::parse(remaining);
        Ok((Some(Self { start_fill, entries, close_fill }), post_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        for (entry, other_entry) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other_entry);
        }
        self.end_fill.recover(&other.end_fill);
    }
}

/*
Parsing:
*/

//-------------------------------------------------------------------------------------------------------------------

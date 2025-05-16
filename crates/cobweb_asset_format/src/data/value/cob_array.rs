use nom::character::complete::char;
use nom::Parser;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobArray
{
    /// Fill before opening `[`.
    pub start_fill: CobFill,
    pub entries: Vec<CobValue>,
    /// Fill before ending `]`.
    pub end_fill: CobFill,
}

impl CobArray
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("[".as_bytes())?;
        for (idx, entry) in self.entries.iter().enumerate() {
            if idx == 0 {
                entry.write_to(writer)?;
            } else {
                entry.write_to_with_space(writer, " ")?;
            }
        }
        self.end_fill.write_to(writer)?;
        writer.write_bytes("]".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((remaining, _)) = char::<_, ()>('[').parse(content) else { return Ok((None, start_fill, content)) };

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            let fill_len = item_fill.len();
            match rc(remaining, |rm| CobValue::try_parse(item_fill, rm))? {
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
                (None, end_fill, after_end) => {
                    remaining = after_end;
                    break end_fill;
                }
            }
        };

        let (remaining, _) = char(']').parse(remaining)?;
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

    #[cfg(feature = "full_cob")]
    pub fn resolve(&mut self, resolver: &CobLoadableResolver) -> Result<(), String>
    {
        let mut idx = 0;
        while idx < self.entries.len() {
            // If resolving the entry returns a group of values, they need to be flattened into this array.
            let Some(group) = self.entries[idx].resolve(resolver)? else {
                idx += 1;
                continue;
            };

            // Remove the old entry.
            let old = self.entries.remove(idx);

            // Flatten the group into the array.
            for val in group.iter() {
                match val {
                    CobValueGroupEntry::KeyValue(_) => {
                        let err_msg = match old {
                            CobValue::Constant(constant) => {
                                format!("failed flattening value group constant ${} into \
                                an array, the group contains a key-value pair which is incompatible with arrays",
                                constant.path.as_str())
                            }
                            _ => format!("failed flattening {{source unknown}} value group into \
                                an array, the group contains a key-value pair which is incompatible with arrays"),
                        };
                        return Err(err_msg);
                    }
                    CobValueGroupEntry::Value(val) => {
                        self.entries.insert(idx, val.clone());
                        idx += 1;
                    }
                }
            }
        }

        Ok(())
    }
}

impl From<Vec<CobValue>> for CobArray
{
    fn from(entries: Vec<CobValue>) -> Self
    {
        Self {
            start_fill: CobFill::default(),
            entries,
            end_fill: CobFill::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

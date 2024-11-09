use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobSceneGroup
{
    /// Fill before opening `\`.
    pub start_fill: CobFill,
    pub entries: Vec<CobSceneLayerEntry>,
    /// Fill before ending `\`.
    pub end_fill: CobFill,
}

impl CobSceneGroup
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
        let Ok((remaining, _)) = char('\\').parse(content) else { return Ok((None, start_fill, content ))};

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut entries = vec![];

        let close_fill = loop {
            let fill_len = item_fill.len();
            // TODO: is this the right way to parse scene layers?
            match CobSceneLayerEntry::try_parse(item_fill, remaining)? {
                (Some(entry), next_fill, after_entry) => {
                    if entries.len() > 0 {
                        if fill_len == 0 {
                            tracing::warn!("failed parsing scene group at {}; entry #{} is not preceded by fill/whitespace",
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
        let (post_fill, remaining) = CobFill::parse(remaining);
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

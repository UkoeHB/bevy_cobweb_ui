use nom::bytes::complete::tag;
use nom::Parser;

use crate::prelude::*;

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

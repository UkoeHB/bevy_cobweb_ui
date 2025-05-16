use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CobNone
{
    pub fill: CobFill,
}

impl CobNone
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write_bytes("none".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        // NOTE: recursion not tested here (not vulnerable)

        let Ok((remaining, maybe_none)) = snake_identifier(content) else { return Ok((None, fill, content)) };
        if *maybe_none.fragment() != "none" {
            return Ok((None, fill, content));
        };
        let (next_fill, remaining) = CobFill::parse(remaining);
        Ok((Some(Self { fill }), next_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

/*
Parsing:
- parse as string
*/

//-------------------------------------------------------------------------------------------------------------------

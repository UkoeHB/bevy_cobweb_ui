use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafBool
{
    pub fill: CafFill,
    pub value: bool,
}

impl CafBool
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        let string = match self.value {
            true => "true",
            false => "false",
        };
        writer.write_bytes(string.as_bytes())?;
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        // NOTE: recursion not tested here (not vulnerable)

        let Ok((remaining, maybe_bool)) = snake_identifier(content) else { return Ok((None, fill, content)) };
        let value = match *maybe_bool.fragment() {
            "true" => true,
            "false" => false,
            _ => return Ok((None, fill, content)),
        };
        let (next_fill, remaining) = CafFill::parse(remaining);
        Ok((Some(Self { fill, value }), next_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

impl From<bool> for CafBool
{
    fn from(value: bool) -> Self
    {
        Self { fill: CafFill::default(), value }
    }
}

/*
Parsing:
- parse as string

fn parse()
{
    value(false, false_parser)
    value(true, true_parser)
}
*/

//-------------------------------------------------------------------------------------------------------------------

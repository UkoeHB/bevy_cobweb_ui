use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, recognize, verify};
use nom::multi::{many0, many1};
use nom::sequence::preceded;
use nom::{IResult, Parser};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafRustPrimitive
{
    pub fill: CafFill,
    pub primitive: SmolStr,
}

impl CafRustPrimitive
{
    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write_bytes(self.primitive.as_bytes())?;
        Ok(())
    }

    pub fn write_canonical(&self, buff: &mut String)
    {
        buff.push_str(self.primitive.as_str());
    }

    /// Nomlike means the ok value is `(remaining, result)`.
    pub fn parse_nomlike(content: Span) -> IResult<Span, Self>
    {
        let (fill, remaining) = CafFill::parse(content);
        let (remaining, res) = recognize(alt((
            preceded(char::<_, nom::error::Error<Span>>('f'), alt((tag("32"), tag("64")))),
            preceded(
                char('i'),
                alt((tag("8"), tag("16"), tag("32"), tag("64"), tag("128"), tag("size"))),
            ),
            preceded(
                char('u'),
                alt((tag("8"), tag("16"), tag("32"), tag("64"), tag("128"), tag("size"))),
            ),
            tag("bool"),
            tag("char"),
        )))
        .parse(remaining)?;
        Ok((remaining, Self { fill, primitive: SmolStr::from(*res.fragment()) }))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Any item that can appear in a generic.
#[derive(Debug, Clone, PartialEq)]
pub enum CafGenericItem
{
    Struct
    {
        fill: CafFill,
        id: SmolStr,
        generics: Option<CafGenerics>,
    },
    Tuple
    {
        /// Fill before opening `(`.
        fill: CafFill,
        values: Vec<CafGenericValue>,
        /// Fill before closing `)`.
        close_fill: CafFill,
    },
    RustPrimitive(CafRustPrimitive),
}

impl CafGenericItem
{
    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        match self {
            Self::Struct { fill, id, generics } => {
                fill.write_to_or_else(writer, space)?;
                writer.write_bytes(id.as_bytes())?;
                if let Some(generics) = generics {
                    generics.write_to(writer)?;
                }
            }
            Self::Tuple { fill, values, close_fill } => {
                fill.write_to_or_else(writer, space)?;
                writer.write_bytes("(".as_bytes())?;
                for (idx, generic) in values.iter().enumerate() {
                    if idx == 0 {
                        generic.write_to_with_space(writer, "")?;
                    } else {
                        generic.write_to_with_space(writer, ", ")?;
                    }
                }
                close_fill.write_to(writer)?;
                writer.write_bytes(")".as_bytes())?;
            }
            Self::RustPrimitive(primitive) => {
                primitive.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn write_canonical(&self, buff: &mut String)
    {
        match self {
            Self::Struct { id, generics, .. } => {
                buff.push_str(id.as_str());
                if let Some(generics) = generics {
                    generics.write_canonical(buff);
                }
            }
            Self::Tuple { values, .. } => {
                buff.push_str("(");
                let num_values = values.len();
                for (idx, value) in values.iter().enumerate() {
                    value.write_canonical(buff);
                    if idx + 1 < num_values {
                        buff.push_str(", ");
                    }
                }
                buff.push_str(")");
            }
            Self::RustPrimitive(primitive) => {
                primitive.write_canonical(buff);
            }
        }
    }

    /// Nomlike means the ok value is `(remaining, result)`.
    pub fn parse_nomlike(content: Span) -> IResult<Span, Self>
    {
        alt((
            Self::parse_struct_nomlike,
            Self::parse_tuple_nomlike,
            map(CafRustPrimitive::parse_nomlike, |p| Self::RustPrimitive(p)),
        ))
        .parse(content)
    }

    fn parse_struct_nomlike(content: Span) -> IResult<Span, Self>
    {
        let (fill, remaining) = CafFill::parse(content);
        let (remaining, id) = camel_identifier(remaining)?;
        let (generics, remaining) = CafGenerics::try_parse(remaining)?;
        Ok((
            remaining,
            Self::Struct { fill, id: SmolStr::from(*id.fragment()), generics },
        ))
    }

    fn parse_tuple_nomlike(content: Span) -> IResult<Span, Self>
    {
        let (fill, remaining) = CafFill::parse(content);
        let (remaining, _) = char('(').parse(remaining)?;
        let (remaining, values) = many0(CafGenericValue::parse_nomlike).parse(remaining)?;
        let (close_fill, remaining) = CafFill::parse(remaining);
        let (remaining, _) = char(')').parse(remaining)?;
        Ok((remaining, Self::Tuple { fill, values, close_fill }))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (
                Self::Struct { fill, generics, .. },
                Self::Struct { fill: other_fill, generics: other_generics, .. },
            ) => {
                fill.recover(other_fill);
                if let (Some(generics), Some(other_generics)) = (generics, other_generics) {
                    generics.recover_fill(other_generics);
                }
            }
            (
                Self::Tuple { fill, values, close_fill },
                Self::Tuple {
                    fill: other_fill,
                    values: other_values,
                    close_fill: other_close_fill,
                },
            ) => {
                fill.recover(other_fill);
                for (value, other_value) in values.iter_mut().zip(other_values.iter()) {
                    value.recover_fill(other_value);
                }
                close_fill.recover(other_close_fill);
            }
            (Self::RustPrimitive(primitive), Self::RustPrimitive(other_primitive)) => {
                primitive.recover_fill(other_primitive);
            }
            _ => (),
        }
    }

    /// Returns `true` if all generic items are types and not macro params.
    pub fn is_resolved(&self) -> bool
    {
        match self {
            Self::Struct { generics, .. } => match generics {
                Some(generics) => generics.is_resolved(),
                None => true,
            },
            Self::Tuple { values, .. } => !values.iter().any(|v| !v.is_resolved()),
            Self::RustPrimitive(_) => true,
        }
    }
}

// Parsing:
// - Primitive must match one of the known primitives (ints, floats, bool).

//-------------------------------------------------------------------------------------------------------------------

/// Note that constants and macros are unavailable inside generics. Only non-optional macro type params can be
/// used.
#[derive(Debug, Clone, PartialEq)]
pub enum CafGenericValue
{
    Item(CafGenericItem),
    MacroParam(CafMacroParam),
}

impl CafGenericValue
{
    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        match self {
            Self::Item(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::MacroParam(val) => {
                val.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn write_canonical(&self, buff: &mut String)
    {
        match self {
            Self::Item(item) => {
                item.write_canonical(buff);
            }
            Self::MacroParam(param) => {
                // This warning probably indicates a bug. Please report it so it can be fixed!
                tracing::warn!("generic contains unresolved macro param {:?} while writing canonical", param);
            }
        }
    }

    /// Nomlike means the ok value is `(remaining, result)`.
    pub fn parse_nomlike(content: Span) -> IResult<Span, Self>
    {
        alt((
            map(CafGenericItem::parse_nomlike, |i| Self::Item(i)),
            // TODO: require special macro param type for generic items?
            map(verify(CafMacroParam::parse_nomlike, |p| p.is_required()), |p| {
                Self::MacroParam(p)
            }),
        ))
        .parse(content)
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Item(val), Self::Item(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::MacroParam(val), Self::MacroParam(other_val)) => {
                val.recover_fill(other_val);
            }
            _ => (),
        }
    }

    /// Returns `true` if all generic items are types and not macro params.
    pub fn is_resolved(&self) -> bool
    {
        let Self::Item(item) = self else { return false };
        item.is_resolved()
    }

    //todo: resolve_macro
    // - Only macro params tied to a macro type param def can be used.
}

/*
Parsing:
- Macro param should be non-optional.
*/

//-------------------------------------------------------------------------------------------------------------------

/// Note that constants and macros are unavailable inside generics. Only macro type params can be used.
#[derive(Debug, Clone, PartialEq)]
pub struct CafGenerics
{
    /// Each of these values is expected to take care of its own fill.
    pub values: Vec<CafGenericValue>,
    /// Fill before closing `>`.
    pub close_fill: CafFill,
}

impl CafGenerics
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        writer.write_bytes("<".as_bytes())?;
        for (idx, generic) in self.values.iter().enumerate() {
            if idx == 0 {
                generic.write_to_with_space(writer, "")?;
            } else {
                generic.write_to_with_space(writer, ", ")?;
            }
        }
        self.close_fill.write_to(writer)?;
        writer.write_bytes(">".as_bytes())?;
        Ok(())
    }

    /// Assembles generic into a clean sequence of items separated by `, `.
    pub fn write_canonical(&self, buff: &mut String)
    {
        buff.push_str("<");
        let num_values = self.values.len();
        for (idx, value) in self.values.iter().enumerate() {
            value.write_canonical(buff);
            if idx + 1 < num_values {
                buff.push_str(", ");
            }
        }
        buff.push_str(">");
    }

    pub fn try_parse(content: Span) -> Result<(Option<Self>, Span), SpanError>
    {
        let Ok((remaining, _)) = char::<_, ()>('<').parse(content) else { return Ok((None, content)) };
        let (remaining, values) = many1(CafGenericValue::parse_nomlike).parse(remaining)?;
        let (close_fill, remaining) = CafFill::parse(remaining);
        let (remaining, _) = char('>').parse(remaining)?;
        Ok((Some(Self { values, close_fill }), remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        // TODO: search for equal pairing instead?
        for (value, other_value) in self.values.iter_mut().zip(other.values.iter()) {
            value.recover_fill(other_value);
        }
        self.close_fill.recover(&other.close_fill);
    }

    /// Returns `true` if all generic items are types and not macro params.
    pub fn is_resolved(&self) -> bool
    {
        !self.values.iter().any(|v| !v.is_resolved())
    }
}

//-------------------------------------------------------------------------------------------------------------------

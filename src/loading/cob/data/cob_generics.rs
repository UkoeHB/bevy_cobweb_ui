use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, recognize};
use nom::multi::{many0, many1};
use nom::sequence::preceded;
use nom::{IResult, Parser};
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobRustPrimitive
{
    pub fill: CobFill,
    pub primitive: SmolStr,
}

impl CobRustPrimitive
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

    /// Nomlike means the Ok value is `(remaining, result)`.
    pub fn parse_nomlike(content: Span) -> IResult<Span, Self>
    {
        let (fill, remaining) = CobFill::parse(content);
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
pub enum CobGenericItem
{
    Struct
    {
        fill: CobFill,
        id: SmolStr,
        generics: Option<CobGenerics>,
    },
    Tuple
    {
        /// Fill before opening `(`.
        fill: CobFill,
        values: Vec<CobGenericValue>,
        /// Fill before closing `)`.
        close_fill: CobFill,
    },
    RustPrimitive(CobRustPrimitive),
}

impl CobGenericItem
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
        rc(content, |c| {
            alt((
                Self::parse_struct_nomlike,
                Self::parse_tuple_nomlike,
                map(CobRustPrimitive::parse_nomlike, |p| Self::RustPrimitive(p)),
            ))
            .parse(c)
        })
    }

    fn parse_struct_nomlike(content: Span) -> IResult<Span, Self>
    {
        // NOTE: for simplicity, don't check recursion here (see Self::parse_nomlike)
        let (fill, remaining) = CobFill::parse(content);
        let (remaining, id) = camel_identifier(remaining)?;
        let (generics, remaining) = CobGenerics::try_parse(remaining)?;
        Ok((
            remaining,
            Self::Struct { fill, id: SmolStr::from(*id.fragment()), generics },
        ))
    }

    fn parse_tuple_nomlike(content: Span) -> IResult<Span, Self>
    {
        // NOTE: for simplicity, don't check recursion here (see Self::parse_nomlike)
        let (fill, remaining) = CobFill::parse(content);
        let (remaining, _) = char('(').parse(remaining)?;
        let (remaining, values) = many0(CobGenericValue::parse_nomlike).parse(remaining)?;
        let (close_fill, remaining) = CobFill::parse(remaining);
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

    // /// Returns `true` if all generic items are types and not macro params.
    // pub fn is_resolved(&self) -> bool
    // {
    //     match self {
    //         Self::Struct { generics, .. } => match generics {
    //             Some(generics) => generics.is_resolved(),
    //             None => true,
    //         },
    //         Self::Tuple { values, .. } => !values.iter().any(|v| !v.is_resolved()),
    //         Self::RustPrimitive(_) => true,
    //     }
    // }
}

// Parsing:
// - Primitive must match one of the known primitives (ints, floats, bool).

//-------------------------------------------------------------------------------------------------------------------

/// Note that constants and macros are unavailable inside generics.
// This is currently a newtype for `CobGenericItem`, but may in the future be reworked to include macro params.
#[derive(Debug, Clone, PartialEq)]
pub struct CobGenericValue(pub CobGenericItem);

impl CobGenericValue
{
    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.0.write_to_with_space(writer, space)
    }

    pub fn write_canonical(&self, buff: &mut String)
    {
        self.0.write_canonical(buff);
    }

    /// Nomlike means the ok value is `(remaining, result)`.
    pub fn parse_nomlike(content: Span) -> IResult<Span, Self>
    {
        // NOTE: for simplicity, don't test recursion here (items internally check recursion)
        alt((map(CobGenericItem::parse_nomlike, |i| Self(i)),)).parse(content)
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.0.recover_fill(&other.0);
    }

    // /// Returns `true` if all generic items are types.
    // pub fn is_resolved(&self) -> bool
    // {
    //     self.0.is_resolved()
    // }
}

//-------------------------------------------------------------------------------------------------------------------

/// Note that constants and macros are unavailable inside generics.
#[derive(Debug, Clone, PartialEq)]
pub struct CobGenerics
{
    /// Each of these values is expected to take care of its own fill.
    pub values: Vec<CobGenericValue>,
    /// Fill before closing `>`.
    pub close_fill: CobFill,
}

impl CobGenerics
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
        // NOTE: for simplicity, don't test recursion here (value internally checks recursion)
        let Ok((remaining, _)) = char::<_, ()>('<').parse(content) else { return Ok((None, content)) };
        let (remaining, values) = many1(CobGenericValue::parse_nomlike).parse(remaining)?;
        let (close_fill, remaining) = CobFill::parse(remaining);
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

    // /// Returns `true` if all generic items are types.
    // pub fn is_resolved(&self) -> bool
    // {
    //     !self.values.iter().any(|v| !v.is_resolved())
    // }
}

//-------------------------------------------------------------------------------------------------------------------

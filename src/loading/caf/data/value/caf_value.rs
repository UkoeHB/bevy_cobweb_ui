use bevy::reflect::serde::TypedReflectSerializer;
use bevy::reflect::{Reflect, TypeRegistry};
use serde::Serialize;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafValue
{
    Enum(CafEnum),
    /// Special built-in types like `auto` and `#FFFFFF` for colors.
    Builtin(CafBuiltin),
    Array(CafArray),
    Tuple(CafTuple),
    Map(CafMap),
    Number(CafNumber),
    Bool(CafBool),
    None(CafNone),
    String(CafString),
    Constant(CafConstant),
    DataMacro(CafDataMacroCall),
    /// Only valid inside a macro definition.
    MacroParam(CafMacroParam),
}

impl CafValue
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        match self {
            Self::Enum(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Builtin(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Array(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Tuple(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Map(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Number(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Bool(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::None(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::String(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::Constant(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::DataMacro(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::MacroParam(val) => {
                val.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let fill = match rc(content, move |c| CafEnum::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Enum(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafBuiltin::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Builtin(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafArray::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Array(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafTuple::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Tuple(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafMap::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Map(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafNumber::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Number(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafBool::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Bool(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafNone::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::None(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafString::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::String(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafConstant::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::Constant(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafDataMacroCall::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::DataMacro(value)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CafMacroParam::try_parse(fill, c))? {
            (Some(value), fill, remaining) => return Ok((Some(Self::MacroParam(value)), fill, remaining)),
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Enum(val), Self::Enum(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Builtin(val), Self::Builtin(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Array(val), Self::Array(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Tuple(val), Self::Tuple(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Map(val), Self::Map(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Number(val), Self::Number(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Bool(val), Self::Bool(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::None(val), Self::None(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::String(val), Self::String(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::Constant(val), Self::Constant(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::DataMacro(val), Self::DataMacro(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::MacroParam(val), Self::MacroParam(other_val)) => {
                val.recover_fill(other_val);
            }
            _ => (),
        }
    }

    pub fn extract<T: ?Sized + Serialize>(value: &T) -> CafResult<Self>
    {
        value.serialize(CafValueSerializer)
    }

    pub fn extract_reflect<T: Reflect + 'static>(value: &T, registry: &TypeRegistry) -> CafResult<Self>
    {
        let wrapper = TypedReflectSerializer::new(value, registry);
        wrapper.serialize(CafValueSerializer)
    }
}

//-------------------------------------------------------------------------------------------------------------------

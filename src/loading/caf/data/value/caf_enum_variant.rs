use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafEnumVariantIdentifier
{
    pub fill: CafFill,
    pub name: SmolStr,
}

impl CafEnumVariantIdentifier
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str)
        -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write(self.name.as_bytes())?;
        Ok(())
    }

    pub fn to_json_string(&self) -> Result<String, std::io::Error>
    {
        Ok(String::from(self.name.as_str()))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

impl From<&str> for CafEnumVariantIdentifier
{
    fn from(string: &str) -> Self
    {
        Self { fill: CafFill::default(), name: SmolStr::from(string) }
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafEnumVariant
{
    Unit
    {
        id: CafEnumVariantIdentifier
    },
    Tuple
    {
        id: CafEnumVariantIdentifier, tuple: CafTuple
    },
    /// Shorthand for and equivalent to a tuple of array.
    Array
    {
        id: CafEnumVariantIdentifier, array: CafArray
    },
    Map
    {
        id: CafEnumVariantIdentifier, map: CafMap
    },
}

impl CafEnumVariant
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str)
        -> Result<(), std::io::Error>
    {
        match self {
            Self::Unit { id } => {
                id.write_to_with_space(writer, space)?;
            }
            Self::Tuple { id, tuple } => {
                id.write_to_with_space(writer, space)?;
                tuple.write_to(writer)?;
            }
            Self::Array { id, array } => {
                id.write_to_with_space(writer, space)?;
                array.write_to(writer)?;
            }
            Self::Map { id, map } => {
                id.write_to_with_space(writer, space)?;
                map.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        match self {
            Self::Unit { id } => {
                // "..id.."
                Ok(serde_json::Value::String(id.to_json_string()?))
            }
            Self::Tuple { id, tuple } => {
                // {"..id..": [..tuple items..]}
                // If there is one tuple-enum item then there will be no wrapping array.
                let key = id.to_json_string()?;
                let value = tuple.to_json_for_enum()?;
                let mut map = serde_json::Map::default();
                map.insert(key, value);
                Ok(serde_json::Value::Object(map))
            }
            Self::Array { id, array } => {
                // {"..id..": [..array items..]}
                // Note that unlike tuples, enum-tuples of single items don't need a wrapping array in JSON.
                // So we just paste the CafArray in directly.
                let key = id.to_json_string()?;
                let value = array.to_json()?;
                let mut map = serde_json::Map::default();
                map.insert(key, value);
                Ok(serde_json::Value::Object(map))
            }
            Self::Map { id, map } => {
                // {"..id..": {..map items..}}
                let key = id.to_json_string()?;
                let value = map.to_json()?;
                let mut map = serde_json::Map::default();
                map.insert(key, value);
                Ok(serde_json::Value::Object(map))
            }
        }
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Unit { id }, Self::Unit { id: other_id }) => {
                id.recover_fill(other_id);
            }
            (Self::Tuple { id, tuple }, Self::Tuple { id: other_id, tuple: other_tuple }) => {
                id.recover_fill(other_id);
                tuple.recover_fill(other_tuple);
            }
            (Self::Array { id, array }, Self::Array { id: other_id, array: other_array }) => {
                id.recover_fill(other_id);
                array.recover_fill(other_array);
            }
            (Self::Map { id, map }, Self::Map { id: other_id, map: other_map }) => {
                id.recover_fill(other_id);
                map.recover_fill(other_map);
            }
            _ => (),
        }
    }

    pub fn unit(variant: &str) -> Self
    {
        Self::Unit { id: variant.into() }
    }

    pub fn array(variant: &str, array: CafArray) -> Self
    {
        Self::Array { id: variant.into(), array }
    }

    pub fn newtype(variant: &str, value: CafValue) -> Self
    {
        Self::Tuple { id: variant.into(), tuple: CafTuple::single(value) }
    }

    pub fn tuple(variant: &str, tuple: CafTuple) -> Self
    {
        Self::Tuple { id: variant.into(), tuple }
    }

    pub fn map(variant: &str, map: CafMap) -> Self
    {
        Self::Map { id: variant.into(), map }
    }

    pub fn id(&self) -> &str
    {
        match self {
            CafEnumVariant::Unit{ id } |
            CafEnumVariant::Tuple{ id, .. } |
            CafEnumVariant::Array{ id, .. } |
            CafEnumVariant::Map{ id, .. } => id.name.as_str()
        }
    }
}

/*
Parsing:
- no whitespace allowed betwen type id and value
*/

//-------------------------------------------------------------------------------------------------------------------

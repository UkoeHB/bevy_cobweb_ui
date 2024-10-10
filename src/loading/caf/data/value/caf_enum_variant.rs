use bevy::reflect::{EnumInfo, TypeInfo, TypeRegistry, VariantInfo};
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

    /// Handles `Option` enums, which are elided in JSON and CAF.
    pub fn try_from_json_option(
        val: &serde_json::Value,
        enum_info: &EnumInfo,
        registry: &TypeRegistry,
    ) -> Result<Option<CafValue>, String>
    {
        if enum_info.type_path_table().ident() != Some("Option") {
            return Ok(None);
        }

        if *val == serde_json::Value::Null {
            // If no `None` then maybe there's a custom Option type or nested options.
            if enum_info.variant("None").is_some() {
                return Ok(Some(CafValue::None(CafNone { fill: CafFill::default() })));
            }
        }

        let Some(VariantInfo::Tuple(some_variant)) = enum_info.variant("Some") else {
            // If no `Some` then maybe there's a custom Option type.
            return Ok(None);
        };

        if some_variant.field_len() != 1 {
            // If `Some` doesn't have one field then maybe there's a custom Option type.
            return Ok(None);
        }
        let field = some_variant.field_at(0).unwrap();
        let Some(registration) = registry.get(field.type_id()) else { unreachable!() };

        Ok(Some(CafValue::from_json(val, registration.type_info(), registry)?))
    }

    /// Note: [`Self::try_from_json_option`] should be called first to filter out `Option` enums.
    pub fn from_json(
        val: &serde_json::Value,
        enum_info: &EnumInfo,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        match val {
            serde_json::Value::String(json_str) => {
                let Some(variant_info) = enum_info.variant(json_str.as_str()) else {
                    return Err(format!(
                        "failed converting {:?} from json enum variant {:?}; enum variant is unknown",
                        enum_info.type_path(), json_str
                    ));
                };
                let VariantInfo::Unit(_) = variant_info else {
                    return Err(format!(
                        "failed converting {:?} from json enum variant {:?}; enum variant is not a unit-like but only \
                        json string is provided (indicating a unit-like)", enum_info.type_path(), json_str
                    ));
                };

                Ok(Self::Unit { id: CafEnumVariantIdentifier::from(json_str.as_str()) })
            }
            serde_json::Value::Object(json_map) => {
                if json_map.len() != 1 {
                    return Err(format!(
                        "failed converting {:?} enum variant from json {:?}; json map doesn't contain exactly 1 \
                        entry corresponding to an enum variant", enum_info.type_path(), json_map
                    ));
                }
                let (enum_variant_str, variant_val) = json_map.iter().next().unwrap();
                let Some(variant_info) = enum_info.variant(enum_variant_str.as_str()) else {
                    return Err(format!(
                        "failed converting {:?} from json enum variant {:?}; enum variant is unknown",
                        enum_info.type_path(), enum_variant_str
                    ));
                };
                let variant_id = CafEnumVariantIdentifier::from(enum_variant_str.as_str());

                match variant_info {
                    VariantInfo::Tuple(info) => {
                        // A tuple-enum of one item is *not* wrapped in an array on the JSON side.
                        // Also, if the item is an array/list then we use the CafEnum::Array shorthand.
                        if info.field_len() == 1 {
                            let field = info.field_at(0).unwrap();
                            let Some(registration) = registry.get(field.type_id()) else { unreachable!() };
                            let field_info = registration.type_info();

                            match field_info {
                                TypeInfo::Array(_) | TypeInfo::List(_) => {
                                    return Ok(Self::Array {
                                        id: variant_id,
                                        array: CafArray::from_json(variant_val, field_info, registry)?,
                                    });
                                }
                                _ => {
                                    return Ok(Self::Tuple {
                                        id: variant_id,
                                        tuple: CafTuple::from_json_as_enum_single(
                                            variant_val,
                                            enum_info.type_path(),
                                            field_info,
                                            registry,
                                        )?,
                                    });
                                }
                            }
                        }

                        // Tuple of 0 or multiple items (json has array wrapper).
                        Ok(Self::Tuple {
                            id: variant_id,
                            tuple: CafTuple::from_json_as_enum(
                                variant_val,
                                enum_info.type_path(),
                                enum_variant_str.as_str(),
                                info,
                                registry,
                            )?,
                        })
                    }
                    VariantInfo::Struct(info) => Ok(Self::Map {
                        id: variant_id,
                        map: CafMap::from_json_as_enum(
                            variant_val,
                            enum_info.type_path(),
                            enum_variant_str.as_str(),
                            info,
                            registry,
                        )?,
                    }),
                    VariantInfo::Unit(_) => {
                        return Err(format!(
                            "failed converting {:?} from json enum variant {:?}; json value {:?} is provided but variant \
                            is a unit-like (has no value)",
                            enum_info.type_path(), enum_variant_str, variant_val
                        ));
                    }
                }
            }
            _ => Err(format!(
                "failed converting {:?} from json {:?}; json is not a string or map",
                enum_info.type_path(), val
            )),
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
}

/*
Parsing:
- no whitespace allowed betwen type id and value
*/

//-------------------------------------------------------------------------------------------------------------------

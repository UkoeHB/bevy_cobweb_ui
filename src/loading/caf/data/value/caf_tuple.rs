use std::any::TypeId;

use bevy::reflect::{TupleVariantInfo, TypeInfo, TypeRegistry};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafTuple
{
    /// Fill before opening `(`.
    pub start_fill: CafFill,
    pub entries: Vec<CafValue>,
    /// Fill before ending `)`.
    pub end_fill: CafFill,
}

impl CafTuple
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str)
        -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write("(".as_bytes())?;
        for (idx, entry) in self.entries.iter().enumerate() {
            if idx == 0 {
                entry.write_to(writer)?;
            } else {
                entry.write_to_with_space(writer, " ")?;
            }
        }
        self.end_fill.write_to(writer)?;
        writer.write(")".as_bytes())?;
        Ok(())
    }

    /// Includes tuple-structs and floating tuples.
    pub fn to_json_for_type(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut array = Vec::with_capacity(self.entries.len());
        for entry in self.entries.iter() {
            array.push(entry.to_json()?);
        }
        Ok(serde_json::Value::Array(array))
    }

    pub fn to_json_for_enum(&self) -> Result<serde_json::Value, std::io::Error>
    {
        // A tuple of one item is not wrapped on the JSON side.
        if self.entries.len() == 1 {
            self.entries[0].to_json()
        } else {
            self.to_json_for_type()
        }
    }

    fn from_json_impl<'a>(
        num_values: usize,
        json_values_iter: impl Iterator<Item = &'a serde_json::Value>,
        type_path: &'static str,
        num_fields: usize,
        field_iter: impl Iterator<Item = TypeId>,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        if num_fields != num_values {
            return Err(format!(
                "failed converting {:?} from json {:?} into a tuple; json does not match expected number \
                of tuple fields",
                type_path, json_values_iter.collect::<Vec<&serde_json::Value>>()
            ));
        }

        let mut entries = Vec::with_capacity(num_values);
        for (field_id, json_value) in field_iter.zip(json_values_iter) {
            let Some(registration) = registry.get(field_id) else { unreachable!() };
            entries.push(CafValue::from_json(json_value, registration.type_info(), registry)?);
        }
        Ok(Self {
            start_fill: CafFill::default(),
            entries,
            end_fill: CafFill::default(),
        })
    }

    /// Plain tuples and tuple-structs are wrapped in an array on the JSON side.
    pub fn from_json_as_type(
        val: &serde_json::Value,
        type_info: &TypeInfo,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        let type_path = type_info.type_path();
        let serde_json::Value::Array(json_vec) = val else {
            return Err(format!(
                "failed converting {:?} from json {:?}; expected json to be an array",
                type_path, val
            ));
        };

        match type_info {
            TypeInfo::TupleStruct(info) => Self::from_json_impl(
                json_vec.len(),
                json_vec.iter(),
                type_path,
                info.field_len(),
                info.iter().map(|i| i.type_id()),
                registry,
            ),
            TypeInfo::Tuple(info) => Self::from_json_impl(
                json_vec.len(),
                json_vec.iter(),
                type_path,
                info.field_len(),
                info.iter().map(|i| i.type_id()),
                registry,
            ),
            _ => Err(format!(
                "failed converting {:?} from json {:?} into a tuple; type is not a tuplestruct/tuple",
                type_info.type_path(), val
            )),
        }
    }

    /// Enum-tuples of a single item are not wrapped in an array on the JSON side.
    pub fn from_json_as_enum_single(
        val: &serde_json::Value,
        type_path: &'static str,
        variant_info: &TypeInfo,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        Self::from_json_impl(
            1,
            [val].into_iter(),
            type_path,
            1,
            [variant_info.type_id()].into_iter(),
            registry,
        )
    }

    /// Enum-tuples of multiple values are wrapped in an array on the JSON side.
    pub fn from_json_as_enum(
        val: &serde_json::Value,
        type_path: &'static str,
        variant_name: &str,
        variant_info: &TupleVariantInfo,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        let serde_json::Value::Array(json_vec) = val else {
            return Err(format!(
                "failed converting {:?}::{:?} from json {:?}; expected json to be an array",
                type_path, variant_name, val
            ));
        };

        Self::from_json_impl(
            json_vec.len(),
            json_vec.iter(),
            type_path,
            variant_info.field_len(),
            variant_info.iter().map(|i| i.type_id()),
            registry,
        )
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

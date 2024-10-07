use bevy::reflect::{TypeInfo, TypeRegistry};

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
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.fill.write_to(writer)?;
        let string = match *self.value {
            true => "true",
            false => "false",
        };
        writer.write(string.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        Ok(serde_json::Value::Bool(self.value))
    }

    pub fn from_json(
        val: &serde_json::Value,
        type_info: &TypeInfo,
        registry: &TypeRegistry,
    ) -> Result<Self, String>
    {
        match type_info {
            TypeInfo::Struct(info) => {}
            TypeInfo::TupleStruct(info) => {}
            TypeInfo::Tuple(_) => {}
            TypeInfo::List(_) => {}
            TypeInfo::Array(_) => {}
            TypeInfo::Map(_) => Err(format!(
                    "failed converting {:?} from json {:?} as an instruction; type is a map not a struct/enum",
                    val, type_info.type_path()
                )),
            TypeInfo::Enum(info) => {}
            TypeInfo::Value(_) => {}
        }
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

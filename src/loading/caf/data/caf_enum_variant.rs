
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafEnumVariantIdentifier
{
    pub start_fill: CafFill,
    pub name: SmolStr,
}

impl CafEnumVariantIdentifier
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write(self.name.as_bytes())?;
        Ok(())
    }

    pub fn to_json_string(&self) -> Result<String, std::io::Error>
    {
        Ok(String::from(self.name.as_str()))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafEnumVariant
{
    Unit{
        id: CafEnumVariantIdentifier
    },
    Tuple{
        id: CafEnumVariantIdentifier,
        tuple: CafTuple,
    },
    /// Shorthand for and equivalent to a tuple of array.
    Array{
        id: CafEnumVariantIdentifier,
        array: CafArray,
    },
    Map{
        id: CafEnumVariantIdentifier,
        map: CafMap,
    }
}

impl CafEnumVariant
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Unit{id} => {
                id.write_to(writer)?;
            }
            Self::Tuple{id, tuple} => {
                id.write_to(writer)?;
                tuple.write_to(writer)?;
            }
            Self::Array{id, array} => {
                id.write_to(writer)?;
                array.write_to(writer)?;
            }
            Self::Map{id, map} => {
                id.write_to(writer)?;
                map.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        match *self {
            Self::Unit{id} => {
                // "..id.."
                Ok(serde_json::Value::String(id.to_json_string()?))
            }
            Self::Tuple{id, tuple} => {
                // {"..id..": [..tuple items..]}
                let key = id.to_json_string()?;
                let value = tuple.to_json()?;
                let mut map = serde_json::Map::default();
                map.insert(key, value);
                Ok(serde_json::Object(map))
            }
            Self::Array{id, array} => {
                // {"..id..": [[..array items..]]}
                let key = id.to_json_string()?;
                let value = serde_json::Value::Array(vec![array.to_json()?]);
                let mut map = serde_json::Map::default();
                map.insert(key, value);
                Ok(serde_json::Object(map))
            }
            Self::Map{id, map} => {
                // {"..id..": {..map items..}}
                let key = id.to_json_string()?;
                let value = map.to_json()?;
                let mut map = serde_json::Map::default();
                map.insert(key, value);
                Ok(serde_json::Object(map))
            }
        }
    }

    pub fn from_json(val: &serde_json::Value, type_info: &TypeInfo, registry: &TypeRegistry) -> Result<Self, String>
    {
        // TODO: check for Option

        let serde_json::Value::Object(json_map) = val else {
            return Err(format!(
                "failed converting {:?} from json {:?}; expected json to be a map",
                type_info.type_path(), val
            ));
        };

        match type_info {
            TypeInfo::Struct(info) => {

            }
            TypeInfo::TupleStruct(info) => {

            }
            TypeInfo::Tuple(_) => {

            }
            TypeInfo::List(_) => {

            }
            TypeInfo::Array(_) => {

            }
            TypeInfo::Map(_) => {
                Err(format!(
                    "failed converting {:?} from json {:?} as an instruction; type is a map not a struct/enum",
                    val, type_info.type_path()
                ))
            }
            TypeInfo::Enum(info) => {

            }
            TypeInfo::Value(_) => {
                
            }
        }
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Unit{id}, Self::Unit{id: other_id}) => {
                id.recover_fill(other_id);
            }
            (Self::Tuple{id, tuple}, Self::Tuple{id: other_id, tuple: other_tuple}) => {
                id.recover_fill(other_id);
                tuple.recover_fill(other_tuple);
            }
            (Self::Array{id, array}, Self::Array{id: other_id, array: other_array}) => {
                id.recover_fill(other_id);
                array.recover_fill(other_array);
            }
            (Self::Map{id, map}, Self::Map{id: other_id, map: other_map}) => {
                id.recover_fill(other_id);
                map.recover_fill(other_map);
            }
            _ => ()
        }
    }
}

/*
Parsing:
- no whitespace allowed betwen type id and value
*/

//-------------------------------------------------------------------------------------------------------------------

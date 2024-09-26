
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
        tuple: CafValueTuple,
    },
    /// Shorthand for and equivalent to a tuple of array.
    Array{
        id: CafEnumVariantIdentifier,
        array: CafValueArray,
    },
    Map{
        id: CafEnumVariantIdentifier,
        map: CafValueMap,
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
}

/*
Parsing:
- no whitespace allowed betwen type id and value
*/

//-------------------------------------------------------------------------------------------------------------------

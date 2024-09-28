


//-------------------------------------------------------------------------------------------------------------------

/// Stores the original number value string and also the number deserialized from JSON.
///
/// We need to keep the original so it can be reserialized in the correct format.
///
/// We store a JSON value for convenience instead of implementing our own deserialization routine.
#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafNumberValue
{
    pub original: SmolStr,
    pub deserialized: serde_json::Number,
}

impl CafNumberValue
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write(self.original.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        Ok(serde_json::Value::Number(self.deserialized.clone()))
    }

    pub fn try_from_json_string(json_str: &String) -> Option<Self>
    {
        let deserialized = serde_json::Number::from_str(json_str).ok()?;
        Some(Self{ original: SmolStr::from(json_str.as_str()), deserialized })
    }
}

/*
Parsing:
- Use regex to identify number, then parse it using a JSON deserializer with serde_json::Number::from_str() and
check result
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafNumber
{
    pub fill: CafFill,
    pub number: CafNumberValue
}

impl CafNumber
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, '')
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: impl AsRef<str>) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        self.number.write_to(writer)?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        self.number.to_json()
    }

    pub fn from_json(val: &serde_json::Value, type_info: &TypeInfo, registry: &TypeRegistry) -> Result<Self, String>
    {
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

    pub fn try_from_json_string(json_str: &String) -> Option<Self>
    {
        let number = CafNumberValue::try_from_json_string(json_str)?;
        Some(Self{ fill: CafFill::default(), number })
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

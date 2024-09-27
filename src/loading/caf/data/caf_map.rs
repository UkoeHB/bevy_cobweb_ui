

//-------------------------------------------------------------------------------------------------------------------

/// Currently only strings and ints/floats are supported as map keys, since we are tied to JSON limitations.
#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafMapKey
{
    FieldName{
        fill: CafFill,
        name: SmolStr,
    },
    String(CafString),
    Numeric(CafNumber)
}

impl CafMapKey
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, '')
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: impl AsRef<str>) -> Result<(), std::io::Error>
    {
        match self {
            Self::FieldName{fill, name} => {
                fill.write_to_or_else(writer, space)?;
                writer.write(name.as_bytes())?;
            }
            Self::String(string) => {
                string.write_to_with_space(writer, space)?;
            }
            Self::Numeric(number) => {
                number.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn to_json_map_key(&self) -> Result<String, std::io::Error>
    {
        match self {
            Self::FieldName{name, ..} => {
                Ok(String::from(name.as_str()))
            }
            Self::String(string) => {
                let serde_json::Value::String(string) = string.to_json()? else { unreachable!() };
                Ok(string)
            }
            Self::Numeric(number) => {
                let mut string = String::default();
                let mut cursor = Cursor::new(&mut string);
                // Drill into the number value, ignoring fill.
                number.number.write_to(&mut cursor)?;
                Ok(string)
            }
        }
    }

    pub fn field_name(name: impl AsRef<str>) -> Self
    {
        Self::FieldName{ fill: CafFill::default(), name: SmolStr::from(name.as_ref()) }
    }

    pub fn from_json_key(key: impl AsRef<str>) -> Self
    {
        // Try to convert a number.
        if let Ok(number) = CafNumber::from_json_string(key) {
            return Self::Numeric(number);
        }
        // Otherwise it must be a string.
        Self::String(CafString::from_json_string(key))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::FieldName{fill, ..}, Self::FieldName{fill: other_fill, ..}) => {
                fill.recover(other_fill);
            }
            (Self::String(string), Self::String(other_string)) => {
                string.recover_fill(other_string);
            }
            (Self::Numeric(number), Self::Numeric(other_number)) => {
                number.recover_fill(other_number);
            }
            _ => ()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafMapKeyValue
{
    key: CafMapKey,
    semicolon_fill: CafFill,  //todo: does allowing fill between key and semicolon create parsing ambiguities?
    value: CafValue,
}

impl CafMapKeyValue
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, '')
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: impl AsRef<str>) -> Result<(), std::io::Error>
    {
        self.key.write_to_with_space(writer, space)?;
        self.semicolon_fill.write_to(writer)?;
        writer.write(':'.as_bytes())?;
        self.value.write_to(writer)?;
        Ok(())
    }

    pub fn add_to_json(&self, map: &mut serde_json::Map) -> Result<(), std::io::Error>
    {
        map.insert(self.key.to_json_map_key()?, self.value.to_json()?);
        Ok(())
    }

    pub fn from_json(
        key: CafMapKey,
        val: &serde_json::Value,
        type_info: &TypeInfo,
        registry: &TypeRegistry
    ) -> Result<Self, String>
    {
        Ok(Self{ key, semicolon_fill: CafFill::default(), value: CafValue::from_json(val, type_info, registry)? })
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.key.recover_fill(&other.key);
        self.semicolon_fill.recover(&other.semicolon_fill);
        self.value.recover_fill(&other.value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafMapEntry
{
    KeyValue(CafMapKeyValue),
    /// Only catch-all params are allowed.
    MacroParam(CafMacroParam)
}

impl CafMapEntry
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, '')
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: impl AsRef<str>) -> Result<(), std::io::Error>
    {
        match self {
            Self::KeyValue(keyvalue) => {
                keyvalue.write_to_with_space(writer, space)?;
            }
            Self::MacroParam(param) => {
                param.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn add_to_json(&self, map: &mut serde_json::Map) -> Result<(), std::io::Error>
    {
        match self {
            Self::KeyValue(keyvalue) => {
                keyvalue.add_to_json(map)?;
            }
            Self::MacroParam(param) => {
                return Err(std::io::Error::other(format!(
                    "macro param {:?} in caf map entry when converting to JSON", param
                )));
            }
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::KeyValue(keyvalue), Self::KeyValue(other_keyvalue)) => {
                keyvalue.recover_fill(other_keyvalue);
            }
            (Self::MacroParam(param), Self::MacroParam(other_param)) => {
                param.recover_fill(other_param);
            }
            _ => ()
        }
    }
}

// Parsing:
// - only catch-all data macro params are allowed

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafMap
{
    /// Fill before opening `{`.
    pub start_fill: CafFill,
    pub entries: Vec<CafMapEntry>,
    /// Fill before ending `}`.
    pub end_fill: CafFill
}

impl CafMap
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write('{'.as_bytes())?;
        for (idx, entry) in self.entries.iter().enumerate() {
            if idx == 0 {
                entry.write_to(writer)?;
            } else {
                entry.write_to_with_space(writer, ' ')?;
            }
        }
        self.end_fill.write_to(writer)?;
        writer.write('}'.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut map = serde_json::Map::with_capacity(self.entries.len());
        for entry in self.entries.iter() {
            entry.add_to_json(&mut map)?;
        }
        Ok(serde_json::Value::Object(map))
    }

    pub fn from_json(val: &serde_json::Value, type_info: &TypeInfo, registry: &TypeRegistry) -> Result<Self, String>
    {
        let serde_json::Value::Object(json_map) = val else {
            return Err(format!(
                "failed converting {:?} from json {:?}; expected json to be a map",
                type_info.type_path(), val
            ));
        };

        match type_info {
            TypeInfo::Struct(info) => {
                let mut entries = Vec::with_capacity(json_map.len());
                // Note: we assume the "preserve_order" feature is enabled for serde_json.
                for (json_key, json_value) in json_map.iter() {
                    let Some(field) = info.field(json_key.as_str()) else {
                        return Err(format!(
                            "failed converting {:?} from json {:?} into a map; json contains unexpected field name {:?}",
                            type_info.type_path(), val, json_key
                        ));
                    };

                    let Some(registration) = type_registry.get(field.type_id()) else { unreachable!() };
                    entries.push(CafMapEntry::KeyValue(CafMapKeyValue::from_json(
                        // Struct maps have field name keys.
                        CafMapKey::field_name(json_key),
                        json_value,
                        registration.type_info(),
                        type_registry
                    )?));
                }
                Ok(Self{ start_fill: CafFill::default(), entries, end_fill: CafFill::default() })
            }
            TypeInfo::Map(_) => {
                let mut entries = Vec::with_capacity(json_map.len());
                // Note: we assume the "preserve_order" feature is enabled for serde_json.
                for (json_key, json_value) in json_map.iter() {
                    let Some(field) = info.field(json_key.as_str()) else {
                        return Err(format!(
                            "failed converting {:?} from json {:?} into a map; json contains unexpected field name {:?}",
                            type_info.type_path(), val, json_key
                        ));
                    };

                    let Some(registration) = type_registry.get(field.type_id()) else { unreachable!() };
                    entries.push(CafMapEntry::KeyValue(CafMapKeyValue::from_json(
                        // Plain maps have value keys.
                        CafMapKey::from_json_key(json_key),
                        json_value,
                        registration.type_info(),
                        type_registry
                    )?));
                }
                Ok(Self{ start_fill: CafFill::default(), entries, end_fill: CafFill::default() })
            }
            _ => Err(format!(
                "failed converting {:?} from json {:?} into a map; type is not a struct/map",
                val, type_info.type_path()
            ))
        }
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

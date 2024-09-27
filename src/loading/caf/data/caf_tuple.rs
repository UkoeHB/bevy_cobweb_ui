

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafTuple
{
    /// Fill before opening `(`.
    pub start_fill: CafFill,
    pub entries: Vec<CafValue>,
    /// Fill before ending `)`.
    pub end_fill: CafFill
}

impl CafTuple
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write('('.as_bytes())?;
        for (idx, entry) in self.entries.iter().enumerate() {
            if idx == 0 {
                entry.write_to(writer)?;
            } else {
                entry.write_to_with_space(writer, ' ')?;
            }
        }
        self.end_fill.write_to(writer)?;
        writer.write(')'.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut array = Vec::with_capacity(self.entries.len());
        for entry in self.entries.iter() {
            array.push(entry.to_json()?);
        }
        Ok(serde_json::Value::Array(array))
    }

    pub fn from_json(val: &serde_json::Value, type_info: &TypeInfo, registry: &TypeRegistry) -> Result<Self, String>
    {
        let serde_json::Value::Array(json_vec) = val else {
            return Err(format!(
                "failed converting {:?} from json {:?}; expected json to be an array",
                type_info.type_path(), val
            ));
        };

        match type_info {
            TypeInfo::TupleStruct(info) => {
                if info.field_len() != json_vec.len() {
                    return Err(format!(
                        "failed converting {:?} from json {:?} into a tuple; json does not match expected number \
                        of tuple fields",
                        val, type_info.type_path()
                    ));
                }

                let mut entries = Vec::with_capacity(json_vec.len());
                for (info, json_value) in info.iter().zip(json_vec.iter()) {
                    let Some(registration) = type_registry.get(info.type_id()) else { unreachable!() };
                    entries.push(CafValue::from_json(json_value, registration.type_info(), type_registry)?);
                }
                Ok(Self{ start_fill: CafFill::default(), entries, end_fill: CafFill::default() })
            }
            TypeInfo::Tuple(info) => {
                if info.field_len() != json_vec.len() {
                    return Err(format!(
                        "failed converting {:?} from json {:?} into a tuple; json does not match expected number \
                        of tuple fields",
                        val, type_info.type_path()
                    ));
                }

                let mut entries = Vec::with_capacity(json_vec.len());
                for (info, json_value) in info.iter().zip(json_vec.iter()) {
                    let Some(registration) = type_registry.get(info.type_id()) else { unreachable!() };
                    entries.push(CafValue::from_json(json_value, registration.type_info(), type_registry)?);
                }
                Ok(Self{ start_fill: CafFill::default(), entries, end_fill: CafFill::default() })
            }
            _ => Err(format!(
                "failed converting {:?} from json {:?} into a tuple; type is not a tuplestruct/tuple",
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

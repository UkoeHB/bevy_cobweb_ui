
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafHexColor
{
    pub fill: CafFill,
    pub original: SmolStr,
    pub color: bevy_color::Rbga,
}

impl CafHexColor
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.fill.write_to(writer)?;
        writer.write(self.original.as_bytes())?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut map = serde_json::Map::default();
        let key = "Rgba".into();
        let mut value = serde_json::Map::default();
        value.insert("red".into(), serde_json::Value::Number(serde_json::Number::from(self.color.red)));
        value.insert("green".into(), serde_json::Value::Number(serde_json::Number::from(self.color.green)));
        value.insert("blue".into(), serde_json::Value::Number(serde_json::Number::from(self.color.blue)));
        value.insert("alpha".into(), serde_json::Value::Number(serde_json::Number::from(self.color.alpha)));
        map.insert(key, serde_json::Value::Object(value));
        Ok(serde_json::Value::Object(map))
    }
}

/*
Parsing:
- proper hex format with optional alpha at the beginning
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafBuiltin
{
    Color(CafHexColor),
    Val{
        fill: CafFill,
        /// There is no number for `Val::Auto`.
        number: Option<CafNumberValue>,
        val: Val
    }
}

impl CafBuiltin
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Color(color) => {
                color.write_to(writer)?;
            }
            Self::Val{number, val} => {
                fill.write_to(writer)?;
                if let Some(number) = number {
                    number.write_to(writer)?;
                }
                match val {
                    Auto => {
                        writer.write("auto".as_bytes())?;
                    }
                    Val::Percent(..) => {
                        writer.write("%".as_bytes())?;
                    }
                    Val::Px(..) => {
                        writer.write("px".as_bytes())?;
                    }
                    Val::Vw(..) => {
                        writer.write("vw".as_bytes())?;
                    }
                    Val::Vh(..) => {
                        writer.write("vh".as_bytes())?;
                    }
                    Val::VMin(..) => {
                        writer.write("vmin".as_bytes())?;
                    }
                    Val::VMax(..) => {
                        writer.write("vmax".as_bytes())?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        match self {
            Self::Color(color) => {
                color.to_json()
            }
            Self::Val{val, ..} => {
                let make_val = |name: &str, val: f32| -> serde_json::Value {
                    let mut map = serde_json::Map::default();
                    let key = name.into();
                    let value = serde_json::Number::from(val);
                    map.insert(key, value);
                    serde_json::Value::Object(map)
                };
                match val {
                    Val::Auto => {
                        Ok(serde_json::Value::String("Auto".into()))
                    }
                    Val::Px(val) => {
                        Ok(make_val("Px", val))
                    }
                    Val::Percent(val) => {
                        Ok(make_val("Percent", val))
                    }
                    Val::Vw(val) => {
                        Ok(make_val("Vw", val))
                    }
                    Val::Vh(val) => {
                        Ok(make_val("Vh", val))
                    }
                    Val::VMin(val) => {
                        Ok(make_val("VMin", val))
                    }
                    Val::VMax(val) => {
                        Ok(make_val("VMax", val))
                    }
                }
            }
        }
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

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Color(color), Self::Color(other_color)) => {
                color.recover_fill(other_color);
            }
            (Self::Val{fill, ..}, Self::Val{fill: other_fill, ..}) => {
                fill.recover(&other.fill);
            }
            _ => ()
        }
    }
}

// Parsing:
// - Allow both uints and floats for Val settings. (looks like uints coerce to floats on Value deserialization)

//-------------------------------------------------------------------------------------------------------------------

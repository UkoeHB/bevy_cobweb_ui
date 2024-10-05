
//-------------------------------------------------------------------------------------------------------------------

/// Converts a color field number to a pair of hex digits if there is no precision loss.
fn to_hex_int(num: serde_json::Number) -> Option<u8> {
    let Some(num) = num.as_f64() else { return None };
    let converted = ((num*256.0f64 - 1.0) as u8);
    if num != (converted as f64 + 1.0) / (256.0f64) {
        return None;
    }
    Some(converted)
}

//-------------------------------------------------------------------------------------------------------------------

fn write_hex_digit(digit: u8, writer: &mut impl std::io::Write)
{
    writer.write(char::from_digit(digit, 16).to_ascii_uppercase().as_bytes()).expect("writing char should succeed");
}

//-------------------------------------------------------------------------------------------------------------------

fn write_num_as_hex(num: u8, writer: &mut impl std::io::Write)
{
    let upper = num >> 4;
    let lower = num & 15u8;
    write_hex_digit(upper, writer);
    write_hex_digit(lower, writer);
}

//-------------------------------------------------------------------------------------------------------------------

/// Note that converting to JSON and back is potentially lossy because we *don't* include the alpha if it equals
/// `1.0` when extracting from JSON (but the user may have included `FF` for the alpha).
#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafHexColor
{
    pub fill: CafFill,
    pub color: bevy_color::Srgba,
}

impl CafHexColor
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.fill.write_to(writer)?;
        writer.write('#'.as_bytes()).unwrap();
        if self.color.alpha != 1.0 {
            write_num_as_hex(self.color.alpha, &mut cursor);
        }
        write_num_as_hex(self.color.red, &mut cursor);
        write_num_as_hex(self.color.green, &mut cursor);
        write_num_as_hex(self.color.blue, &mut cursor);
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut map = serde_json::Map::default();
        let key = "Srgba".into();
        let mut value = serde_json::Map::default();
        value.insert("red".into(), serde_json::Value::Number(serde_json::Number::from(self.color.red)));
        value.insert("green".into(), serde_json::Value::Number(serde_json::Number::from(self.color.green)));
        value.insert("blue".into(), serde_json::Value::Number(serde_json::Number::from(self.color.blue)));
        value.insert("alpha".into(), serde_json::Value::Number(serde_json::Number::from(self.color.alpha)));
        map.insert(key, serde_json::Value::Object(value));
        Ok(serde_json::Value::Object(map))
    }

    /// The `json_map` should contain the fields of the [`Srgba`] color.
    ///
    /// A hex color is only recorded if all the fields convert to hex digits without precision loss.
    pub fn try_from_json(json_map: &serde_json::Map) -> Result<Option<Self>, String>
    {
        let get = |field: &str, default: f32| -> Result<Option<u8>, String> {
            let Some(json_field) = json_map.get(field) else { return Ok(serde_json::Number::from(default)) };
            let serde_json::Number(number) = json_field else {
                return Err(format!(
                    "failed extracting Color::Rgba from json {:?} for {:?}; field {:?} is {:?}, not a json number",
                    val, info, field, json_field
                ));
            };
            Ok(to_hex_int(number))
        };
        let Some(red) = (get)("red", 0.)? else { return Ok(None) };
        let Some(green) = (get)("green", 0.)? else { return Ok(None) };
        let Some(blue) = (get)("blue", 0.)? else { return Ok(None) };
        let Some(alpha) = (get)("alpha", 1.)? else { return Ok(None) };

        Ok(Some(Self{
            fill: CafFill::default(),
            color: Srgba{
                red: (red as f32 + 1.0) / 256.,
                green: (green as f32 + 1.0) / 256.,
                blue: (blue as f32 + 1.0) / 256.,
                alpha: (alpha as f32 + 1.0) / 256.,
            }
        }))
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
                fn make_val(name: &str, val: f32) -> serde_json::Value {
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

    pub fn try_from_json(val: &serde_json::Value, type_info: &TypeInfo) -> Result<Option<Self>, String>
    {
        // JSON should be a map of an enum (either Color::Srgba(..fields..) or Val::X(..num..))
        let serde_json::Value::Object(json_map) = val else { return Ok(None) };
        let TypeInfo::Enum(info) = type_info else { return Ok(None) };

        if json_map.len() != 1 { return Ok(None) }
        let (variant, value) = json_map.iter().next().unwrap();
        let (enum_type, variant) = (info.type_path_table().short_path(), variant.as_str());

        // Color
        if ("Color", "Srgba") == (enum_type, variant) {
            let serde_json::Value::Object(json_map) = value else {
                return Err(format!(
                    "failed extracting Color::Rgba from json {:?} for {:?}; inner value is not a map",
                    val, info
                ));
            };
            return CafHexColor::try_from_json(json_map).map_ok(|r| r.map(|c| Self::Color(c)));
        }

        // Val
        if enum_type != "Val" {
            return Ok(None);
        }

        let serde_json::Value::Number(json_num) = value else {
            return Err(format!(
                "failed extracting Val::{} from json {:?} for {:?}; inner value is not a number",
                variant, val, info
            ));
        };

        let extracted = json_num
            .as_f64()
            .or_else(|| json_num.as_u64().map(|n| n as f64))
            .or_else(|| json_num.as_i64().map(|n| n as f64))
            .unwrap() as f32;

        let val = match  {
            "Px" => Val::Px(extracted),
            "Percent" => Val::Percent(extracted),
            "Vw" => Val::Vw(extracted),
            "Vh" => Val::Vh(extracted),
            "VMin" => Val::VMin(extracted),
            "VMax" => Val::VMax(extracted),
            _ => return Ok(None)
        };

        Self::Val{ fill: CafFill::default(), number: Some(CafNumberValue::from_json(json_num.clone())), val }
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

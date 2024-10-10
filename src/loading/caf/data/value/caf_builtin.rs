use bevy::prelude::*;
use bevy::reflect::TypeInfo;
use zerocopy::IntoBytes;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Converts a color field number to a pair of hex digits if there is no precision loss.
fn to_hex_int(num: f64) -> Option<u8>
{
    let converted = (num * 256.0f64 - 1.0) as u8;
    if num as f32 != ((converted as f64 + 1.0) / (256.0f64)) as f32 {
        return None;
    }
    Some(converted)
}

//-------------------------------------------------------------------------------------------------------------------

/// Converts a color field number to a pair of hex digits if there is no precision loss.
fn json_to_hex_int(num: &serde_json::Number) -> Option<u8>
{
    let Some(num) = num.as_f64() else { return None };
    to_hex_int(num)
}

//-------------------------------------------------------------------------------------------------------------------

fn write_hex_digit(digit: u8, writer: &mut impl std::io::Write)
{
    writer
        .write(
            char::from_digit(digit as u32, 16)
                .unwrap()
                .to_ascii_uppercase()
                .as_bytes(),
        )
        .expect("writing char should succeed");
}

//-------------------------------------------------------------------------------------------------------------------

fn write_num_as_hex(num: u8, writer: &mut impl std::io::Write)
{
    let upper = num >> 4;
    let lower = num & 0xF;
    write_hex_digit(upper, writer);
    write_hex_digit(lower, writer);
}

//-------------------------------------------------------------------------------------------------------------------

/// Note that converting to JSON and back is potentially lossy because we *don't* include the alpha if it equals
/// `1.0` when extracting from JSON (but the user may have included `FF` for the alpha).
#[derive(Debug, Clone, PartialEq)]
pub struct CafHexColor
{
    pub fill: CafFill,
    pub color: Srgba,
}

impl CafHexColor
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str)
        -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write("#".as_bytes()).unwrap();
        if self.color.alpha != 1.0 {
            write_num_as_hex((self.color.alpha - 1. / 256.) as u8, writer);
        }
        write_num_as_hex((self.color.red - 1. / 256.) as u8, writer);
        write_num_as_hex((self.color.green - 1. / 256.) as u8, writer);
        write_num_as_hex((self.color.blue - 1. / 256.) as u8, writer);
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        let mut map = serde_json::Map::<String, serde_json::Value>::default();
        let key = "Srgba".into();
        let mut value = serde_json::Map::<String, serde_json::Value>::default();
        value.insert(
            "red".into(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.color.red as f64).expect("color should be finite"),
            ),
        );
        value.insert(
            "green".into(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.color.green as f64).expect("color should be finite"),
            ),
        );
        value.insert(
            "blue".into(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.color.blue as f64).expect("color should be finite"),
            ),
        );
        value.insert(
            "alpha".into(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(self.color.alpha as f64).expect("color should be finite"),
            ),
        );
        map.insert(key, serde_json::Value::Object(value));
        Ok(serde_json::Value::Object(map))
    }

    /// The `json_map` should contain the fields of the [`Srgba`] color.
    ///
    /// A hex color is only recorded if all the fields convert to hex digits without precision loss.
    pub fn try_from_json(json_map: &serde_json::Map<String, serde_json::Value>) -> Result<Option<Self>, String>
    {
        let get = |field: &str, default: u8| -> Result<Option<u8>, String> {
            let Some(json_field) = json_map.get(field) else { return Ok(Some(default)) };
            let serde_json::Value::Number(number) = json_field else {
                return Err(format!(
                    "failed extracting Color::Rgba from json {:?}; field {:?} is {:?}, not a json number",
                    json_map, field, json_field
                ));
            };
            Ok(json_to_hex_int(&number))
        };
        let Some(red) = (get)("red", 0)? else { return Ok(None) };
        let Some(green) = (get)("green", 0)? else { return Ok(None) };
        let Some(blue) = (get)("blue", 0)? else { return Ok(None) };
        let Some(alpha) = (get)("alpha", 1)? else { return Ok(None) };

        Ok(Some(Self {
            fill: CafFill::default(),
            color: Srgba {
                red: (red as f32 + 1.0) / 256.,
                green: (green as f32 + 1.0) / 256.,
                blue: (blue as f32 + 1.0) / 256.,
                alpha: (alpha as f32 + 1.0) / 256.,
            },
        }))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

impl TryFrom<Srgba> for CafHexColor
{
    type Error = ();

    fn try_from(value: Srgba) -> Result<Self, ()>
    {
        if to_hex_int(value.red as f64).is_none()
            || to_hex_int(value.green as f64).is_none()
            || to_hex_int(value.blue as f64).is_none()
            || to_hex_int(value.alpha as f64).is_none()
        {
            return Err(());
        }
        Ok(Self { fill: CafFill::default(), color: value })
    }
}

/*
Parsing:
- proper hex format with optional alpha at the beginning
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafBuiltin
{
    Color(CafHexColor),
    Val
    {
        fill: CafFill,
        /// There is no number for `Val::Auto`.
        number: Option<CafNumberValue>,
        val: Val,
    },
}

impl CafBuiltin
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str)
        -> Result<(), std::io::Error>
    {
        match self {
            Self::Color(color) => {
                color.write_to_with_space(writer, space)?;
            }
            Self::Val { fill, number, val } => {
                fill.write_to_or_else(writer, space)?;
                if let Some(number) = number {
                    number.write_to(writer)?;
                }
                match val {
                    Val::Auto => {
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
            Self::Color(color) => color.to_json(),
            Self::Val { val, .. } => {
                fn make_val(name: &str, val: f32) -> serde_json::Value
                {
                    let mut map = serde_json::Map::<String, serde_json::Value>::default();
                    let key = name.into();
                    let value = serde_json::Value::Number(
                        serde_json::Number::from_f64(val as f64).unwrap_or(serde_json::Number::from(0)),
                    );
                    map.insert(key, value);
                    serde_json::Value::Object(map)
                }
                match *val {
                    Val::Auto => Ok(serde_json::Value::String("Auto".into())),
                    Val::Px(val) => Ok(make_val("Px", val)),
                    Val::Percent(val) => Ok(make_val("Percent", val)),
                    Val::Vw(val) => Ok(make_val("Vw", val)),
                    Val::Vh(val) => Ok(make_val("Vh", val)),
                    Val::VMin(val) => Ok(make_val("VMin", val)),
                    Val::VMax(val) => Ok(make_val("VMax", val)),
                }
            }
        }
    }

    pub fn try_from_json(val: &serde_json::Value, type_info: &TypeInfo) -> Result<Option<Self>, String>
    {
        // JSON should be a map of an enum (either Color::Srgba(..fields..) or Val::X(..num..))
        let serde_json::Value::Object(json_map) = val else { return Ok(None) };
        let TypeInfo::Enum(info) = type_info else { return Ok(None) };

        if json_map.len() != 1 {
            return Ok(None);
        }
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
            return CafHexColor::try_from_json(json_map).map(|r| r.map(|c| Self::Color(c)));
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

        let val = match enum_type {
            "Px" => Val::Px(extracted),
            "Percent" => Val::Percent(extracted),
            "Vw" => Val::Vw(extracted),
            "Vh" => Val::Vh(extracted),
            "VMin" => Val::VMin(extracted),
            "VMax" => Val::VMax(extracted),
            _ => return Ok(None),
        };

        Ok(Some(Self::Val {
            fill: CafFill::default(),
            number: Some(CafNumberValue::from_json_number(json_num.clone())),
            val,
        }))
    }

    pub fn try_from_unit_variant(typename: &str, variant: &str) -> CafResult<Option<Self>>
    {
        if typename == "Val" && variant == "Auto" {
            return Ok(Some(Self::Val {
                fill: CafFill::default(),
                number: None,
                val: Val::Auto,
            }));
        }

        Ok(None)
    }

    /// The value should not contain any macros/constants.
    pub fn try_from_newtype_variant(typename: &str, variant: &str, value: &CafValue) -> CafResult<Option<Self>>
    {
        if typename == "Color" && variant == "Srgba" {
            let CafValue::EnumVariant(CafEnumVariant::Map { id, map }) = value else { return Ok(None) };
            if id.name != "Srgba" {
                return Ok(None);
            }
            let mut color = Srgba::default();
            for entry in map.entries.iter() {
                let CafMapEntry::KeyValue(keyval) = entry else { return Err(CafError::MalformedBuiltin) };
                let CafMapKey::FieldName { fill: _, name } = &keyval.key else {
                    return Err(CafError::MalformedBuiltin);
                };
                let CafValue::Number(num) = &keyval.value else { return Ok(None) };
                let Some(float) = num.number.deserialized.as_f64() else { return Ok(None) };
                let value = float as f32;

                if name == "red" {
                    color.red = value;
                } else if name == "green" {
                    color.green = value;
                } else if name == "blue" {
                    color.blue = value;
                } else if name == "alpha" {
                    color.alpha = value;
                } else {
                    return Ok(None);
                }
            }

            return Ok(CafHexColor::try_from(color).map(|c| Self::Color(c)).ok());
        }

        if typename == "Val" {
            let CafValue::Number(num) = value else { return Ok(None) };
            let Some(float) = num.number.deserialized.as_f64() else { return Ok(None) };
            let extracted = float as f32;

            let val = match variant {
                "Px" => Val::Px(extracted),
                "Percent" => Val::Percent(extracted),
                "Vw" => Val::Vw(extracted),
                "Vh" => Val::Vh(extracted),
                "VMin" => Val::VMin(extracted),
                "VMax" => Val::VMax(extracted),
                _ => return Err(CafError::MalformedBuiltin),
            };

            return Ok(Some(Self::Val {
                fill: CafFill::default(),
                number: Some(num.number.clone()),
                val,
            }));
        }

        Ok(None)
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Color(color), Self::Color(other_color)) => {
                color.recover_fill(other_color);
            }
            (Self::Val { fill, .. }, Self::Val { fill: other_fill, .. }) => {
                fill.recover(&other_fill);
            }
            _ => (),
        }
    }
}

// Parsing:
// - Allow both uints and floats for Val settings. (looks like uints coerce to floats on Value deserialization)

//-------------------------------------------------------------------------------------------------------------------

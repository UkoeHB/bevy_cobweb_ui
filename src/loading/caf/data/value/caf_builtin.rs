use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Converts a color field number to a pair of hex digits if there is no precision loss.
fn to_hex_int(num: f64) -> Option<u8>
{
    let converted = (num * 256.0f64 - 1.0) as u8;
    let left = num as f32;
    let right = ((converted as f64 + 1.0) / (256.0f64)) as f32;
    if left != right {
        return None;
    }
    Some(converted)
}

//-------------------------------------------------------------------------------------------------------------------

fn write_num_as_hex(num: u8, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
{
    write!(writer, "{:02X}", num)
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
        println!("PRINTING COLOR: {:?}", self.color);
        self.fill.write_to_or_else(writer, space)?;
        writer.write("#".as_bytes()).unwrap();
        if self.color.alpha != 1.0 {
            write_num_as_hex((self.color.alpha * 255.) as u8, writer)?;
        }
        write_num_as_hex((self.color.red * 255.) as u8, writer)?;
        write_num_as_hex((self.color.green * 255.) as u8, writer)?;
        write_num_as_hex((self.color.blue * 255.) as u8, writer)?;
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(&other.fill);
    }
}

impl TryFrom<Srgba> for CafHexColor
{
    type Error = ();

    /// Only succeeds if all fields can be converted to hex without precision loss.
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
                    number.write_to_simplified(writer)?;
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
            println!("COLOR");
            let CafValue::Map(CafMap { entries, .. }) = value else { return Ok(None) };
            println!("ENTRIES");
            let mut color = Srgba::default();
            for entry in entries.iter() {
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

            println!("HEX COLOR");
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


//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafHexColor
{
    pub fill: CafFill,
    pub original: Arc<str>,
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
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafStructBuiltin
{
    Color(CafHexColor),
    Val{
        fill: CafFill,
        /// There is no number for `Val::Auto`.
        number: Option<CafNumberValue>,
        val: Val
    }
}

impl CafStructBuiltin
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
}

//-------------------------------------------------------------------------------------------------------------------

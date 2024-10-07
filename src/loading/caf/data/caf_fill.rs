use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

/// Section of a CAF file containing filler charaters.
///
/// Includes whitespace (spaces and newlines), comments (line and block comments), and ignored characters (commas
/// and semicolons).
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafFill
{
    // TODO: replace with Cow of string slice
    pub string: SmolStr,
}

impl CafFill
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write(self.string.as_bytes())?;
        Ok(())
    }

    pub fn write_to_or_else(
        &self,
        writer: &mut impl std::io::Write,
        fallback: impl AsRef<str>,
    ) -> Result<(), std::io::Error>
    {
        if self.string.len() == 0 {
            writer.write(fallback.as_ref().as_bytes())?;
        } else {
            self.write_to(writer)?;
        }
        Ok(())
    }

    /// Parse the current read source into a filler. The filler may be empty.
    ///
    /// Errors if an illegal character is encountered.
    //pub fn parse() -> IResult<, (Self, )>
    //{
    // TODO
    // Scan for:
    // - Spaces
    // - Newlines
    // - Line comments
    // - Block comments
    // - Ignored characters: ,;
    // - Banned characters: (see sublime-syntax)
    //}

    pub fn new(string: impl Into<SmolStr>) -> Self
    {
        Self { string: string.into() }
    }

    pub fn len(&self) -> usize
    {
        self.string.len()
    }

    /// Number of space characters at the end of the filler if it ends in '\n   '.
    ///
    /// Used to calibrate scene tree depth of scene nodes.
    pub fn ends_newline_then_num_spaces(&self) -> Option<usize>
    {
        let trimmed = self.string.as_str().trim_end_matches(' ');
        if !trimmed.ends_with('\n') {
            return None;
        }
        Some(self.len() - trimmed.len())
    }

    /// Checks if the filler ends in a newline.
    pub fn ends_with_newline(&self) -> bool
    {
        self.string.as_str().ends_with('\n')
    }

    /// If `self.len() == 0` then clone the other's fill value.
    pub fn recover(&mut self, other: &Self)
    {
        if self.len() == 0 {
            *self = other.clone();
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

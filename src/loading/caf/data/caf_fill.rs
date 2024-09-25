use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafFillSegment
{
    /// Spaces and newlines (\n characters).
    ///
    /// Carriage returns and tabs are not supported.
    Whitespace(SmolStr),
    /// Commas treated as whitespace.
    FloatingCommas(usize),
    /// A comment can contain: `//`, `text` (optional: 1 terminal newline). We store the text here.
    LineComment(String),
    /// A block comment contains: `/*`, `text` (optional: newlines anywhere), `*/`. We store the text here.
    BlockComment(String),
}

impl CafFillSegment
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Whitespace(space) => writer.write(space.as_str().as_bytes())?,
            Self::FloatingCommas(count) => {
                for _ in 0..count {
                    writer.write(','.as_bytes())?;
                }
            }
            Self::LineComment(comment) =>
            {
                writer.write(comment.as_str().as_bytes())?
            }
            Self::BlockComment(comment) => writer.write(comment.as_str().as_bytes())?,
        };
        Ok(())
    }

    pub fn write_to_or_else(&self, writer: &mut impl std::io::Write, or: impl AsRef<str>) -> Result<(), std::io::Error>
    {
        if self.len() > 0 {
            self.write_to(writer)?;
        } else {
            writer.write(or.as_ref().as_bytes())?;
        }
    }

    pub fn newline() -> Self
    {
        Self::Whitespace(SmolStr::from("\n"))
    }

    pub fn newlines(num: usize) -> Self
    {
        Self::Whitespace(SmolStr::from("\n".repeat(num)))
    }

    pub fn space() -> Self
    {
        Self::Whitespace(SmolStr::from(" "))
    }

    pub fn spaces(num: usize) -> Self
    {
        Self::Whitespace(SmolStr::from(" ".repeat(num)))
    }

    pub fn comment(comment: impl AsRef<str>) -> Self
    {
        Self::LineComment(String::from(comment.as_ref()))
    }

    pub fn block_comment(comment: impl AsRef<str>) -> Self
    {
        Self::BlockComment(String::from(comment.as_ref()))
    }

    pub fn ends_with_newline(&self) -> bool
    {
        let Self::Whitespace(space) = self else { return false };
        space.as_str().ends_with('\n')
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Section of a CAF file containing filler charaters.
///
/// Includes whitespace (spaces and newlines), comments (line and block comments), and ignored characters (commas
/// and semicolons).
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CafFill
{
    // TODO: replace with Cow of string slice
    pub string: String
}

impl CafFill
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write(&self.string)?;
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
        // - Banned characters: [^A-Za-z0-9_ \n;,\{\}\[\]\(\)<>\-+=$#@!%*?\\:\/."']
    //}

    pub fn new(string: impl Into<String>) -> Self
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
        let trimmed = self.string.as_str.trim_end_matches(' ');
        if !trimmed.ends_with('\n') { return None }
        Some(self.len() - trimmed.len())
    }

    /// Checks if the filler ends in a newline.
    pub fn ends_with_newline(&self) -> bool
    {
        self.string.as_str().ends_with('\n')
    }
}

//-------------------------------------------------------------------------------------------------------------------

use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafFillSegment
{
    Whitespace(SmolStr),
    LineComment(String),
    BlockComment(String),
}

impl CafFillSegment
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Whitespace(space) => writer.write(space.as_str().as_bytes())?,
            Self::LineComment(comment) => writer.write(comment.as_str().as_bytes())?,
            Self::BlockComment(comment) => writer.write(comment.as_str().as_bytes())?,
        };
        Ok(())
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

#[derive(Debug, Clone, PartialEq)]
pub struct CafFill
{
    pub segments: Vec<CafFillSegment>,
}

impl CafFill
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        for segment in self.segments.iter() {
            segment.write_to(writer)?;
        }
        Ok(())
    }

    pub fn newline() -> Self
    {
        Self { segments: vec![CafFillSegment::newline()] }
    }

    pub fn newlines(num: usize) -> Self
    {
        Self { segments: vec![CafFillSegment::newlines(num)] }
    }

    pub fn space() -> Self
    {
        Self { segments: vec![CafFillSegment::space()] }
    }

    pub fn spaces(num: usize) -> Self
    {
        Self { segments: vec![CafFillSegment::spaces(num)] }
    }

    pub fn comment(comment: impl AsRef<str>) -> Self
    {
        Self { segments: vec![CafFillSegment::comment(comment)] }
    }

    pub fn block_comment(comment: impl AsRef<str>) -> Self
    {
        Self { segments: vec![CafFillSegment::block_comment(comment)] }
    }

    pub fn ends_with_newline(&self) -> bool
    {
        self.segments
            .last()
            .map(|s| s.ends_with_newline())
            .unwrap_or(false)
    }
}

impl Default for CafFill
{
    fn default() -> Self
    {
        Self { segments: Vec::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

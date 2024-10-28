use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafSection
{
    Manifest(CafManifest),
    Import(CafImport),
    Using(CafUsing),
    Defs(CafDefs),
    Commands(CafCommands),
    Scenes(CafScenes),
}

impl CafSection
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        match self {
            Self::Manifest(section) => section.write_to(first_section, writer),
            Self::Import(section) => section.write_to(first_section, writer),
            Self::Using(section) => section.write_to(first_section, writer),
            Self::Defs(section) => section.write_to(first_section, writer),
            Self::Commands(section) => section.write_to(first_section, writer),
            Self::Scenes(section) => section.write_to(first_section, writer),
        }
    }

    /// Tries to parse a section from the available content.
    pub fn try_parse(fill: CafFill, content: Span) -> Result<(Option<Self>, CafFill, Span), SpanError>
    {
        let fill = match CafManifest::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Manifest(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match CafImport::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Import(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match CafUsing::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Using(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match CafDefs::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Defs(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match CafCommands::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Commands(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match CafScenes::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Scenes(section)), fill, remaining)),
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Caf
{
    /// Location of the caf file within the project's `assets` directory.
    ///
    /// Must have the `.caf` extension.
    pub file: CafFile,
    pub sections: Vec<CafSection>,
    /// Whitespace and comments at the end of the file.
    pub end_fill: CafFill,
}

impl Caf
{
    /// Will automatically insert a newline at the end if one is missing.
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        for (idx, section) in self.sections.iter().enumerate() {
            section.write_to(idx == 0, writer)?;
        }
        let ends_newline = self.end_fill.ends_with_newline();
        self.end_fill.write_to(writer)?;
        if !ends_newline {
            writer.write_bytes("\n".as_bytes())?;
        }
        Ok(())
    }

    pub fn parse(span: Span) -> Result<Self, SpanError>
    {
        let Some(file) = CafFile::try_new(span.extra.file) else {
            tracing::warn!("failed parsing CAF file at {}; file name doesn't end with '.caf'",
                get_location(span).as_str());
            return Err(span_verify_error(span));
        };

        let mut sections = vec![];
        let (mut fill, mut remaining) = CafFill::parse(span);

        let end_fill = loop {
            match CafSection::try_parse(fill, remaining)? {
                (Some(section), next_fill, after_section) => {
                    sections.push(section);
                    fill = next_fill;
                    remaining = after_section;
                }
                (None, end_fill, end_of_file) => {
                    if end_of_file.len() != 0 {
                        tracing::warn!("incomplete CAF file parsing, error at {}", get_location(end_of_file).as_str());
                        return Err(span_verify_error(end_of_file));
                    }

                    break end_fill;
                }
            }
        };

        Ok(Self { file, sections, end_fill })
    }
}

//-------------------------------------------------------------------------------------------------------------------

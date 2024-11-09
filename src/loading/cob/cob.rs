use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobSection
{
    Manifest(CobManifest),
    Import(CobImport),
    Using(CobUsing),
    Defs(CobDefs),
    Commands(CobCommands),
    Scenes(CobScenes),
}

impl CobSection
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
    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let fill = match CobManifest::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Manifest(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match CobImport::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Import(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match CobUsing::try_parse(fill, content)? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Using(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobDefs::try_parse(fill, c))? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Defs(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobCommands::try_parse(fill, c))? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Commands(section)), fill, remaining)),
            (None, fill, _) => fill,
        };
        let fill = match rc(content, move |c| CobScenes::try_parse(fill, c))? {
            (Some(section), fill, remaining) => return Ok((Some(Self::Scenes(section)), fill, remaining)),
            (None, fill, _) => fill,
        };

        Ok((None, fill, content))
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Cob
{
    /// Location of the cob file within the project's `assets` directory.
    ///
    /// Must have the `.cob` extension.
    pub file: CobFile,
    pub sections: Vec<CobSection>,
    /// Whitespace and comments at the end of the file.
    pub end_fill: CobFill,
}

impl Cob
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
        let Some(file) = CobFile::try_new(span.extra.file) else {
            tracing::warn!("failed parsing COB file at {}; file name doesn't end with '.cob'",
                get_location(span).as_str());
            return Err(span_verify_error(span));
        };

        debug_assert_eq!(get_local_recursion_count(), 0);

        let mut sections = vec![];
        let (mut fill, mut remaining) = CobFill::parse(span);

        let end_fill = loop {
            match rc(remaining, move |rm| CobSection::try_parse(fill, rm))? {
                (Some(section), next_fill, after_section) => {
                    sections.push(section);
                    fill = next_fill;
                    remaining = after_section;
                }
                (None, end_fill, end_of_file) => {
                    if end_of_file.len() != 0 {
                        tracing::warn!("incomplete COB file parsing, error at {}", get_location(end_of_file).as_str());
                        return Err(span_verify_error(end_of_file));
                    }

                    break end_fill;
                }
            }
        };

        debug_assert_eq!(get_local_recursion_count(), 0);

        Ok(Self { file, sections, end_fill })
    }
}

//-------------------------------------------------------------------------------------------------------------------

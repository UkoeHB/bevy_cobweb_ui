use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CafSection
{
    Manifest(CafManifest),
    Imports(CafImports),
    Using(CafUsing),
    Macros(CafMacros),
    Constants(CafConstants),
    Specs(CafSpecs),
    Scene(CafScene),
}

impl CafSection
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Manifest(section) => section.write_to(writer),
            Self::Imports(section) => section.write_to(writer),
            Self::Using(section) => section.write_to(writer),
            Self::Macros(section) => section.write_to(writer),
            Self::Constants(section) => section.write_to(writer),
            Self::Specs(section) => section.write_to(writer),
            Self::Scene(section) => section.write_to(writer),
        }
    }

    pub fn eq_ignore_whitespace(&self, other: &CafSection) -> bool
    {
        match self {
            Self::Manifest(section) => {
                let Self::Manifest(other) = other else { return false };
                section.eq_ignore_whitespace(other)
            }
            Self::Imports(section) => {
                let Self::Imports(other) = other else { return false };
                section.eq_ignore_whitespace(other)
            }
            Self::Using(section) => {
                let Self::Using(other) = other else { return false };
                section.eq_ignore_whitespace(other)
            }
            Self::Macros(section) => {
                let Self::Macros(other) = other else { return false };
                section.eq_ignore_whitespace(other)
            }
            Self::Constants(section) => {
                let Self::Constants(other) = other else { return false };
                section.eq_ignore_whitespace(other)
            }
            Self::Specs(section) => {
                let Self::Specs(other) = other else { return false };
                section.eq_ignore_whitespace(other)
            }
            Self::Scene(section) => {
                let Self::Scene(other) = other else { return false };
                section.eq_ignore_whitespace(other)
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Caf
{
    pub sections: Vec<CafSection>,
    /// Whitespace and comments at the end of the file.
    pub end_fill: CafFill,
    pub metadata: CafMetadata,
}

impl Caf
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        for section in &self.sections {
            section.write_to(writer)?;
        }
        self.end_fill.write_to(writer)?;
        Ok(())
    }

    /// Insert a newline to the end if there is none.
    ///
    /// This is useful cleanup when writing back to a file. Github, for example, likes a newline at the end.
    pub fn end_with_newline(&mut self)
    {
        if !self.end_fill.ends_with_newline() {
            self.end_fill.segments.push(CafFillSegment::newline());
        }
    }

    pub fn eq_ignore_whitespace(&self, other: &Caf) -> bool
    {
        if self.sections.len() != other.sections.len() {
            return false;
        }
        !self
            .sections
            .iter()
            .zip(other.sections.iter())
            .any(|(a, b)| !a.eq_ignore_whitespace(b))
    }
}

//-------------------------------------------------------------------------------------------------------------------

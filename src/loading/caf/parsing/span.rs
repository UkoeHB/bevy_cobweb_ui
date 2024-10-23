//-------------------------------------------------------------------------------------------------------------------

use nom_locate::LocatedSpan;

/// Metadata passed along with [`Span`] for error messages.
#[derive(Debug, Copy, Clone)]
pub struct CafLocationMetadata<'a>
{
    /// The name of the CAF file being parsed.
    pub file: &'a str,
}

//-------------------------------------------------------------------------------------------------------------------

/// Type alias for [`LocatedSpan`]. Used in [`Caf`] parsing for identifying the location of errors.
pub type Span<'a> = LocatedSpan<&'a str, CafLocationMetadata<'a>>;

//-------------------------------------------------------------------------------------------------------------------

/// Converts a [`Span`] to a formatted location.
pub fn get_location(span: Span) -> String
{
    format!("file: {}, line: {}, column: {}", span.extra.file, span.location_line(), span.get_utf8_column())
}

//-------------------------------------------------------------------------------------------------------------------

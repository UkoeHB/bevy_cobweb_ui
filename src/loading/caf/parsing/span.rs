use nom::error::ErrorKind;
use nom_locate::LocatedSpan;

//-------------------------------------------------------------------------------------------------------------------

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

/// Type alias for span errors.
pub type SpanError<'a> = nom::Err<nom::error::Error<Span<'a>>>;

//-------------------------------------------------------------------------------------------------------------------

/// Converts a [`Span`] to a formatted location.
pub fn get_location(span: Span) -> String
{
    format!("file: {}, line: {}, column: {}", span.extra.file, span.location_line(), span.get_utf8_column())
}

//-------------------------------------------------------------------------------------------------------------------

/// Makes a [`SpanError`] for a specific error code while parsing.
pub fn span_error(content: Span, code: ErrorKind) -> SpanError
{
    nom::Err::Error(nom::error::Error { input: content, code })
}

//-------------------------------------------------------------------------------------------------------------------

/// Makes a [`SpanError`] for a verification error while parsing.
pub fn span_verify_error(content: Span) -> SpanError
{
    span_error(content, ErrorKind::Verify)
}

//-------------------------------------------------------------------------------------------------------------------

/// Extracts the span that a [`SpanError`] references.
pub fn unwrap_error_content(error: SpanError) -> Span
{
    let nom::Err::Error(nom::error::Error { input, .. }) = error else {
        panic!("failed unwrapping span error content from {error:?}");
    };
    input
}

//-------------------------------------------------------------------------------------------------------------------

use std::fmt::Display;
use std::io::ErrorKind;

//-------------------------------------------------------------------------------------------------------------------

/// Result for the [`CobLoadable`](crate::prelude::CobLoadable) and [`CobValue`](crate::prelude::CobValue)
/// serializer and deserialize seed.
pub type CobResult<T> = Result<T, CobError>;

//-------------------------------------------------------------------------------------------------------------------

/// Categorizes the cause of a `CobError`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ErrorCategory
{
    /// The error was caused by a failure to read or write bytes on an I/O
    /// stream.
    Io,

    /// The error was caused by input that was not a syntactically valid COB value.
    Syntax,

    /// The error was caused by input data that was semantically incorrect.
    ///
    /// For example, COB containing a number that is semantically incorrect when the
    /// type being deserialized into holds a String.
    Data,
}

//-------------------------------------------------------------------------------------------------------------------

/// This type represents all possible errors that can occur when serializing or
/// deserializing Cob values.
#[derive(Debug)]
pub enum CobError
{
    /// Catchall for syntax error messages
    Message(String),

    /// Some I/O error occurred while serializing or deserializing.
    Io(std::io::Error),

    /// An loadable was not registered with bevy type reflection and failed to serialize.
    LoadableNotRegistered,
    /// Only structs and enums can be serialized as loadables.
    NotALoadable,
    /// A built-in type failed to serialize to [`CobBuiltin`](crate::prelude::CobBuiltin).
    MalformedBuiltin,
    /// Failed deserializing a newtype loadable or enum variant that is represented as a unit-like
    /// struct/variant.
    UnresolvedNewtypeStruct,
    /// An loadable identifier failed to parse.
    MalformedLoadableId,
}

impl CobError
{
    /// Categorizes the cause of this error.
    pub fn classify(&self) -> ErrorCategory
    {
        match *self {
            Self::Message(_) => ErrorCategory::Data,
            Self::Io(_) => ErrorCategory::Io,
            Self::LoadableNotRegistered
            | Self::NotALoadable
            | Self::MalformedBuiltin
            | Self::UnresolvedNewtypeStruct
            | Self::MalformedLoadableId => ErrorCategory::Syntax,
        }
    }

    /// Returns true if this error was caused by a failure to read or write
    /// bytes on an I/O stream.
    pub fn is_io(&self) -> bool
    {
        self.classify() == ErrorCategory::Io
    }

    /// Returns true if this error was caused by input that was not
    /// syntactically valid COB.
    pub fn is_syntax(&self) -> bool
    {
        self.classify() == ErrorCategory::Syntax
    }

    /// Returns true if this error was caused by input data that was
    /// semantically incorrect.
    ///
    /// For example, COB containing a number that is semantically incorrect when the
    /// type being deserialized into holds a String.
    pub fn is_data(&self) -> bool
    {
        self.classify() == ErrorCategory::Data
    }

    /// The kind reported by the underlying standard library I/O error, if this
    /// error was caused by a failure to read or write bytes on an I/O stream.
    pub fn io_error_kind(&self) -> Option<ErrorKind>
    {
        if let Self::Io(io_error) = self {
            Some(io_error.kind())
        } else {
            None
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<CobError> for std::io::Error
{
    /// Convert a `CobError` into a `std::io::Error`.
    ///
    /// Syntax and data errors are turned into `InvalidData` I/O errors.
    fn from(e: CobError) -> Self
    {
        if let CobError::Io(err) = e {
            err
        } else {
            match e.classify() {
                ErrorCategory::Io => unreachable!(),
                ErrorCategory::Syntax | ErrorCategory::Data => std::io::Error::new(ErrorKind::InvalidData, e),
            }
        }
    }
}

impl From<std::io::Error> for CobError
{
    fn from(e: std::io::Error) -> Self
    {
        Self::Io(e)
    }
}

impl From<String> for CobError
{
    fn from(e: String) -> Self
    {
        Self::Message(e)
    }
}

impl Display for CobError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            Self::Message(msg) => f.write_str(msg),
            Self::Io(err) => Display::fmt(err, f),
            Self::LoadableNotRegistered => {
                f.write_str("tried serializing a type that wasn't registered with bevy's type registry")
            }
            Self::NotALoadable => f.write_str("tried serializing a type as CobLoadable that isn't a struct/enum"),
            Self::MalformedBuiltin => f.write_str("tried serializing a builtin type that is malformed"),
            Self::UnresolvedNewtypeStruct => f.write_str(
                "failed deserializing a newtype struct or enum variant represented as a unit struct/variant",
            ),
            Self::MalformedLoadableId => {
                f.write_str("tried extracting an loadable id from a malformed short path")
            }
        }
    }
}

impl std::error::Error for CobError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self {
            Self::Io(err) => err.source(),
            _ => None,
        }
    }
}

impl serde::de::Error for CobError
{
    #[cold]
    fn custom<T: Display>(msg: T) -> CobError
    {
        CobError::Message(msg.to_string())
    }
}

impl serde::ser::Error for CobError
{
    #[cold]
    fn custom<T: Display>(msg: T) -> CobError
    {
        CobError::Message(msg.to_string())
    }
}

//-------------------------------------------------------------------------------------------------------------------

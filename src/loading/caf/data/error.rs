use std::fmt::Display;
use std::io::ErrorKind;

//-------------------------------------------------------------------------------------------------------------------

/// Result for the [`CafLoadable`] and [`CafValue`] serializer and deserialize seed.
pub type CafResult<T> = Result<T, CafError>;

//-------------------------------------------------------------------------------------------------------------------

/// Categorizes the cause of a `CafError`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ErrorCategory
{
    /// The error was caused by a failure to read or write bytes on an I/O
    /// stream.
    Io,

    /// The error was caused by input that was not a syntactically valid CAF value.
    Syntax,

    /// The error was caused by input data that was semantically incorrect.
    ///
    /// For example, CAF containing a number that is semantically incorrect when the
    /// type being deserialized into holds a String.
    Data,
}

//-------------------------------------------------------------------------------------------------------------------

/// This type represents all possible errors that can occur when serializing or
/// deserializing Caf values.
#[derive(Debug)]
pub enum CafError
{
    /// Catchall for syntax error messages
    Message(String),

    /// Some I/O error occurred while serializing or deserializing.
    Io(std::io::Error),

    /// An loadable was not registered with bevy type reflection and failed to serialize.
    LoadableNotRegistered,
    /// Only structs and enums can be serialized as loadables.
    NotALoadable,
    /// A built-in type failed to serialize to [`CafBuiltin`].
    MalformedBuiltin,
    /// Failed deserializing a newtype loadable or enum variant that is represented as a unit-like
    /// struct/variant.
    UnresolvedNewtypeStruct,
    /// An loadable identifier failed to parse.
    MalformedLoadableId,
}

impl CafError
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
    /// syntactically valid CAF.
    pub fn is_syntax(&self) -> bool
    {
        self.classify() == ErrorCategory::Syntax
    }

    /// Returns true if this error was caused by input data that was
    /// semantically incorrect.
    ///
    /// For example, CAF containing a number that is semantically incorrect when the
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
impl From<CafError> for std::io::Error
{
    /// Convert a `CafError` into a `std::io::Error`.
    ///
    /// Syntax and data errors are turned into `InvalidData` I/O errors.
    fn from(e: CafError) -> Self
    {
        if let CafError::Io(err) = e {
            err
        } else {
            match e.classify() {
                ErrorCategory::Io => unreachable!(),
                ErrorCategory::Syntax | ErrorCategory::Data => std::io::Error::new(ErrorKind::InvalidData, e),
            }
        }
    }
}

impl From<std::io::Error> for CafError
{
    fn from(e: std::io::Error) -> Self
    {
        Self::Io(e)
    }
}

impl From<String> for CafError
{
    fn from(e: String) -> Self
    {
        Self::Message(e)
    }
}

impl Display for CafError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            Self::Message(msg) => f.write_str(msg),
            Self::Io(err) => Display::fmt(err, f),
            Self::LoadableNotRegistered => {
                f.write_str("tried serializing a type that wasn't registered with bevy's type registry")
            }
            Self::NotALoadable => f.write_str("tried serializing a type as CafLoadable that isn't a struct/enum"),
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

impl std::error::Error for CafError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        match self {
            Self::Io(err) => err.source(),
            _ => None,
        }
    }
}

impl serde::de::Error for CafError
{
    #[cold]
    fn custom<T: Display>(msg: T) -> CafError
    {
        CafError::Message(msg.to_string())
    }
}

impl serde::ser::Error for CafError
{
    #[cold]
    fn custom<T: Display>(msg: T) -> CafError
    {
        CafError::Message(msg.to_string())
    }
}

//-------------------------------------------------------------------------------------------------------------------

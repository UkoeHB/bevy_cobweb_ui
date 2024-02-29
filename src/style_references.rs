//local shortcuts

//third-party shortcuts

//standard shortcuts
use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// The token that separates parts of a style path.
pub const STYLE_PATH_SEPARATOR: &'static str = "::";

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a stylesheet file in the `asset` directory.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct StyleFile
{
    file: Arc<str>,
}

impl StyleFile
{
    /// Creates a new style file reference from a file name.
    ///
    /// The file name may contain trailing file extensions (i.e. `.style.json`), but should not contain trailing
    /// syntax (i.e. `examples/sample` and `examples/sample.style.json` are valid, but `examples/sample/` is not).
    pub fn new(file: &str) -> Self
    {
        // Strip file extensions.
        let file = Arc::from(file.split('.').next().unwrap());

        Self{ file }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a specific style in a stylesheet file.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct StylePath
{
    path: Arc<[Arc<str>]>,
}

impl StylePath
{
    /// Creates a new style path.
    pub fn new(new_path: &str) -> Self
    {
        let mut path = Vec::default();
        Self::extend_inner(new_path, &mut path);

        Self{ path: Arc::from(path) }
    }

    /// Extends an existing style path with a path extension.
    pub fn extend(&self, extension: &str) -> Self
    {
        let mut path = Vec::from(&*self.path);
        Self::extend_inner(extension, &mut path);

        Self{ path: Arc::from(path) }
    }

    fn extend_inner(extension: &str, path: &mut Vec<Arc<str>>)
    {
        for path_element in extension.split(STYLE_PATH_SEPARATOR)
        {
            if path_element.is_empty() { continue; }
            path.push(Arc::from(String::from(path_element)));
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents a complete reference to a style instance in a stylesheet asset.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StyleRef
{
    pub file: StyleFile,
    pub path: StylePath,
}

impl StyleRef
{
    /// Creates a new style reference from a file name.
    pub fn from_file(file: &str) -> Self
    {
        Self::new(file, "")
    }

    /// Creates a new style reference from a file name and path.
    pub fn new(file: &str, path: &str) -> Self
    {
        Self{ file: StyleFile::new(file), path: StylePath::new(path) }
    }

    /// Extends an existing style reference with a path extension.
    pub fn extend(&self, extension: &str) -> Self
    {
        Self{
            file: self.file.clone(),
            path: self.path.extend(extension)
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores a complete style path, including the style's [`type_path`](bevy::reflect::TypePath::type_path).
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FullStylePath
{
    pub path: StylePath,
    pub full_type_name: &'static str,
}

impl FullStylePath
{
    /// Finalizes a [`StylePath`] by specifying the style's [`type_path`](bevy::reflect::TypePath::type_path),
    /// which is used to identify the style in stylesheet files.
    pub fn new(path: StylePath, full_type_name: &'static str) -> Self
    {
        Self{ path, full_type_name }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores a fully-specified reference to a style.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FullStyleRef
{
    pub file: StyleFile,
    pub path: FullStylePath,
}

impl FullStyleRef
{
    /// Creates a full style reference.
    pub fn new(file: StyleFile, path: FullStylePath) -> Self
    {
        Self{ file, path }
    }
}

//-------------------------------------------------------------------------------------------------------------------

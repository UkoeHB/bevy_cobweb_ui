use smol_str::SmolStr;

use std::sync::Arc;

//-------------------------------------------------------------------------------------------------------------------

/// The token that separates parts of a style path.
///
/// Example: `menu::header::title`, where `menu`, `header`, and `title` are path extensions.
pub const STYLE_PATH_SEPARATOR: &'static str = "::";

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a stylesheet file in the `asset` directory.
///
/// Stylesheet files use the `.style.json` extension.
///
/// Example: `ui/home.style.json` for a `home` stylesheet in `assets/ui`.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct StyleFile
{
    pub(crate) file: Arc<str>,
}

impl StyleFile
{
    /// Creates a new style file reference from a file name.
    ///
    /// The file name should include the file extension (i.e. `.style.json`).
    pub fn new(file: &str) -> Self
    {
        Self{ file: Arc::from(file) }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a specific style in a stylesheet file.
/// 
/// Path extensions are stored as [`SmolStr`], so it is recommended for extensions to be <= 25 characters long.
///
/// Example: `menu::header::title` for accessing the `title` style path in a stylesheet.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct StylePath
{
    pub(crate) path: Arc<[SmolStr]>,
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

    fn extend_inner(extension: &str, path: &mut Vec<SmolStr>)
    {
        for path_element in extension.split(STYLE_PATH_SEPARATOR)
        {
            if path_element.is_empty() { continue; }
            path.push(SmolStr::from(path_element));
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents a complete reference to a style instance in a stylesheet asset.
///
/// Example:
/// - File: `ui/home.style.json` for a `home` stylesheet in `assets/ui`.
/// - Path: `menu::header::title` for accessing the `title` style path in the `home` stylesheet.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct StyleRef
{
    /// See [`StyleFile`].
    pub file: StyleFile,
    /// See [`StylePath`].
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

    /// Shorthand method for [`Self::extend`].
    pub fn e(&self, extension: &str) -> Self
    {
        self.extend(extension)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores a complete [`StylePath`] in addition to the style's [`type_path`](bevy::reflect::TypePath::type_path).
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FullStylePath
{
    /// See [`StylePath`].
    pub path: StylePath,
    /// See [`type_path`](bevy::reflect::TypePath::type_path).
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
    /// See [`StyleFile`].
    pub file: StyleFile,
    /// See [`FullStylePath`].
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

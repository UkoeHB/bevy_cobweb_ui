use std::sync::Arc;

use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

/// The token that separates parts of a loadable path.
///
/// Example: `menu::header::title`, where `menu`, `header`, and `title` are path extensions.
pub const LOADABLE_PATH_SEPARATOR: &'static str = "::";

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a loadable-sheet file in the `asset` directory.
///
/// Loadable-sheet files use the `.loadable.json` extension.
///
/// Example: `ui/home.loadable.json` for a `home` loadable-sheet in `assets/ui`.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LoadableFile
{
    pub(crate) file: Arc<str>,
}

impl LoadableFile
{
    /// Creates a new loadable file reference from a file name.
    ///
    /// The file name should include the file extension (i.e. `.loadable.json`).
    pub fn new(file: &str) -> Self
    {
        Self { file: Arc::from(file) }
    }
}

impl Default for LoadableFile
{
    fn default() -> Self
    {
        Self::new("")
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a specific loadable in a loadable-sheet file.
///
/// Path extensions are stored as [`SmolStr`], so it is recommended for extensions to be <= 25 characters long.
///
/// Example: `menu::header::title` for accessing the `title` loadable path in a loadable-sheet.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LoadablePath
{
    pub(crate) path: Arc<[SmolStr]>,
}

impl LoadablePath
{
    /// Creates a new loadable path.
    pub fn new(new_path: &str) -> Self
    {
        let mut path = Vec::default();
        Self::extend_inner(new_path, &mut path);

        Self { path: Arc::from(path) }
    }

    /// Extends an existing loadable path with a path extension.
    pub fn extend(&self, extension: &str) -> Self
    {
        let mut path = Vec::from(&*self.path);
        Self::extend_inner(extension, &mut path);

        Self { path: Arc::from(path) }
    }

    fn extend_inner(extension: &str, path: &mut Vec<SmolStr>)
    {
        for path_element in extension.split(LOADABLE_PATH_SEPARATOR) {
            if path_element.is_empty() {
                continue;
            }
            path.push(SmolStr::from(path_element));
        }
    }
}

impl Default for LoadablePath
{
    fn default() -> Self
    {
        Self::new("")
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents a complete reference to a loadable instance in a loadable-sheet asset.
///
/// Example:
/// - File: `ui/home.loadable.json` for a `home` loadable-sheet in `assets/ui`.
/// - Path: `menu::header::title` for accessing the `title` loadable path in the `home` loadable-sheet.
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct LoadableRef
{
    /// See [`LoadableFile`].
    pub file: LoadableFile,
    /// See [`LoadablePath`].
    pub path: LoadablePath,
}

impl LoadableRef
{
    /// Creates a new loadable reference from a file name.
    pub fn from_file(file: &str) -> Self
    {
        Self::new(file, "")
    }

    /// Creates a new loadable reference from a file name and path.
    pub fn new(file: &str, path: &str) -> Self
    {
        Self { file: LoadableFile::new(file), path: LoadablePath::new(path) }
    }

    /// Extends an existing loadable reference with a path extension.
    pub fn extend(&self, extension: &str) -> Self
    {
        Self { file: self.file.clone(), path: self.path.extend(extension) }
    }

    /// Shorthand method for [`Self::extend`].
    pub fn e(&self, extension: &str) -> Self
    {
        self.extend(extension)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores a complete [`LoadablePath`] in addition to the loadable's
/// [`type_path`](bevy::reflect::TypePath::type_path).
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FullLoadablePath
{
    /// See [`LoadablePath`].
    pub path: LoadablePath,
    /// See [`type_path`](bevy::reflect::TypePath::type_path).
    pub full_type_name: &'static str,
}

impl FullLoadablePath
{
    /// Finalizes a [`LoadablePath`] by specifying the loadable's
    /// [`type_path`](bevy::reflect::TypePath::type_path), which is used to identify the loadable in
    /// loadable-sheet files.
    pub fn new(path: LoadablePath, full_type_name: &'static str) -> Self
    {
        Self { path, full_type_name }
    }
}

impl Default for FullLoadablePath
{
    fn default() -> Self
    {
        Self::new(LoadablePath::default(), "")
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores a fully-specified reference to a loadable.
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct FullLoadableRef
{
    /// See [`LoadableFile`].
    pub file: LoadableFile,
    /// See [`FullLoadablePath`].
    pub path: FullLoadablePath,
}

impl FullLoadableRef
{
    /// Creates a full loadable reference.
    pub fn new(file: LoadableFile, path: FullLoadablePath) -> Self
    {
        Self { file, path }
    }
}

//-------------------------------------------------------------------------------------------------------------------

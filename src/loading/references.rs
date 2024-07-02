use std::sync::Arc;

use smallvec::SmallVec;
use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

/// The token that separates parts of a loadable path.
///
/// Example: `menu::header::title`, where `menu`, `header`, and `title` are path extensions.
pub const LOADABLE_PATH_SEPARATOR: &str = "::";

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a cobweb asset file in the `asset` directory, or a manifest key for that file.
///
/// Cobweb asset files use the `.load.json` extension.
///
/// Example: `ui/home.load.json` for a `home` cobweb asset in `assets/ui`.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum LoadableFile
{
    File(Arc<str>),
    ManifestKey(Arc<str>),
}

impl LoadableFile
{
    /// Creates a new loadable file reference from a file name.
    ///
    /// If the file name does not include a file extension (i.e. `.load.json`), then it will be treated as a
    /// manifest key.
    pub fn new(file: impl AsRef<str>) -> Self
    {
        let file = file.as_ref();
        match file.ends_with(".load.json") {
            true => Self::File(Arc::from(file)),
            false => Self::ManifestKey(Arc::from(file)),
        }
    }

    /// Gets the internal string representation.
    pub fn as_str(&self) -> &str
    {
        match self {
            Self::File(file) => file,
            Self::ManifestKey(key) => key,
        }
    }

    /// Gets the internal `Arc<str>` representation.
    pub fn inner(&self) -> &Arc<str>
    {
        match self {
            Self::File(file) => file,
            Self::ManifestKey(key) => key,
        }
    }

    /// Returns `true` if this file reference is a file path.
    pub fn is_file_path(&self) -> bool
    {
        matches!(*self, Self::File(_))
    }

    /// Returns `true` if this file reference is a manifest key pointing to the actual file path.
    pub fn is_manifest_key(&self) -> bool
    {
        matches!(*self, Self::ManifestKey(_))
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

/// Represents the path to a specific scene node in a cobweb asset file.
///
/// Path extensions are stored as [`SmolStr`], so it is recommended for extensions to be <= 25 characters long.
///
/// Example: `menu::header::title` for accessing the `title` scene node in the `menu` scene in a cobweb asset file.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LoadablePath
{
    pub(crate) path: Arc<[SmolStr]>,
}

impl LoadablePath
{
    /// Creates a new loadable path.
    pub fn new(new_path: impl AsRef<str>) -> Self
    {
        let new_path = new_path.as_ref();
        let mut path = SmallVec::<[SmolStr; 10]>::default();
        Self::extend_inner(new_path, &mut path);

        Self { path: Arc::from(path.as_slice()) }
    }

    /// Parses a path with one segment from the given string.
    ///
    /// Returns `None` if `new_path` is not exactly one non-empty segment.
    pub fn parse_single(new_path: impl AsRef<str>) -> Option<Self>
    {
        let new_path = new_path.as_ref();
        let segment = Self::parse_single_inner(new_path)?;

        Some(Self { path: Arc::from([SmolStr::from(segment)]) })
    }

    /// Extends an existing loadable path with a path extension.
    pub fn extend(&self, extension: impl AsRef<str>) -> Self
    {
        let extension = extension.as_ref();
        let mut path = SmallVec::<[SmolStr; 10]>::from(&*self.path);
        Self::extend_inner(extension, &mut path);

        Self { path: Arc::from(path.as_slice()) }
    }

    /// Extends an existing loadable path with a path extension.
    ///
    /// Returns `None` if `extension` is not exactly one non-empty segment.
    pub fn extend_single(&self, extension: impl AsRef<str>) -> Option<Self>
    {
        let extension = extension.as_ref();
        let mut path = SmallVec::<[SmolStr; 10]>::from(&*self.path);
        let segment = Self::parse_single_inner(extension)?;
        path.push(SmolStr::from(segment));

        Some(Self { path: Arc::from(path.as_slice()) })
    }

    /// Gets the number of path segments.
    pub fn len(&self) -> usize
    {
        self.path.len()
    }

    fn extend_inner(extension: &str, path: &mut SmallVec<[SmolStr; 10]>)
    {
        for path_element in extension.split(LOADABLE_PATH_SEPARATOR) {
            if path_element.is_empty() {
                continue;
            }
            path.push(SmolStr::from(path_element));
        }
    }

    fn parse_single_inner(extension: &str) -> Option<&str>
    {
        let mut segments = extension.split(LOADABLE_PATH_SEPARATOR);
        let first_segment = segments.next()?;
        if first_segment.is_empty() {
            return None;
        }
        let None = segments.next() else {
            return None;
        };

        Some(first_segment)
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

/// Represents a complete reference to a scene node in a cobweb asset asset.
///
/// Example:
/// - **File**: `ui/home.load.json` for a `home` cobweb asset in `assets/ui`.
/// - **Path**: `menu::header::title` for accessing the `title` scene node in the `menu` scene in the `home` cobweb
/// asset.
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
    pub fn from_file(file: impl AsRef<str>) -> Self
    {
        Self::new(file.as_ref(), "")
    }

    /// Creates a new loadable reference from a file name and path.
    pub fn new(file: impl AsRef<str>, path: impl AsRef<str>) -> Self
    {
        Self {
            file: LoadableFile::new(file.as_ref()),
            path: LoadablePath::new(path),
        }
    }

    /// Extends an existing loadable reference with a path extension.
    pub fn extend(&self, extension: impl AsRef<str>) -> Self
    {
        Self {
            file: self.file.clone(),
            path: self.path.extend(extension.as_ref()),
        }
    }

    /// Shorthand for [`Self::extend`].
    pub fn e(&self, extension: impl AsRef<str>) -> Self
    {
        self.extend(extension)
    }
}

//-------------------------------------------------------------------------------------------------------------------

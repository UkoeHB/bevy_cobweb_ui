use std::ops::Add;
use std::sync::Arc;

use smallvec::SmallVec;
use smol_str::SmolStr;

pub use crate::prelude::cob::{CobFile, ManifestKey};

//-------------------------------------------------------------------------------------------------------------------

/// The token that separates parts of a scene path.
///
/// Example: `menu::header::title`, where `menu`, `header`, and `title` are path extensions.
pub const SCENE_PATH_SEPARATOR: &str = "::";

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a cobweb asset file in the `asset` directory, or a manifest key for that file.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum SceneFile
{
    File(CobFile),
    ManifestKey(ManifestKey),
}

impl SceneFile
{
    /// Creates a new COB file reference from a file name.
    ///
    /// If the file name does not include the `.cob` or `.cobweb` file extensions, then it will be treated as a
    /// manifest key.
    pub fn new(file: impl AsRef<str>) -> Self
    {
        let file = file.as_ref();
        if let Some(file) = CobFile::try_new(file) {
            Self::File(file)
        } else {
            Self::ManifestKey(ManifestKey::new(file))
        }
    }

    /// Gets the internal string representation.
    pub fn as_str(&self) -> &str
    {
        match self {
            Self::File(file) => file.as_str(),
            Self::ManifestKey(key) => key.as_str(),
        }
    }

    /// Gets the internal `Arc<str>` representation.
    pub fn inner(&self) -> &Arc<str>
    {
        match self {
            Self::File(file) => &*file,
            Self::ManifestKey(key) => &*key,
        }
    }

    /// Returns `true` if this file reference is a file path (i.e. it ends with `.cob`).
    pub fn is_file_path(&self) -> bool
    {
        matches!(*self, Self::File(_))
    }

    /// Returns `true` if this file reference is a manifest key pointing to the actual file path.
    pub fn is_manifest_key(&self) -> bool
    {
        matches!(*self, Self::ManifestKey(_))
    }

    /// Gets the internal file if there is one.
    pub fn file(&self) -> Option<&CobFile>
    {
        match self {
            Self::File(file) => Some(file),
            Self::ManifestKey(_) => None,
        }
    }

    /// Gets the internal manifest key if there is one.
    pub fn manifest_key(&self) -> Option<&ManifestKey>
    {
        match self {
            Self::File(_) => None,
            Self::ManifestKey(key) => Some(key),
        }
    }

    /// Returns `true` if the string ends in `.cob` or `.cobweb`.
    pub fn str_is_file_path(string: impl AsRef<str>) -> bool
    {
        let str = string.as_ref();
        str.ends_with(".cob") || str.ends_with(".cobweb")
    }

    /// Extends an existing scene file with a path extension.
    pub fn extend(&self, extension: impl AsRef<str>) -> SceneRef
    {
        SceneRef { file: self.clone(), path: ScenePath::new(extension) }
    }

    /// Shorthand for [`Self::extend`].
    pub fn e(&self, extension: impl AsRef<str>) -> SceneRef
    {
        self.extend(extension)
    }
}

impl<T: AsRef<str>> Add<T> for SceneFile
{
    type Output = SceneRef;
    fn add(self, rhs: T) -> Self::Output
    {
        self.extend(rhs)
    }
}

impl<T: AsRef<str>> Add<T> for &SceneFile
{
    type Output = SceneRef;
    fn add(self, rhs: T) -> Self::Output
    {
        self.extend(rhs)
    }
}

impl From<CobFile> for SceneFile
{
    fn from(file: CobFile) -> Self
    {
        Self::File(file)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents the path to a specific scene node in a cobweb asset file.
///
/// Path extensions are stored as [`SmolStr`], so it is recommended for extensions to be <= 25 characters long.
///
/// Example: `menu::header::title` for accessing the `title` scene node in the `menu` scene in a cobweb asset file.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ScenePath
{
    pub(crate) path: Arc<[SmolStr]>,
}

impl ScenePath
{
    /// Creates a new scene path.
    pub fn new(new_path: impl AsRef<str>) -> Self
    {
        let new_path = new_path.as_ref();
        let mut path = SmallVec::<[SmolStr; 10]>::default();
        Self::extend_inner(new_path, &mut path);

        Self { path: Arc::from(path.as_slice()) }
    }

    /// Creates an empty scene path.
    pub fn empty() -> Self
    {
        Self { path: Arc::from([]) }
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

    /// Extends an existing scene path with a path extension.
    pub fn extend(&self, extension: impl AsRef<str>) -> Self
    {
        let extension = extension.as_ref();
        let mut path = SmallVec::<[SmolStr; 10]>::from(&*self.path);
        Self::extend_inner(extension, &mut path);

        Self { path: Arc::from(path.as_slice()) }
    }

    /// Extends an existing scene path with a path extension.
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

    /// Extends a path starting after the requested index.
    pub fn extend_from_index(&self, index: usize, extension: impl AsRef<str>) -> Self
    {
        let extension = extension.as_ref();
        let mut path = SmallVec::<[SmolStr; 10]>::from(&self.path[0..=index]);
        Self::extend_inner(extension, &mut path);

        Self { path: Arc::from(path.as_slice()) }
    }

    /// Gets the number of path segments.
    pub fn len(&self) -> usize
    {
        self.path.len()
    }

    /// Iterates the path's segments.
    pub fn iter(&self) -> impl Iterator<Item = &str> + Clone + DoubleEndedIterator
    {
        self.path.iter().map(|s| s.as_str())
    }

    fn extend_inner(extension: &str, path: &mut SmallVec<[SmolStr; 10]>)
    {
        for path_element in extension.split(SCENE_PATH_SEPARATOR) {
            if path_element.is_empty() {
                continue;
            }
            path.push(SmolStr::from(path_element));
        }
    }

    fn parse_single_inner(extension: &str) -> Option<&str>
    {
        let mut segments = extension.split(SCENE_PATH_SEPARATOR);
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

//-------------------------------------------------------------------------------------------------------------------

/// Represents a complete reference to a scene node in a cobweb asset asset.
///
/// Example:
/// - **File**: `ui/home.cob` for a `home` cobweb asset in `assets/ui`.
/// - **Path**: `menu::header::title` for accessing the `title` scene node in the `menu` scene in the `home` cobweb
/// asset.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SceneRef
{
    /// See [`SceneFile`].
    pub file: SceneFile,
    /// See [`ScenePath`].
    pub path: ScenePath,
}

impl SceneRef
{
    /// Creates a new scene reference from a file name.
    pub fn from_file(file: impl AsRef<str>) -> Self
    {
        Self::new(file.as_ref(), "")
    }

    /// Creates a new scene reference from a file name and path.
    pub fn new(file: impl AsRef<str>, path: impl AsRef<str>) -> Self
    {
        Self {
            file: SceneFile::new(file.as_ref()),
            path: ScenePath::new(path),
        }
    }

    /// Extends an existing scene reference with a path extension.
    pub fn extend(&self, extension: impl AsRef<str>) -> Self
    {
        Self {
            file: self.file.clone(),
            path: self.path.extend(extension.as_ref()),
        }
    }

    /// Extends a path starting after the requested index.
    pub fn extend_from_index(&self, index: usize, extension: impl AsRef<str>) -> Self
    {
        Self {
            file: self.file.clone(),
            path: self.path.extend_from_index(index, extension.as_ref()),
        }
    }

    /// Shorthand for [`Self::extend`].
    pub fn e(&self, extension: impl AsRef<str>) -> Self
    {
        self.extend(extension)
    }
}

impl<T: AsRef<str>> Add<T> for SceneRef
{
    type Output = SceneRef;
    fn add(self, rhs: T) -> Self::Output
    {
        self.extend(rhs)
    }
}

impl<T: AsRef<str>> Add<T> for &SceneRef
{
    type Output = SceneRef;
    fn add(self, rhs: T) -> Self::Output
    {
        self.extend(rhs)
    }
}

impl<A: AsRef<str>, B: AsRef<str>> From<(A, B)> for SceneRef
{
    fn from((a, b): (A, B)) -> Self
    {
        SceneRef::new(a.as_ref(), b.as_ref())
    }
}

//-------------------------------------------------------------------------------------------------------------------

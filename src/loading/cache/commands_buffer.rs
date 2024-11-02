
//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
struct PendingCommandsCounter
{
    pending: usize,
}

impl PendingCommandsCounter
{
    fn add(&mut self, num: usize)
    {
        pending += num;
    }

    fn remove(&mut self, num: usize)
    {
        debug_assert!(num <= pending);
        pending.saturating_sub(num);
    }

    fn get(&self) -> usize
    {
        self.pending
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
enum CachedCommand
{
    /// Command that was inserted/updated but has not been applied yet.
    Pending(ReflectedLoadable),
    /// Command that is saved in case it gets refreshed.
    #[cfg(feature = "hot_reload")]
    Done(ReflectedLoadable),
    #[cfg(not(feature = "hot_reload"))]
    Done,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
enum RegisteredFile
{
    Pending(CafFile),
    Loaded(CafFile)
}

impl RegisteredFile
{
    fn file(&self) -> &CafFile
    {
        match self {
            Self::Pending(file) |
            Self::Loaded(file) => file
        }
    }

    fn set_loaded(&mut self)
    {
        *self = Self::Loaded(self.file().clone())
    }

    fn set_pending(&mut self)
    {
        *self = Self::Pending(self.file().clone())
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct FileCommandsInfo
{
    /// This file (if pending, then commands and descendants are unknown).
    file: RegisteredFile,
    /// This file's parent. Used to traverse back up the tree to reach siblings/cousins.
    ///
    /// If `None` then the file was orphaned due to a hot-reload.
    parent: Option<CafFile>,
    /// Cached commands from this file.
    commands: Vec<CachedCommand>,
    /// Files in this file's manifest. Commands in these files will be applied immediately after the commands in
    /// this file.
    ///
    /// Used to traverse down the tree to descendants.
    descendants: Vec<CafFile>,

    /// Index into flattened hierarchy.
    ///
    /// Used to determine the 'traversal start point' when files are hot-reloaded.
    #[cfg(feature = "hot_reload")]
    idx: usize,

    /// Indicates if the current file is orphaned or part of an orphaned branch.
    ///
    /// Orphaned files do not participate in the 'pending commands' counter.
    #[cfg(feature = "hot_reload")]
    is_orphaned: bool,
}

//-------------------------------------------------------------------------------------------------------------------

const GLOBAL_PSEUDO_FILE: &'static str = "__g.caf";

//-------------------------------------------------------------------------------------------------------------------

/// Manages commands loaded from CAF files to ensure they are applied in global order.
#[derive(Default, Debug)]
pub(super) struct CommandsBuffer
{
    /// 'Earliest' file in the hierarchy with un-applied commands.
    ///
    /// If `None` then no file is targeted for pending commands.
    pending: Option<CafFile>,

    /// Flattened file hierarchy.
    hierarchy: HashMap<CafFile, FileCommandsInfo>,

    /// Cached file list.
    ///
    /// Used to disambiguate orphaned files from loaded files when their internally-tracked indices might
    /// overlap.
    #[cfg(feature = "hot_reload")]
    file_order: Vec<CafFile>,

    /// Number of unapplied commands in non-orphaned files. Used to short-circuit traversal when refreshing
    /// commands.
    counter: PendingCommandsCounter,
}

impl CommandsBuffer
{
    // new: add 'global' file where manually-loaded files will be added as descendants

    // prepare file (parent, self)
    // - add entry with self as pending file
    // - debug assert that file name doesn't equal global file

    // set file descendants (self, descendants)
    // - this is separate from commands to ensure we create entries for all manifest loads immediately
    //   after extracting a manifest section; this way when manifest loads show up, they will have a pre-existing
    //   slot to land in
    //   - race condition: file A manifest loads file B, file B is loading, file A reloads but removes file B before
    //   file B has loaded, file B loads
    //     - solved by tracking orphaned branches
    // - if file already exists
    //   - might need to change the parent (warn if this changes)
    //   - if children are removed, then traverse them to set `is_orphaned` and update counter
    //     - set orphaned child's `parent` to `None`
    //     - include recursion limit in case of manifest loop (can also terminate and warn when encounter same
    //     `is_orphaned` value)
    //   - if child is added, set `parent` to self and propagate `is_orphaned` flag (if existing child's `is_orphaned`
    //   doesn't match self) and update counter
    //     - include recursion limit in case of manifest loop (can also terminate and warn when encounter same
    //     `is_orphaned` value if direct child's flag differs from a deeper child)
    //   - if commands or children are changed, then if self is within `file_order`, truncate `file_order` to self
    //   and update the traversal point to <= self
    //     - NOTE: this should ensure the traversal point always points to a file in the 'true hierarchy' and all
    //     pending commands are >= that file in the hierarchy
    //     - NOTE: the file may or may not be orphaned; only non-orphaned files will be in `file_order`

    // set file commands
    // - if command value changes, then set it to pending
    // - if non-orphaned
    //   - adjust unapplied commands count based on difference (+ or -) in pending commands here
    // - make sure new commands order is preserved

    // iterate forward, applying commands from loaded files; stop when number of applied commands reaches zero,
    // or when encounter a pending file
    // - if file's parent doesn't match the known parent, then skip it and warn (can occur due to duplicates)
    // - push back to `file_order` as files are traversed
    // - update the `idx` in each file's info
    // - set recursion limit in case of manifest loop (a cap of 100 is probably more than adequate for a file hierarchy)
    // - if not `hot_reload`, then when traversal completes the last file, clean up internal resources (or do this in
    // the CobwebAssetCache?)
}

//-------------------------------------------------------------------------------------------------------------------

// - after manifest extraction, cache descendants until file is processed, so the descendants and commands can
// be added simultaneously
//   - it's possible that a manifest load can be added to the buffer before its parent, if its parent gets stuck
//     waiting for import dependencies

// plugin
// - after caf extraction, iterate buffer to try an apply as many commands as possible

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use wasm_timer::{SystemTime, UNIX_EPOCH};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn get_current_time() -> Duration
{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

//-------------------------------------------------------------------------------------------------------------------

struct CommandLoadCommand
{
    callback: fn(&mut World, ReflectedLoadable, SceneRef),
    scene_ref: SceneRef,
    loadable: ReflectedLoadable,
}

impl Command for CommandLoadCommand
{
    fn apply(self, world: &mut World)
    {
        (self.callback)(world, self.loadable, self.scene_ref);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
struct PendingCounter
{
    pending: usize,
}

impl PendingCounter
{
    fn add(&mut self, num: usize)
    {
        self.pending += num;
    }

    fn remove(&mut self, num: usize)
    {
        debug_assert!(num <= self.pending);
        self.pending = self.pending.saturating_sub(num);
    }

    fn get(&self) -> usize
    {
        self.pending
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct CachedCommand
{
    command: ErasedLoadable,
    is_pending: bool,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum FileStatus
{
    Pending,
    AwaitingCommands,
    Loaded,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum FileParent
{
    SelfIsRoot,
    #[cfg(feature = "hot_reload")]
    SelfIsOrphan,
    Parent(CobFile),
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct FileCommandsInfo
{
    /// This file's status.
    ///
    /// If pending, then commands and descendants are unknown.
    status: FileStatus,
    /// This file's parent. Used to traverse back up the tree to reach siblings/cousins.
    parent: FileParent,
    /// Cached commands from this file.
    commands: Vec<CachedCommand>,
    /// Files in this file's manifest. Commands in these files will be applied immediately after the commands in
    /// this file.
    ///
    /// Used to traverse down the tree to descendants.
    ///
    /// We use an `Arc` so we can use a stack-based approach to traverse the hierarchy, avoiding lifetime issues.
    descendants: Arc<[CobFile]>,

    /// Index into flattened hierarchy. If `CommandsBuffer::file_order` vec position doesn't
    /// match this file at `idx`, then the index is stale.
    ///
    /// Used to determine the 'traversal start point' when files are hot-reloaded.
    #[cfg(feature = "hot_reload")]
    idx: usize,

    /// Indicates if the current file is orphaned or part of an orphaned branch.
    ///
    /// Orphaned files do not participate in the 'pending commands' counter.
    is_orphaned: bool,

    /// Indicates if this info has been initialized. Used to detect if a file is being refreshed.
    initialized: bool,
}

//-------------------------------------------------------------------------------------------------------------------

const GLOBAL_PSEUDO_FILE: &'static str = "__g.cob";

//-------------------------------------------------------------------------------------------------------------------

/// Manages commands loaded from COB files to ensure they are applied in global order.
///
/// When the `hot_reload` feature is not enabled, this resource will be removed in schedule
/// `OnExit(LoadState::Loading)`.
#[derive(Resource, Debug)]
pub(crate) struct CommandsBuffer
{
    /// 'Earliest' file in the hierarchy with un-applied commands.
    ///
    /// If `None` then no file is targeted for pending commands.
    traversal_point: Option<CobFile>,

    /// Flattened file hierarchy.
    hierarchy: HashMap<CobFile, FileCommandsInfo>,

    /// Cached file list.
    ///
    /// Used to disambiguate orphaned files from loaded files when their internally-tracked indices might
    /// overlap.
    #[cfg(feature = "hot_reload")]
    file_order: Vec<CobFile>,

    /// Number of unapplied commands in non-orphaned files. Used to short-circuit traversal when refreshing
    /// commands.
    command_counter: PendingCounter,

    /// Number of pending non-orphaned files. Used to synchronize hierarchy state in a hot-reloading environment.
    file_counter: PendingCounter,

    /// Flag indicating whether any file has been refreshed after it was initialized. If true, then commands
    /// will not be applied if any files are pending. Only relevant in a hot-reloading environment where we need
    /// to avoid race conditions involving multiple files being refreshed concurrently.
    any_files_refreshed: bool,

    /// Time lock on applying commands after an orphaning event.
    ///
    /// Used to mitigate race conditions around hot-reloaded reparenting.
    commands_unlock_time: Duration,

    /// Cached for reuse.
    dummy_commands_path: ScenePath,
    /// Cached for reuse.
    empty_descendants: Arc<[CobFile]>,
    /// Cached for reuse.
    root_descendents: Arc<[CobFile]>,

    /// Cached for memory reuse.
    #[cfg(feature = "hot_reload")]
    seen_cached: Vec<bool>,
    /// Cached for memory reuse.
    #[cfg(feature = "hot_reload")]
    stack_cached: Vec<(usize, Arc<[CobFile]>)>,
    /// Cached for memory reuse.
    inverted_stack_cached: Vec<(usize, Arc<[CobFile]>, bool)>,
}

impl CommandsBuffer
{
    /// Makes a new buffer with pseudo 'global' file at the root level,
    /// where manually-loaded files will be added as descendants.
    pub(super) fn new() -> Self
    {
        let global = Self::global_file();
        let mut buffer = Self {
            traversal_point: Some(global.clone()), // Start traversal at global file.
            hierarchy: HashMap::default(),
            #[cfg(feature = "hot_reload")]
            file_order: vec![], // file_order is empty, indicating a 'fresh traversal'
            command_counter: PendingCounter::default(),
            file_counter: PendingCounter::default(),
            any_files_refreshed: false,
            commands_unlock_time: Duration::default(),
            dummy_commands_path: ScenePath::new("#commands"),
            empty_descendants: Arc::from([]),
            root_descendents: Arc::from([global.clone()]),
            #[cfg(feature = "hot_reload")]
            seen_cached: vec![],
            #[cfg(feature = "hot_reload")]
            stack_cached: vec![],
            inverted_stack_cached: vec![],
        };
        buffer.hierarchy.insert(
            global,
            FileCommandsInfo {
                status: FileStatus::Pending,
                parent: FileParent::SelfIsRoot,
                commands: vec![],
                descendants: buffer.empty_descendants.clone(),
                #[cfg(feature = "hot_reload")]
                idx: usize::MAX,
                is_orphaned: false,
                initialized: false,
            },
        );
        buffer.file_counter.add(1);

        buffer
    }

    fn global_file() -> CobFile
    {
        CobFile::try_new(GLOBAL_PSEUDO_FILE).unwrap()
    }

    /// Returns `true` if the buffer has been refreshed and there are any pending files or commands, or if the
    /// orphan timer is running.
    ///
    /// Used in the `SceneBuffer` to prevent scene reloads when there might be unapplied commands.
    ///
    /// Note that we filter on `any_files_refreshed` because this is only used in a hot reloading environment.
    /// Users are expected to only spawn scenes in `LoadState::Done`, which will ensure scene loads and commands
    /// are ordered properly when "hot_reload" is not enabled.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn is_blocked(&self) -> bool
    {
        if !self.any_files_refreshed {
            return false;
        }
        if self.file_counter.get() > 0 {
            return true;
        }
        if self.command_counter.get() > 0 {
            return true;
        }
        if self.commands_unlock_time > get_current_time() {
            return true;
        }
        false
    }

    /// Sets descendants of the 'global root'. These should be files manually loaded in the app.
    pub(crate) fn set_root_file(&mut self, descendants: Vec<CobFile>)
    {
        let file = Self::global_file();
        self.set_file_descendants(file.clone(), descendants);
        self.set_file_commands(file, vec![]);
    }

    /// Tries to update the traversal point to the requested file.
    ///
    /// Will truncate `self.file_order` to the new traversal point.
    #[cfg(feature = "hot_reload")]
    fn update_traversal_point(&mut self, target: CobFile, target_idx: usize)
    {
        // Try to get current target from file order list. If not in the list at `target_idx`, then the target
        // must be *after* the end of the current `file_order` vec. And therefore is *after* the current traversal
        // point, which is always a member of `file_order`.
        let Some(maybe_target) = self.file_order.get(target_idx) else {
            self.traversal_point = self
                .file_order
                .last()
                .cloned()
                .or_else(|| Some(self.root_descendents[0].clone()));
            return;
        };
        if *maybe_target != target {
            self.traversal_point = self
                .file_order
                .last()
                .cloned()
                .or_else(|| Some(self.root_descendents[0].clone()));
            return;
        }

        // Crop files after this file so push-backs in the command traversal are accurate.
        self.file_order.truncate(target_idx + 1);

        // Save the new traversal point.
        self.traversal_point = Some(target);
    }

    /// Tries to update the traversal point to the file before the requested file.
    #[cfg(feature = "hot_reload")]
    fn update_traversal_point_to_prev(&mut self, target: CobFile, target_idx: usize)
    {
        let Some(maybe_target) = self.file_order.get(target_idx) else {
            self.traversal_point = self
                .file_order
                .last()
                .cloned()
                .or_else(|| Some(self.root_descendents[0].clone()));
            return;
        };
        if *maybe_target != target {
            self.traversal_point = self
                .file_order
                .last()
                .cloned()
                .or_else(|| Some(self.root_descendents[0].clone()));
            return;
        }

        self.file_order.truncate(target_idx);
        self.traversal_point = self
            .file_order
            .get(target_idx.saturating_sub(1))
            .cloned()
            .or_else(|| Some(self.root_descendents[0].clone()));
    }

    #[cfg(feature = "hot_reload")]
    fn update_commands_unlock_time(&mut self)
    {
        self.commands_unlock_time = get_current_time() + Duration::from_millis(500);
    }

    /// Sets descendants of a file (no hot reloading).
    /// - This is separate from setting commands to ensure we create entries for all manifest loads immediately
    ///   after extracting a manifest section. This way when manifest loads show up (which can occur out of order),
    ///   they will have a pre-existing slot to land in.
    #[cfg(not(feature = "hot_reload"))]
    pub(crate) fn set_file_descendants(&mut self, file: CobFile, descendants: Vec<CobFile>)
    {
        let Some(info) = self.hierarchy.get_mut(&file) else {
            tracing::error!("failed setting file descendants for unknown file {:?}; all files should be pre-registered \
                as descendants of other files (this is a bug)", file);
            return;
        };

        if info.status != FileStatus::Pending {
            tracing::error!("failed setting descendants {:?} for file {:?}; file was already loaded with descendants \
            {:?} (this is a bug)", descendants, file, info.descendants);
            return;
        }

        // Update status.
        info.status = FileStatus::AwaitingCommands;
        info.initialized = true;

        // Initialize descendants' slots.
        for descendant in descendants.iter() {
            debug_assert!(descendant.as_str() != GLOBAL_PSEUDO_FILE);

            if let Some(prev) = self.hierarchy.insert(
                descendant.clone(),
                FileCommandsInfo {
                    status: FileStatus::Pending,
                    parent: FileParent::Parent(file.clone()),
                    commands: vec![],
                    descendants: self.empty_descendants.clone(),
                    is_orphaned: false,
                    initialized: false,
                },
            ) {
                tracing::warn!("duplicate file {:?} registered in commands buffer; new parent: {:?}, prev info: {:?}",
                    descendant, file, prev);
            } else {
                self.file_counter.add(1);
            }
        }

        // Set descendants.
        self.hierarchy.get_mut(&file).unwrap().descendants = Arc::from(descendants);
    }

    /// Sets descendants of a file (with hot reloading).
    /// - Note that there is a race condition. If: file A manifest loads file B, file B is loading, file A reloads
    ///   but removes file B before file B has loaded, file B loads. Then: file B will be orphaned and its commands
    ///   shouldn't be applied.
    ///     - This is solved by tracking orphaned branches.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn set_file_descendants(&mut self, file: CobFile, descendants: Vec<CobFile>)
    {
        let Some(info) = self.hierarchy.get_mut(&file) else {
            tracing::error!("failed setting file descendants for unknown file {:?}; all files should be pre-registered \
                as descendants of other files (this is a bug)", file);
            return;
        };
        let file_idx = info.idx;
        let file_is_orphaned = info.is_orphaned;

        // Check if already initialized.
        if info.initialized {
            self.any_files_refreshed = true;
        }

        // Check if now pending.
        // - We must not count orphaned files in case a file gets deleted while stuck waiting for commands. We
        // assume all deleted files will eventually be removed from manifest files, meaning they will eventually be
        // orphaned, thereby unblocking us in the command applier step.
        // - This opens a race condition involving a file being reparented, a concurrent arbitrary file change, AND
        //   a
        // concurrent defs change that needs to propagate through the file that was arbitrarily changed. The race
        // condition mainly exists if the reparenting is done by removing the original manifest entry
        // before adding it to the new parent. Specifically, there can be a moment where the reparenting
        // branch is orphaned and the new parent has not been set. If a file outside the orphaned region
        // depends on defs within that region, and those defs get stuck on a file separately refreshed,
        // then the command ordering guarantees we try to enforce here can be violated.
        //
        // The other direction is not a race condition, but only if the order is maintained. If the order gets
        // reversed in the async asset framework machinery, then the race condition resurfaces.
        //
        // There is a simpler race condition where a defs change concurrent with a reparenting can cause a
        // high-priority command to get stuck in the orphaned branch, leading to a dependent command being
        // applied before it.
        //
        // To mitigate these race conditions, we track the most recent orphaning event and delay applying commands
        // until some time has passed.
        if !info.is_orphaned && info.status == FileStatus::Loaded {
            self.file_counter.add(1);
        }

        // Update status.
        info.status = FileStatus::AwaitingCommands;
        info.initialized = true;

        // Update descendants' slots.
        let prev_descendants = std::mem::take(&mut info.descendants);
        let mut seen = std::mem::take(&mut self.seen_cached);
        seen.clear();
        seen.resize(prev_descendants.len(), false);

        for descendant in descendants.iter() {
            debug_assert!(descendant.as_str() != GLOBAL_PSEUDO_FILE);

            // Check if this descendant is already recorded.
            if let Some(pos) = prev_descendants.iter().position(|p| p == descendant) {
                seen[pos] = true;
                continue;
            }

            match self.hierarchy.entry(descendant.clone()) {
                Vacant(entry) => {
                    entry.insert(FileCommandsInfo {
                        status: FileStatus::Pending,
                        parent: FileParent::Parent(file.clone()),
                        commands: vec![],
                        descendants: self.empty_descendants.clone(),
                        is_orphaned: file_is_orphaned,
                        idx: usize::MAX,
                        initialized: false,
                    });

                    // Add pending file status to the counter.
                    if !file_is_orphaned {
                        self.file_counter.add(1);
                    }
                }
                Occupied(mut entry) => {
                    let entry = entry.get_mut();
                    let prev_parent = entry.parent.clone();
                    entry.parent = FileParent::Parent(file.clone());
                    entry.idx = usize::MAX; // Make sure this doesn't point to a valid index.
                    let prev_orphaned = entry.is_orphaned;

                    // Repair the previous parent.
                    match prev_parent {
                        FileParent::SelfIsRoot => {
                            tracing::error!("error while reparenting root file {:?}; the root can't be reparented (this \
                                is a bug)",
                                descendant);
                        }
                        FileParent::SelfIsOrphan => (),
                        FileParent::Parent(parent_file) => {
                            if parent_file.as_str() == GLOBAL_PSEUDO_FILE {
                                tracing::warn!("reparenting file {:?} from the root to {:?}; you are both loading the \
                                    file and have it in the manifest of {:?}, it is recommended to remove the \
                                    manual load (i.e. the app.load({:?}))", descendant, entry.parent, entry.parent, **descendant);
                            } else {
                                tracing::warn!("reparenting file {:?} from {:?} to {:?}; you probably have the file in two manifests, \
                                    it is recommended to remove one of those entries", descendant, parent_file, entry.parent);
                            }

                            let Some(parent_info) = self.hierarchy.get_mut(&parent_file) else {
                                tracing::error!("failed reparenting file {:?}; the parent file {:?} is missing (this is a bug)",
                                    descendant, parent_file);
                                continue;
                            };

                            // NOTE: This can create a hole in the existing file_order, but it should not cause any
                            // problems. The hole will not be accessible (it's
                            // essentially a slight memory leak) and will probably
                            // be repaired when the previous parent refreshes with a correct manifest and the
                            // traversal point gets pushed below the hole.
                            let mut new_descendants = Vec::with_capacity(parent_info.descendants.len());
                            for parent_desc in parent_info.descendants.iter().filter(|d| *d != descendant) {
                                new_descendants.push(parent_desc.clone());
                            }
                            parent_info.descendants = Arc::from(new_descendants);
                        }
                    }

                    // Repair orphan status of reparented branch.
                    if prev_orphaned != file_is_orphaned {
                        let mut stack = std::mem::take(&mut self.stack_cached);
                        stack.push((0, Arc::from([descendant.clone()])));

                        self.iter_hierarchy_mut(
                            "repairing orphan status of files",
                            stack,
                            move |buff, file, info| -> bool {
                                if info.is_orphaned == file_is_orphaned {
                                    tracing::error!("encountered orphaned={} file {:?} that is a child of a \
                                        file {:?} with the same orphan state while switching orphan state (this is a bug)",
                                        file_is_orphaned, file, info.parent);
                                    return false;
                                }

                                // Set orphaned status.
                                info.is_orphaned = file_is_orphaned;

                                let num_pending = info.commands.iter().filter(|c| c.is_pending).count();
                                if file_is_orphaned {
                                    // Invalidate the index for sanity.
                                    info.idx = usize::MAX;

                                    // Remove pending commands from the counter (since they are now stuck on an orphaned branch).
                                    buff.command_counter.remove(num_pending);

                                    // Remove pending file status from the counter.
                                    if info.status != FileStatus::Loaded {
                                        buff.file_counter.remove(1);
                                    }
                                } else {
                                    // Add pending commands to the counter (since they moving off an orphaned branch).
                                    buff.command_counter.add(num_pending);

                                    // Add pending file status to the counter.
                                    if info.status != FileStatus::Loaded {
                                        buff.file_counter.add(1);
                                    }
                                }

                                true
                            }
                        );
                    }
                }
            }
        }

        // If any descendants changed, then truncate the traversal point to the nearest elder file. We need to
        // iterate children of this file, which are ordered before this file but after the nearest elder.
        if !file_is_orphaned && &*prev_descendants != &*descendants {
            // Identify the lowest index in the previous descendants list. We will truncate to the file before
            // that.
            let mut lowest = (file.clone(), file_idx);

            if let Some(first_prev_descendant) = prev_descendants.get(0) {
                let mut stack = std::mem::take(&mut self.inverted_stack_cached);
                stack.push((0, Arc::from([first_prev_descendant.clone()]), false));

                self.iter_hierarchy_inverted_mut("rejiggering traversal point", stack, |_, file, info| -> bool {
                    // Just get the very first file in the inverted branch.
                    lowest = (file.clone(), info.idx);
                    false
                });
            }

            self.update_traversal_point_to_prev(lowest.0, lowest.1);
        }

        // Orphan removed descendants.
        let mut is_orphan_root = true;

        for removed in seen
            .iter()
            .enumerate()
            .filter(|(_, s)| !*s)
            .map(|(idx, _)| &prev_descendants[idx])
        {
            let mut stack = std::mem::take(&mut self.stack_cached);
            stack.push((0, Arc::from([removed.clone()])));

            self.iter_hierarchy_mut(
                "orphaning files",
                stack,
                move |buff, file, info| -> bool {
                    if is_orphan_root {
                        info.parent = FileParent::SelfIsOrphan;

                        // Set this here at the place where orphans are created.
                        buff.update_commands_unlock_time();
                    } else {
                        is_orphan_root = false;
                    }

                    // If already orphaned, no need to traverse.
                    if info.is_orphaned {
                        if !is_orphan_root {
                            tracing::error!("encountered orphaned file {:?} that is a child of a non-orphaned file {:?} \
                                (this is a bug)", file, info.parent);
                        }
                        return false;
                    }

                    // Set orphaned.
                    info.is_orphaned = true;

                    // Remove pending commands from the counter (since they are now stuck on an orphaned branch).
                    let num_pending = info.commands.iter().filter(|c| c.is_pending).count();
                    buff.command_counter.remove(num_pending);

                    // Remove pending file status from the counter.
                    if info.status != FileStatus::Loaded {
                        buff.file_counter.remove(1);
                    }

                    true
                }
            );
        }

        // Set descendants.
        self.hierarchy.get_mut(&file).unwrap().descendants = Arc::from(descendants);

        // Recover memory.
        seen.clear();
        self.seen_cached = seen;
    }

    /// Marks a file as awaiting commands. Used when the file is refreshed due to import dependency changes, where
    /// the file won't be re-processed from scratch.
    #[cfg(feature = "hot_reload")]
    pub(crate) fn prep_commands_refresh(&mut self, file: CobFile)
    {
        let Some(info) = self.hierarchy.get_mut(&file) else {
            tracing::error!("failed setting file descendants for unknown file {:?}; all files should be pre-registered \
                as descendants of other files (this is a bug)", file);
            return;
        };

        // Check if now pending.
        if !info.is_orphaned && info.status == FileStatus::Loaded {
            self.file_counter.add(1);
        }

        // Update status.
        info.status = FileStatus::AwaitingCommands;
    }

    /// Adds commands to a file.
    ///
    /// The incoming commands are expected to be deduplicated.
    pub(crate) fn set_file_commands(&mut self, file: CobFile, commands: Vec<(&'static str, ErasedLoadable)>)
    {
        let Some(info) = self.hierarchy.get_mut(&file) else {
            tracing::error!("failed setting file commands for unknown file {:?}; all files should be pre-registered \
                as descendants of other files (this is a bug)", file);
            return;
        };

        #[cfg(not(feature = "hot_reload"))]
        {
            debug_assert!(info.commands.len() == 0);
        }

        // Convert to CachedCommands.
        let mut new_commands = Vec::with_capacity(commands.len());

        for (_full_type_name, command) in commands {
            #[cfg(not(feature = "hot_reload"))]
            {
                new_commands.push(CachedCommand { command, is_pending: true });
                self.command_counter.add(1);
            }

            // For hot reloading we need to check if existing values are changing.
            #[cfg(feature = "hot_reload")]
            {
                match info
                    .commands
                    .iter_mut()
                    .find(|c| c.command.type_id == command.type_id)
                {
                    Some(matches) => {
                        match matches.command.loadable.equals(&command.loadable) {
                            Some(true) => new_commands.push(matches.clone()),
                            Some(false) => {
                                new_commands.push(CachedCommand { command, is_pending: true });
                                if !info.is_orphaned && !matches.is_pending {
                                    self.command_counter.add(1);
                                }
                            }
                            None => {
                                tracing::error!("failed refreshing command {:?} in {:?}, its reflected value doesn't implement \
                                    PartialEq", _full_type_name, file);
                                continue;
                            }
                        }

                        // Clear this to facilitate the 'num removed' calculation below.
                        matches.is_pending = false;
                    }
                    None => {
                        new_commands.push(CachedCommand { command, is_pending: true });
                        if !info.is_orphaned {
                            self.command_counter.add(1);
                        }
                    }
                }
            }
        }

        // Any pre-existing command that is still marked pending is not present in the new list, so we should
        // remove it from the counter.
        if !info.is_orphaned {
            let num_removed = info.commands.iter().filter(|c| c.is_pending).count();
            self.command_counter.remove(num_removed);
            self.file_counter.remove(1);
        }

        // Save the new commands list.
        info.commands = new_commands;
        if info.status != FileStatus::AwaitingCommands {
            tracing::error!("adding commands for {:?} that is in {:?} not FileStatus::AwaitingCommands (this is a bug)",
                file, info.status);
        }
        info.status = FileStatus::Loaded;

        // Update traversal point if we have pending commands.
        // - Only needed in `hot_reload` because in non-hot-reload we iterate through the hierarchy exactly once.
        #[cfg(feature = "hot_reload")]
        {
            if !info.is_orphaned && info.commands.iter().any(|c| c.is_pending) {
                let idx = info.idx;
                self.update_traversal_point(file.clone(), idx);
            }
        }
    }

    /// Replaces a specific command in a file.
    #[cfg(feature = "editor")]
    pub(crate) fn patch_command(&mut self, file: CobFile, longname: &'static str, command: ErasedLoadable)
    {
        let Some(info) = self.hierarchy.get_mut(&file) else {
            tracing::warn!("failed patching command for unknown file {file:?}");
            return;
        };

        match info
            .commands
            .iter_mut()
            .find(|c| c.command.type_id == command.type_id)
        {
            Some(matches) => match matches.command.loadable.equals(&command.loadable) {
                Some(true) => (),
                Some(false) | None => {
                    if !info.is_orphaned && !matches.is_pending {
                        self.command_counter.add(1);
                    }
                    *matches = CachedCommand { command, is_pending: true };
                }
            },
            None => {
                tracing::warn!("failed patching command {longname} in {file:?}; no previously existing instance \
                    of the given command type");
                return;
            }
        }

        // Update traversal point if we have pending commands.
        if !info.is_orphaned && info.commands.iter().any(|c| c.is_pending) {
            let idx = info.idx;
            self.update_traversal_point(file, idx);
        }
    }

    /// Iterates through the cached hierarchy from the latest traversal point, applying pending commands as they
    /// are encountered.
    pub(super) fn apply_pending_commands(&mut self, c: &mut Commands, callbacks: &LoadableRegistry)
    {
        // Don't apply any commands if any files are pending if at least one file has hot reloaded.
        // - This is needed to avoid race conditions involving multiple files refreshing concurrently. Since defs
        //   can
        // be imported 'backward' in the hierarchy, it is possible for a def change to get stuck on a pending file
        // that is after the traversal point, even though the def change needs to be used before the
        // traversal point. Critically, a file after the traversal point may depend on that change
        // propagating both to itself and the upstream command. We need to guarantee the upstream command
        // is re-applied first, which means ensuring all refreshes fully propagate before applying any
        // refreshed commands.
        if self.any_files_refreshed && self.file_counter.get() > 0 {
            return;
        }

        // Don't apply commands if there are none pending.
        if self.command_counter.get() == 0 {
            return;
        }

        // Don't apply commands if under a time lock caused by file orphaning.
        if self.commands_unlock_time > get_current_time() {
            return;
        }

        // Do nothing if there is no traversal point.
        let Some(traversal_point) = self.traversal_point.take() else { return };
        let mut dummy_scene_ref = SceneRef {
            file: SceneFile::File(traversal_point.clone()),
            path: self.dummy_commands_path.clone(),
        };

        // Stack of descendants.
        // [ is in file_order already, current idx, descendants ]
        let mut stack = std::mem::take(&mut self.inverted_stack_cached);
        let mut recursion_count = 0;

        // Build stack for the current traversal point.
        let Some(info) = self.hierarchy.get(&traversal_point) else {
            tracing::error!("failed applying pending commands; tried accessing file {:?} that is missing \
                (this is a bug)", traversal_point);
            return;
        };
        let mut current = &traversal_point;
        let mut parent = &info.parent;

        loop {
            // Check for recursion.
            recursion_count += 1;
            if recursion_count > 100 {
                tracing::error!("aborting from applying pending commands, recursion limit encountered; there is likely a \
                    manifest file loop");
                return;
            }

            let parent_file = match parent {
                FileParent::SelfIsRoot => {
                    stack.insert(0, (0, self.root_descendents.clone(), false));
                    break;
                }
                #[cfg(feature = "hot_reload")]
                FileParent::SelfIsOrphan => {
                    tracing::error!("aborting from applying pending commands; encountered orphaned file (this is a bug)");
                    return;
                }
                FileParent::Parent(file) => file,
            };

            let Some(parent_info) = self.hierarchy.get(parent_file) else {
                tracing::error!("failed applying pending commands; tried accessing file {:?} that is missing \
                    (this is a bug)", parent_file);
                return;
            };

            let Some(pos) = parent_info.descendants.iter().position(|d| d == current) else {
                tracing::warn!("failed applying pending commands; the parent {:?} of file {:?} doesn't contain the file \
                    in its descendants list {:?} (this is a bug)", parent_file, current, parent_info.descendants);
                return;
            };

            stack.insert(0, (pos, parent_info.descendants.clone(), false));
            current = parent_file;
            parent = &parent_info.parent;
        }

        // If we are 'starting fresh' then all files need to be traversed.
        #[cfg(not(feature = "hot_reload"))]
        {
            // Set the current stack-top to descendants-done, because this 'traversal point' is *after*
            // descendants.
            // - If self is the root then we must be starting from the 'beginning'.
            if info.parent != FileParent::SelfIsRoot {
                stack.last_mut().map(|(_, _, desc_done)| *desc_done = true);
            }
        }
        #[cfg(feature = "hot_reload")]
        {
            if self.file_order.len() == 0 {
                if traversal_point.as_str() != GLOBAL_PSEUDO_FILE {
                    tracing::error!("file order is empty but traversal point {:?} is not the global file (this is a bug)",
                        traversal_point);
                }
            } else {
                // Set the current stack-top to descendants-done, because this 'traversal point' is *after*
                // descendants.
                stack.last_mut().map(|(_, _, desc_done)| *desc_done = true);
            }
        }

        // Traverse hierarchy to collect commands.
        #[cfg(feature = "hot_reload")]
        let mut is_first = true;

        self.iter_hierarchy_inverted_mut(
            "applying pending commands",
            stack,
            move |buff, file, info| -> bool {
                // Save the file.
                #[cfg(feature = "hot_reload")]
                {
                    // Only push back new files.
                    if !is_first || buff.file_order.len() == 0 {
                        info.idx = buff.file_order.len();
                        buff.file_order.push(file.clone());
                    }
                    is_first = false;
                }

                // If this stack member is not ready to extract commands, then we need to 'pause' here and try again
                // later.
                // - Do this before checking if the counter is zero so we don't lose our traversal point.
                if info.status != FileStatus::Loaded {
                    #[cfg(not(feature = "hot_reload"))]
                    {
                        buff.traversal_point = Some(file.clone());
                    }

                    #[cfg(feature = "hot_reload")]
                    buff.update_traversal_point(file.clone(), info.idx);

                    return false;
                }

                // Apply pending commands.
                dummy_scene_ref.file = SceneFile::File(file.clone());

                for cached in info.commands.iter_mut().filter(|c| c.is_pending) {
                    cached.is_pending = false;
                    buff.command_counter.remove(1);

                    let Some(callback) = callbacks.get_for_command(cached.command.type_id) else {
                        tracing::warn!("ignoring command in {:?} that wasn't registered with CobLoadableRegistrationAppExt",
                            file);
                        continue;
                    };

                    c.queue(CommandLoadCommand {
                        callback,
                        scene_ref: dummy_scene_ref.clone(),
                        loadable: cached.command.loadable.clone(),
                    });
                }

                // Check for early-out.
                // - We don't check this for non-hot-reload because in that case once the `buff.traversal_point` equals `None`
                // it can never change to something else. We need to keep iterating forward to find a pending file or
                // the end of the hierarchy.
                #[cfg(feature = "hot_reload")]
                {
                    if buff.command_counter.get() == 0 {
                        return false;
                    }
                }

                true
            }
        );
    }

    #[cfg(feature = "hot_reload")]
    fn iter_hierarchy_mut(
        &mut self,
        action: &str,
        mut stack: Vec<(usize, Arc<[CobFile]>)>,
        mut callback: impl FnMut(&mut Self, &CobFile, &mut FileCommandsInfo) -> bool,
    )
    {
        let mut recursion_count = 0;
        let mut hierarchy = std::mem::take(&mut self.hierarchy);

        while let Some((idx, stack_members)) = stack.pop() {
            // Check for recursion.
            recursion_count += 1;
            if recursion_count > 100 {
                tracing::error!("aborting from {action}, recursion limit encountered; there is likely a \
                    manifest file loop");
                break;
            }

            // Get this stack member's info.
            let Some(info) = hierarchy.get_mut(&stack_members[idx]) else {
                tracing::error!("failed {action}; tried accessing file {:?} that is missing (this is a bug)",
                    stack_members[idx]);
                break;
            };

            // Invoke callback on this stack member.
            if !(callback)(self, &stack_members[idx], info) {
                break;
            }

            // Increment index of this stack entry.
            if idx + 1 < stack_members.len() {
                stack.push((idx + 1, stack_members));
            }

            // Add child stack.
            if info.descendants.len() > 0 {
                stack.push((0, info.descendants.clone()));
            }
        }

        self.hierarchy = hierarchy;
        stack.clear();
        self.stack_cached = stack;
    }

    /// Applies the callback in inverted order, with children ordered before their parents.
    fn iter_hierarchy_inverted_mut(
        &mut self,
        action: &str,
        mut stack: Vec<(usize, Arc<[CobFile]>, bool)>,
        mut callback: impl FnMut(&mut Self, &CobFile, &mut FileCommandsInfo) -> bool,
    )
    {
        let mut recursion_count = 0;
        let mut hierarchy = std::mem::take(&mut self.hierarchy);

        while let Some((idx, stack_members, descendants_are_done)) = stack.pop() {
            // Check for recursion.
            recursion_count += 1;
            if recursion_count > 100 {
                tracing::error!("aborting from {action}, recursion limit encountered; there is likely a \
                    manifest file loop");
                break;
            }

            // Get this stack member's info.
            let Some(info) = hierarchy.get_mut(&stack_members[idx]) else {
                tracing::error!("failed {action}; tried accessing file {:?} that is missing (this is a bug)",
                    stack_members[idx]);
                break;
            };

            // Descendants are always done if there are none.
            let descendants_are_done = descendants_are_done || info.descendants.len() == 0;

            // Callback is invoked only after all descendants have iterated.
            match descendants_are_done {
                false => {
                    // Iterate into children.
                    stack.push((idx, stack_members, false));
                    stack.push((0, info.descendants.clone(), false));
                }
                true => {
                    // Invoke callback on this stack member.
                    if !(callback)(self, &stack_members[idx], info) {
                        break;
                    }

                    if idx + 1 >= stack_members.len() {
                        // Set parent to descendants-done.
                        stack.last_mut().map(|(_, _, desc_done)| *desc_done = true);
                    } else {
                        // Go to the next sibling.
                        stack.push((idx + 1, stack_members, false));
                    }
                }
            }
        }

        self.hierarchy = hierarchy;
        stack.clear();
        self.inverted_stack_cached = stack;
    }
}

//-------------------------------------------------------------------------------------------------------------------

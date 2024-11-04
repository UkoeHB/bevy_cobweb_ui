#[cfg(feature = "hot_reload")]
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::sync::Arc;

use bevy::ecs::world::Command;
use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

struct CommandLoadCommand
{
    callback: fn(&mut World, ReflectedLoadable, SceneRef),
    loadable_ref: SceneRef,
    loadable: ReflectedLoadable,
}

impl Command for CommandLoadCommand
{
    fn apply(self, world: &mut World)
    {
        (self.callback)(world, self.loadable, self.loadable_ref);
    }
}

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
        self.pending += num;
    }

    fn remove(&mut self, num: usize)
    {
        debug_assert!(num <= self.pending);
        self.pending = self.pending.saturating_sub(num);
    }

    fn _get(&self) -> usize
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

#[derive(Debug, Clone)]
enum FileParent
{
    SelfIsRoot,
    #[cfg(feature = "hot_reload")]
    SelfIsOrphan,
    Parent(CafFile),
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
    descendants: Arc<[CafFile]>,

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
}

//-------------------------------------------------------------------------------------------------------------------

const GLOBAL_PSEUDO_FILE: &'static str = "__g.caf";

//-------------------------------------------------------------------------------------------------------------------

/// Manages commands loaded from CAF files to ensure they are applied in global order.
///
/// When the `hot_reload` feature is not enabled, this resource will be removed in schedule
/// `OnExit(LoadState::Loading)`.
#[derive(Resource, Debug)]
pub(crate) struct CommandsBuffer
{
    /// 'Earliest' file in the hierarchy with un-applied commands.
    ///
    /// If `None` then no file is targeted for pending commands.
    traversal_point: Option<CafFile>,

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
    /// Makes a new buffer with pseudo 'global' file at the root level,
    /// where manually-loaded files will be added as descendants.
    pub(super) fn new() -> Self
    {
        let global = Self::global_file();
        let mut buffer = Self {
            traversal_point: Some(global.clone()), // Start traversal at global file.
            hierarchy: HashMap::default(),
            #[cfg(feature = "hot_reload")]
            file_order: vec![global.clone()],
            counter: PendingCommandsCounter::default(),
        };
        buffer.hierarchy.insert(
            global,
            FileCommandsInfo {
                status: FileStatus::Pending,
                parent: FileParent::SelfIsRoot,
                commands: vec![],
                descendants: Arc::from([]),
                #[cfg(feature = "hot_reload")]
                idx: 0,
                is_orphaned: false,
            },
        );

        buffer
    }

    fn global_file() -> CafFile
    {
        CafFile::try_new(GLOBAL_PSEUDO_FILE).unwrap()
    }

    /// Sets descendants of the 'global root'. These should be files manually loaded in the app.
    pub(crate) fn set_root_file(&mut self, descendants: Vec<CafFile>)
    {
        let file = Self::global_file();
        self.set_file_descendants(file.clone(), descendants);
        self.set_file_commands(file, vec![]);
    }

    /// Tries to update the traversal point to the requested file.
    ///
    /// Will truncate `self.file_order` to the new traversal point.
    #[cfg(feature = "hot_reload")]
    fn update_traversal_point(&mut self, target: CafFile, target_idx: usize)
    {
        // Try to get current target from file order list. If not in the list at `target_idx`, then the target
        // must be *after* the end of the current `file_order` vec. And therefore is *after* the current traversal
        // point, which is always a member of `file_order`.
        let Some(maybe_target) = self.file_order.get(target_idx) else {
            self.traversal_point = self.file_order.last().cloned();
            return;
        };
        if *maybe_target != target {
            self.traversal_point = self.file_order.last().cloned();
            return;
        }

        // Crop files after this file so push-backs in the command traversal are accurate.
        self.file_order.truncate(target_idx + 1);

        // Save the new traversal point.
        self.traversal_point = Some(target);
    }

    /// Sets descendants of a file (no hot reloading).
    /// - This is separate from setting commands to ensure we create entries for all manifest loads immediately
    ///   after extracting a manifest section. This way when manifest loads show up (which can occur out of order),
    ///   they will have a pre-existing slot to land in.
    #[cfg(not(feature = "hot_reload"))]
    pub(crate) fn set_file_descendants(&mut self, file: CafFile, descendants: Vec<CafFile>)
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

        // Initialize descendants' slots.
        for descendant in descendants.iter() {
            debug_assert!(descendant.as_str() != GLOBAL_PSEUDO_FILE);

            if let Some(prev) = self.hierarchy.insert(
                descendant.clone(),
                FileCommandsInfo {
                    status: FileStatus::Pending,
                    parent: FileParent::Parent(file.clone()),
                    commands: vec![],
                    descendants: Arc::from([]),
                    is_orphaned: false,
                },
            ) {
                tracing::warn!("duplicate file {:?} registered in commands buffer; new parent: {:?}, prev info: {:?}",
                    descendant, file, prev);
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
    pub(crate) fn set_file_descendants(&mut self, file: CafFile, descendants: Vec<CafFile>)
    {
        let Some(info) = self.hierarchy.get_mut(&file) else {
            tracing::error!("failed setting file descendants for unknown file {:?}; all files should be pre-registered \
                as descendants of other files (this is a bug)", file);
            return;
        };
        let file_idx = info.idx;
        let file_is_orphaned = info.is_orphaned;

        // Update status.
        info.status = FileStatus::AwaitingCommands;

        // Update descendants' slots.
        let prev_descendants = std::mem::take(&mut info.descendants);
        let mut seen: Vec<bool> = vec![];
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
                        descendants: Arc::from([]),
                        is_orphaned: file_is_orphaned,
                        idx: 0,
                    });
                }
                Occupied(mut entry) => {
                    let entry = entry.get_mut();
                    let prev_parent = entry.parent.clone();
                    entry.parent = FileParent::Parent(file.clone());
                    entry.idx = 0; // Make sure this doesn't point to a valid index.
                    let prev_orphaned = entry.is_orphaned;

                    // Repair the previous parent.
                    tracing::warn!("reparenting file {:?} from {:?} to {:?}", descendant, prev_parent, entry.parent);

                    match prev_parent {
                        FileParent::SelfIsRoot => {
                            tracing::error!("error while reparenting root file {:?}; the root can't be reparented",
                                descendant);
                        }
                        FileParent::SelfIsOrphan => (),
                        FileParent::Parent(parent_file) => {
                            let Some(parent_info) = self.hierarchy.get_mut(&parent_file) else {
                                tracing::error!("failed reparenting file {:?}; the parent file {:?} is missing (this is a bug)",
                                    descendant, parent_file);
                                continue;
                            };

                            let mut new_descendants = Vec::with_capacity(parent_info.descendants.len());
                            for parent_desc in parent_info.descendants.iter().filter(|d| *d != descendant) {
                                new_descendants.push(parent_desc.clone());
                            }
                            parent_info.descendants = Arc::from(new_descendants);
                        }
                    }

                    // Repair orphan status of reparented branch.
                    if prev_orphaned != file_is_orphaned {
                        self.iter_hierarchy_mut(
                            "repairing orphan status of files",
                            vec![(0, 0, Arc::from([descendant.clone()]))],
                            move |buff, _, file, info| -> Option<u8> {
                                if info.is_orphaned == file_is_orphaned {
                                    tracing::error!("encountered orphaned={} file {:?} that is a child of a \
                                        file {:?} with the same orphan state while switching orphan state (this is a bug)",
                                        file_is_orphaned, file, info.parent);
                                    return None;
                                }

                                // Set orphaned status.
                                info.is_orphaned = file_is_orphaned;

                                let num_pending = info.commands.iter().filter(|c| c.is_pending).count();
                                if file_is_orphaned {
                                    // Remove pending commands from the counter (since they are now stuck on an orphaned branch).
                                    buff.counter.remove(num_pending);
                                } else {
                                    // Add pending commands to the counter (since they moving off an orphaned branch).
                                    buff.counter.add(num_pending);
                                }

                                Some(0)
                            }
                        );
                    }
                }
            }
        }

        // If any descendants changed, then truncate the traversal point to this file.
        if !file_is_orphaned && &*prev_descendants != &*descendants {
            self.update_traversal_point(file.clone(), file_idx);
        }

        // Orphan removed descendants.
        for removed in seen
            .iter()
            .enumerate()
            .filter(|(_, s)| !*s)
            .map(|(idx, _)| &prev_descendants[idx])
        {
            self.iter_hierarchy_mut(
                "orphaning files",
                vec![(true, 0, Arc::from([removed.clone()]))],
                move |buff, is_orphan_root, file, info| -> Option<bool> {
                    if is_orphan_root {
                        info.parent = FileParent::SelfIsOrphan;
                    }

                    // If already orphaned, no need to traverse.
                    if info.is_orphaned {
                        if !is_orphan_root {
                            tracing::error!("encountered orphaned file {:?} that is a child of a non-orphaned file {:?} \
                                (this is a bug)", file, info.parent);
                        }
                        return None;
                    }

                    // Set orphaned.
                    info.is_orphaned = true;

                    // Remove pending commands from the counter (since they are now stuck on an orphaned branch).
                    let num_pending = info.commands.iter().filter(|c| c.is_pending).count();
                    buff.counter.remove(num_pending);

                    Some(false)
                }
            );
        }

        // Set descendants.
        self.hierarchy.get_mut(&file).unwrap().descendants = Arc::from(descendants);
    }

    /// Adds commands to a file.
    ///
    /// The incoming commands are expected to be deduplicated.
    pub(crate) fn set_file_commands(&mut self, file: CafFile, commands: Vec<(&'static str, ErasedLoadable)>)
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

        // Convert to CachedCommands while (for hot reloading) checking if values changed.
        let mut new_commands = Vec::with_capacity(commands.len());

        for (_full_type_name, command) in commands {
            #[cfg(not(feature = "hot_reload"))]
            {
                new_commands.push(CachedCommand { command, is_pending: true });
                self.counter.add(1);
            }

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
                                    self.counter.add(1);
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
                            self.counter.add(1);
                        }
                    }
                }
            }
        }

        // Any pre-existing command that is still marked pending is not present in the new list, so we should
        // remove it from the counter.
        if !info.is_orphaned {
            let num_removed = info.commands.iter().filter(|c| c.is_pending).count();
            self.counter.remove(num_removed);
        }

        // Save the new commands list.
        info.commands = new_commands;
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

    /// Iterates through the cached hierarchy from the latest traversal point, applying pending commands as they
    /// are encountered. iterate forward, applying commands from loaded files; stop when number of applied
    /// commands reaches zero,
    // or when encounter a pending file
    // - if file's parent doesn't match the known parent, then skip it and warn (can occur due to duplicates)
    // - push back to `file_order` as files are traversed
    // - update the `idx` in each file's info
    // - set recursion limit in case of manifest loop (a cap of 100 is probably more than adequate for a file
    //   hierarchy)
    pub(super) fn apply_pending_commands(&mut self, c: &mut Commands, callbacks: &LoaderCallbacks)
    {
        let Some(traversal_point) = self.traversal_point.take() else { return };
        let mut dummy_loadable_ref = SceneRef {
            file: SceneFile::File(traversal_point.clone()),
            path: ScenePath::new("#commands"),
        };

        // Stack of descendants.
        // [ is in file_order already, current idx, descendants ]
        let mut stack: Vec<(bool, usize, Arc<[CafFile]>)> = vec![];
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
                    stack.insert(0, (true, 0, Arc::from([current.clone()])));
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

            stack.insert(0, (true, pos, parent_info.descendants.clone()));
            current = parent_file;
            parent = &parent_info.parent;
        }

        // Traverse hierarchy to collect commands.
        self.iter_hierarchy_mut(
            "applying pending commands",
            stack,
            move |buff, _in_file_order, file, info| -> Option<bool> {
                // Save the file.
                #[cfg(feature = "hot_reload")]
                {
                    // Only push back new files.
                    if !_in_file_order {
                        info.idx = buff.file_order.len();
                        buff.file_order.push(file.clone());
                    }
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

                    return None;
                }

                // Apply pending commands.
                dummy_loadable_ref.file = SceneFile::File(file.clone());

                for cached in info.commands.iter_mut().filter(|c| c.is_pending) {
                    cached.is_pending = false;
                    buff.counter.remove(1);

                    let Some(callback) = callbacks.get_for_command(cached.command.type_id) else {
                        tracing::warn!("ignoring command in {:?} that wasn't registered with CobwebAssetRegistrationAppExt",
                            file);
                        continue;
                    };

                    c.add(CommandLoadCommand {
                        callback,
                        loadable_ref: dummy_loadable_ref.clone(),
                        loadable: cached.command.loadable.clone(),
                    });
                }

                // Check for early-out.
                // - We don't check this for non-hot-reload because in that case once the `buff.traversal_point` equals `None`
                // it can never change to something else. We need to keep iterating forward to find a pending file or
                // the end of the hierarchy.
                // TODO: the logic here is slightly spaghetti
                #[cfg(feature = "hot_reload")]
                {
                    if buff.counter._get() == 0 {
                        return None;
                    }
                }

                Some(false)
            }
        );
    }

    /// The callback should return the next `T` value to save in downstream hierarchy entries.
    fn iter_hierarchy_mut<T: Clone>(
        &mut self,
        action: &str,
        mut stack: Vec<(T, usize, Arc<[CafFile]>)>,
        mut callback: impl FnMut(&mut Self, T, &CafFile, &mut FileCommandsInfo) -> Option<T>,
    )
    {
        let mut recursion_count = 0;
        let mut hierarchy = std::mem::take(&mut self.hierarchy);

        while let Some((custom_data, idx, stack_members)) = stack.pop() {
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
                self.hierarchy = hierarchy;
                return;
            };

            // Invoke callback on this stack member.
            let next_custom_data = match (callback)(self, custom_data, &stack_members[idx], info) {
                Some(next_custom_data) => next_custom_data,
                None => {
                    self.hierarchy = hierarchy;
                    return;
                }
            };

            // Increment index of this stack entry.
            if idx + 1 < stack_members.len() {
                stack.push((next_custom_data.clone(), idx + 1, stack_members));
            }

            // Add child stack.
            if info.descendants.len() > 0 {
                stack.push((next_custom_data, 0, info.descendants.clone()));
            }
        }

        self.hierarchy = hierarchy;
    }
}

//-------------------------------------------------------------------------------------------------------------------

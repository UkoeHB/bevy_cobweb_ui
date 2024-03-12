//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn layout_root(
){
    
    // Traverse all root nodes until there are no nodes marked dirty.
    // - If a node is not dirty and does not have changed SizeRef, skip it (but run its children unless there are zero dirty).
    // - Handle dirty frames immediately after updating their non-derived children.


    // Mark self non-dirty.

    // Get SizeRef, set reactive component
    // - SizeRefSource
    // - Default to parent size else log error if parent has no size.

    // Skip check: node not dirty and SizeRef did not change.
    // - Abort if direct update or zero dirty remaining.

        // Compute NodeSizeEstimate
        // - SizeRef: Dims, MinDims, MaxDims
        // Get NodeSizeAdjustment
        // - NodeSizeAdjuster enum with optional callback: fn(&world, entity, size_estimate, min, max) (also FnMut boxed option)
        // - Default to zero.
        // Compute NodeSize, set reactive component
        // - NodeSizeAdjustment, NodeSizeEstimate, (SizeRef: MinDims, MaxDims)

        // ## Handle Position node
        // Compute Transform
        // - Position, NodeSize, SizeRef

    // Update non-derived children
    // - Track if child NodeSize changes.

    // ## Handle Frame node
    // Skip check: node not dirty and no non-derived child NodeSize changes.
    // - Abort if direct update or zero dirty remaining.
        // Compute child Transforms and derived child size recommendations
        // - Frame, NodeSize, Member NodeSizes, Member InFrames, Member InFrameDeriveds
        // Update derived children
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn layout_direct(
){

    // Target-update nodes marked dirty after applying deferred.
    // - If an encountered node is not dirty and does not have changed SizeRef, stop propagating.
    // - Handle dirty frames immediately after updating their non-derived children.
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn layout_full_traversal(
    mut tracker: ResMut<DirtyNodeTracker>,
){
    for node in root_nodes.iter()
    {
        // Update the root and its progeny.
        layout_root();
    }

    // Clear the tracker.
    // - This will remove any nodes that were despawned after being marked dirty.
    tracker.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn layout_targeted_traversal(
    mut tracker: ResMut<DirtyNodeTracker>,
){
    // Take dirty list.
    // - Note: Entries will never be inserted to the tracker while updating layout.
    let mut dirty = tracker.take_list();

    // Target-update all dirty.
    for node in dirty.drain(..)
    {
        // Skip nodes that have already been removed from the tracker.
        if !tracker.contains(node) { continue; }

        // Update the dirty node and its progeny.
        layout_direct();
    }

    // Return dirty list.
    tracker.return_list(dirty);

    // Clear the tracker.
    // - Sanity check, the tracker should already be empty.
    debug_assert_eq!(tracker.len(), 0);
    tracker.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the layout of UI nodes.
///
/// This is an iterative process because each 'layout pass' can trigger reactions that require further layout passes.
///
/// Layout performance is a tradeoff between the cost of traversing non-dirty nodes and the cost of redundantly recomputing
/// nodes.
/// We assume full tree traversal is more efficient in the general case, so we do a full tree traversal to start (starting
/// at root nodes).
///
/// After the initial traversal, we assume newly dirty nodes are less likely to intersect with each other, so follow-up
/// passes are only partial traversals that start at dirty nodes.
/// Note that this may cause nodes to be updated redundantly, which may also cause redundant reactions to those nodes.
fn layout(world: &mut World)
{
    // Traverse the entire tree once if there are multiple dirty nodes.
    if world.resource::<DirtyNodeTracker>().len() > 1
    {
        world.syscall((), layout_full_traversal);
    }

    // Cleanup after full traversal.
    while world.resource::<DirtyNodeTracker>().len() > 0
    {
        world.syscall((), layout_targeted_traversal);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

//todo: system to mark Frame nodes dirty if their Children changes
//todo: system to mark CobwebNode nodes dirty if their Parent changes

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct DirtyNodeTracker
{
    dirty: EntityHashSet,
    list: Vec<Entity>,
}

impl DirtyNodeTracker
{
    pub(crate) fn insert(&mut self, entity: Entity)
    {
        let _ = self.dirty.insert(entity);
        self.list.push(entity);
    }

    fn len(&self) -> usize
    {
        self.dirty.len()
    }

    fn contains(&mut self, entity: Entity) -> bool
    {
        self.dirty.contains(entity)
    }

    fn remove(&mut self, entity: Entity) -> bool
    {
        self.dirty.remove(entity)
    }

    fn take_list(&mut self) -> Vec<Entity>
    {
        let list = self.list;
        self.list = Vec::default();
        list
    }

    fn return_list(&mut self, list: Vec<Entity>)
    {
        debug_assert_eq!(self.list.len(), 0);
        self.list = list;
    }

    fn clear(&mut self)
    {
        self.dirty.clear();
        self.list.clear();
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LayoutAlgorithmPlugin;

impl Plugin for LayoutAlgorithmPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<ZLevel>()
            .add_systems(PostUpdate, update_z_order.in_set(LayoutSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------

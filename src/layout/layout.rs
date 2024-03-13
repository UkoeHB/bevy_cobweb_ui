//! Updates the layout of UI nodes.
//!
//! This is an iterative process because each 'layout pass' can trigger reactions that require further layout passes.
//!
//! Layout performance is a tradeoff between the cost of traversing non-dirty nodes and the cost of redundantly recomputing
//! nodes.
//! We assume full tree traversal is more efficient in the general case, so we do a full tree traversal to start (starting
//! at root nodes).
//!
//! After the initial traversal, we assume newly dirty nodes are less likely to intersect with each other, so follow-up
//! passes are only partial traversals that start at dirty nodes.
//! Note that this may cause nodes to be updated redundantly, which may also cause redundant reactions to those nodes.


//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::system::SystemParam;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemParam)]
struct ProcessNodeParamsReact<'w, 's>
{
    rc: ReactCommands<'w, 's>,
    sizeref: Query<'w, 's, &'static mut React<SizeRef>, With<CobwebNode>>,
    nodesize: Query<'w, 's, &'static mut React<NodeSize>, With<CobwebNode>>,
}

impl<'w, 's> ProcessNodeParamsReact<'w, 's>
{
    /// Sets a new [`SizeRef`] value and returns the old one if changed.
    ///
    /// Logs a warning if the entity has no existing [`SizeRef`] component.
    fn set_sizeref(&mut self, node: Entity, new_sizeref: SizeRef) -> Option<SizeRef>
    {
        let Ok(mut sizeref) = self.sizeref.get_mut(node)
        else
        {
            tracing::warn!("failed setting SizeRef on {:?}, SizeRef component is missing", node);
            return None;
        };
        sizeref.set_if_not_eq(&mut self.rc, new_sizeref)
    }

    /// Sets a new [`NodeSize`] value and returns the old one if changed.
    ///
    /// Logs a warning if the entity has no existing [`NodeSize`] component.
    fn set_nodesize(&mut self, node: Entity, new_nodesize: NodeSize) -> Option<NodeSize>
    {
        let Ok(mut nodesize) = self.nodesize.get_mut(node)
        else
        {
            tracing::warn!("failed setting NodeSize on {:?}, NodeSize component is missing", node);
            return None;
        };
        nodesize.set_if_not_eq(&mut self.rc, new_nodesize)
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemParam)]
struct ProcessNodeParams<'w, 's>
{
    inner: ParamSet<'w, 's,(
        &'w World,
        ResMut<'w, DirtyNodeTracker>,
        Query<'w, 's, &'static mut Transform, With<CobwebNode>>,
        ProcessNodeParamsReact<'w, 's>,
    )>,
}

impl<'w, 's> ProcessNodeParams<'w, 's>
{
    fn world(&mut self) -> &World
    {
        self.inner.p0()
    }
    fn tracker(&mut self) -> ResMut<DirtyNodeTracker>
    {
        self.inner.p1()
    }
    fn transform(&mut self) -> Query<&'static mut Transform, With<CobwebNode>>
    {
        self.inner.p2()
    }
    fn react(&mut self) -> ProcessNodeParamsReact
    {
        self.inner.p3()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(PartialEq)]
enum ProcessNodeResult
{
    AbortNoChange,
    NoSizeChange,
    SizeChange,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_node_position(
    params: &mut ProcessNodeParams,
    node: Entity,
    sizeref: SizeRef,
    nodesize: NodeSize,
    position: &Position,
){
    let mut query = params.transform();
    let Ok(mut transform) = query.get_mut(node)
    else
    {
        tracing::warn!("failed updating Transform on node {:?} with {:?}, Transform is missing", node, position);
        return;
    };

    compute_new_transform(sizeref, nodesize, position, &mut transform);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_node_size(
    params: &mut ProcessNodeParams,
    dims: &Query<&React<Dims>, With<CobwebNode>>,
    //mindims: &Query<&React<MinDims>, With<CobwebNode>>,
    //maxdims: &Query<&React<MaxDims>, With<CobwebNode>>,
    //adjuster: &Query<&React<NodeSizeAdjuster>, With<CobwebNode>>,
    position: &Query<&React<Position>, With<CobwebNode>>,
    node: Entity,
    sizeref: SizeRef,
) -> ProcessNodeResult
{
    // Compute `NodeSizeEstimate` for the node.
    // - Uses `Dims::default()` if the node has no `Dims`.
    //todo include MinDims, MaxDims
    let new_nodesize = NodeSize(dims.get(node).map(|d| **d).unwrap_or_default().compute(*sizeref));

    //todo: Get NodeSizeAdjustment
    // - NodeSizeAdjuster enum with optional callback: fn(&world, entity, size_estimate, min, max) (also FnMut boxed option)
    // - Default to zero.

    // Compute adjusted `NodeSize` value.
    //todo: use NodeSizeAdjustment, NodeSizeEstimate, (SizeRef: MinDims, MaxDims)

    // Update the node's `NodeSize` component.
    let prev_nodesize = params.react().set_nodesize(node, new_nodesize);

    // Update position if this is a `Position` node.
    if let Ok(position) = position.get(node)
    {
        update_node_position(params, node, sizeref, new_nodesize, &*position);
    }

    match prev_nodesize.is_some()
    {
        true => ProcessNodeResult::SizeChange,
        false => ProcessNodeResult::NoSizeChange,
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn process_node_layout(
    is_full_traversal: bool,
    params: &mut ProcessNodeParams,
    nodes: &Query<(), With<CobwebNode>>,
    source: &Query<&React<SizeRefSource>, With<CobwebNode>>,
    dims: &Query<&React<Dims>, With<CobwebNode>>,
    //mindims: &Query<&React<MinDims>, With<CobwebNode>>,
    //maxdims: &Query<&React<MaxDims>, With<CobwebNode>>,
    //adjuster: &Query<&React<NodeSizeAdjuster>, With<CobwebNode>>,
    position: &Query<&React<Position>, With<CobwebNode>>,
    children: &Query<&Children, With<CobwebNode>>,
    //frame: &Query<&React<Frame>, With<CobwebNode>>,
    //inframe: &Query<&React<InFrame>, With<CobwebNode>>,
    //derived: &Query<&React<InFrameDerived>, With<CobwebNode>>,
    node: Entity,
) -> ProcessNodeResult
{
    // Mark self non-dirty.
    let node_dirty = params.tracker().remove(node);

    // Compute new `SizeRef` for the node.
    // - Uses `SizeRefSource::default()` if the node has no `SizeRefSource`.
    let new_sizeref = source.get(node).map(|s| &**s).unwrap_or(&SizeRefSource::default()).compute(params.world(), node);
    let prev_sizeref = params.react().set_sizeref(node, new_sizeref);

    // We need to update if the node was dirty or its `SizeRef` changed.
    let need_update = node_dirty || prev_sizeref.is_some();

    // Abort if we don't need to update and this isn't a full traversal (or there are zero dirty remaining).
    if !need_update && (!is_full_traversal || params.tracker().len() == 0)
    {
        return ProcessNodeResult::AbortNoChange;
    }

    // Update the node size.
    let node_result = match need_update
    {
        true => update_node_size(params, dims, position, node, new_sizeref),
        false => ProcessNodeResult::NoSizeChange,
    };

    // Update non-derived node children.
    // - Note that node children potentially depend on this node's `NodeSize`, so we must do this after updating this
    //   node's node size.
    let mut _child_changed = false;
    for child in children.get(node).ok().into_iter().map(|c| c.iter()).flatten()
    {
        // Skip non-node children.
        if nodes.contains(*child) { continue; }

        let child_result = process_node_layout(
            is_full_traversal,
            params,
            nodes,
            source,
            dims,
            //mindims,
            //maxdims,
            //adjuster,
            position,
            children,
            //frame,
            //inframe,
            //derived,
            *child,
        );

        if child_result == ProcessNodeResult::SizeChange { _child_changed = true; }
    }

    // ## Handle Frame node
    // Skip check: node not dirty and no non-derived child NodeSize changes.
        // Compute child Transforms and derived child size recommendations
        // - Frame, NodeSize, Member NodeSizes, Member InFrames, Member InFrameDeriveds
        // Update derived children

    node_result
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn layout_full_traversal(
    root_nodes: Query<&Children, With<UiRoot>>,
    mut params: ProcessNodeParams,
    nodes: Query<(), With<CobwebNode>>,
    source: Query<&React<SizeRefSource>, With<CobwebNode>>,
    dims: Query<&React<Dims>, With<CobwebNode>>,
    //mindims: Query<&React<MinDims>, With<CobwebNode>>,
    //maxdims: Query<&React<MaxDims>, With<CobwebNode>>,
    //adjuster: Query<&React<NodeSizeAdjuster>, With<CobwebNode>>,
    position: Query<&React<Position>, With<CobwebNode>>,
    children: Query<&Children, With<CobwebNode>>,
    //frame: Query<&React<Frame>, With<CobwebNode>>,
    //inframe: Query<&React<InFrame>, With<CobwebNode>>,
    //derived: Query<&React<InFrameDerived>, With<CobwebNode>>,
){
    // Iterate children of root nodes.
    for node in root_nodes.iter().map(|c| c.iter()).flatten()
    {
        process_node_layout(
            true,
            &mut params,
            &nodes,
            &source,
            &dims,
            //&mindims,
            //&maxdims,
            //&adjuster,
            &position,
            &children,
            //&frame,
            //&inframe,
            //&derived,
            *node,
        );
    }

    // Clear the tracker.
    // - This will remove any nodes that weren't updated by the traversal (implying they were despawned).
    params.tracker().clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn layout_targeted_traversal(
    mut params: ProcessNodeParams,
    nodes: Query<(), With<CobwebNode>>,
    source: Query<&React<SizeRefSource>, With<CobwebNode>>,
    dims: Query<&React<Dims>, With<CobwebNode>>,
    //mindims: Query<&React<MinDims>, With<CobwebNode>>,
    //maxdims: Query<&React<MaxDims>, With<CobwebNode>>,
    //adjuster: Query<&React<NodeSizeAdjuster>, With<CobwebNode>>,
    position: Query<&React<Position>, With<CobwebNode>>,
    children: Query<&Children, With<CobwebNode>>,
    //frame: Query<&React<Frame>, With<CobwebNode>>,
    //inframe: Query<&React<InFrame>, With<CobwebNode>>,
    //derived: Query<&React<InFrameDerived>, With<CobwebNode>>,
){
    // Take dirty list.
    // - Note: Entries will never be inserted to the tracker while updating layout.
    let mut dirty = params.tracker().take_list();

    // Target-update all dirty.
    for node in dirty.drain(..)
    {
        // Skip nodes that have already been removed from the tracker.
        if !params.tracker().contains(node) { continue; }

        process_node_layout(
            false,
            &mut params,
            &nodes,
            &source,
            &dims,
            //&mindims,
            //&maxdims,
            //&adjuster,
            &position,
            &children,
            //&frame,
            //&inframe,
            //&derived,
            node,
        );
    }

    // Return dirty list.
    let mut tracker = params.tracker();
    tracker.return_list(dirty);

    // Clear the tracker.
    // - Sanity check: the tracker should already be empty.
    debug_assert_eq!(tracker.len(), 0);
    tracker.clear();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const LAYOUT_TRAVERSAL_CAP: usize = 10_000;

fn layout_targeted_traversal_loop(world: &mut World)
{
    // Exclusive loop to handle trailing reactions.
    let mut loop_count = 0;
    while world.resource::<DirtyNodeTracker>().len() > 0
    {
        loop_count += 1;
        if loop_count > LAYOUT_TRAVERSAL_CAP
        {
            tracing::error!("aborting layout traversal after {:?} iterations, probably because of looping reactions",
                LAYOUT_TRAVERSAL_CAP);
            return;
        }

        world.syscall((), layout_targeted_traversal);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

//todo: system to mark Frame nodes dirty if their Children changes
//todo: system to mark CobwebNode nodes dirty if their Parent changes
//todo: HierarchyEvent::ChildRemoved + RemovedComponents<FrameMember> -> trigger Frame rebuild on child removal (i.e. despawn)
//      - is this covered by tracking Children changes?

//todo: Frame dirty on member insert, member dirty on insert

//todo: track insertions/removals/mutations of the following (ignore removals caused by despawns)
// - SizeRefSource, Dims, MinDims, MaxDims, NodeSizeAdjuster, Position, Frame, InFrame, InFrameDerived

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Tracks UI nodes that need layout updates this tick.
///
/// Use [`Self::insert`] to mark nodes dirty as needed.
///
/// Built-in [`UiInstructions`](UiInstruction) will automatically mark nodes dirty when their tracked components are
/// changed (e.g. [`Dims`], [`Position`], etc.).
/// Hierarchy changes are automatically detected, and the relevant nodes will be updated.
#[derive(Resource, Default)]
pub struct DirtyNodeTracker
{
    dirty: EntityHashSet,
    list: Vec<Entity>,
}

impl DirtyNodeTracker
{
    /// Inserts a dirty entity to the tracker.
    pub fn insert(&mut self, entity: Entity)
    {
        let _ = self.dirty.insert(entity);
        self.list.push(entity);
    }

    /// Gets the number of dirty nodes.
    pub fn len(&self) -> usize
    {
        self.dirty.len()
    }

    /// Checks if an entity is currently marked dirty.
    pub fn contains(&mut self, entity: Entity) -> bool
    {
        self.dirty.contains(&entity)
    }

    fn remove(&mut self, entity: Entity) -> bool
    {
        self.dirty.remove(&entity)
    }

    fn take_list(&mut self) -> Vec<Entity>
    {
        let mut temp = Vec::default();
        std::mem::swap(&mut self.list, &mut temp);
        temp
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
        app.init_resource::<DirtyNodeTracker>()
            // These systems are broken into separate steps to improve parallelism as much as possible. The improvement
            // is 'not much' because layout uses a `&World` reference, which will prevent most other systems from
            // running in parallel.
            .add_systems(PostUpdate,
                (
                    apply_deferred,
                    (
                        layout_full_traversal,
                        apply_deferred,
                    )
                        .run_if(|t: Res<DirtyNodeTracker>| t.len() > 1),
                    (
                        layout_targeted_traversal,
                        apply_deferred,
                    )
                        .run_if(|t: Res<DirtyNodeTracker>| t.len() > 0),
                    layout_targeted_traversal_loop.run_if(|t: Res<DirtyNodeTracker>| t.len() > 0),
                ).in_set(LayoutSetCompute));
    }
}

//-------------------------------------------------------------------------------------------------------------------

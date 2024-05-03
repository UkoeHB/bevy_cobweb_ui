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

use crate::*;

use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy_cobweb::prelude::*;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemParam)]
struct ProcessNodeParamsReact<'w, 's>
{
    c: Commands<'w, 's>,
    nodesize: Query<'w, 's, &'static mut React<NodeSize>, With<CobwebNode>>,
}

impl<'w, 's> ProcessNodeParamsReact<'w, 's>
{
    /// Sets a new [`NodeSize`] value and returns the old one if changed.
    fn set_nodesize(&mut self, node: Entity, new_nodesize: NodeSize) -> Option<NodeSize>
    {
        let Ok(mut nodesize) = self.nodesize.get_mut(node)
        else
        {
            tracing::warn!("failed setting NodeSize on {:?}, NodeSize component is missing", node);
            return None;
        };
        nodesize.set_if_neq(&mut self.c, new_nodesize)
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
        Query<'w, 's, &'static mut BaseSizeRef, With<CobwebNode>>,
        Query<'w, 's, &'static mut SizeRef, With<CobwebNode>>,
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

    fn base_sizeref(&mut self) -> Query<&'static mut BaseSizeRef, With<CobwebNode>>
    {
        self.inner.p3()
    }

    fn sizeref(&mut self) -> Query<&'static mut SizeRef, With<CobwebNode>>
    {
        self.inner.p4()
    }

    fn react(&mut self) -> ProcessNodeParamsReact
    {
        self.inner.p5()
    }

    /// Sets a new [`BaseSizeRef`] value and returns `true` if changed.
    fn set_base_sizeref(&mut self, node: Entity, new_base_sizeref: BaseSizeRef) -> bool
    {
        let mut query = self.base_sizeref();
        let Ok(mut base_sizeref) = query.get_mut(node)
        else
        {
            tracing::warn!("failed setting BaseSizeRef on {:?}, BaseSizeRef component is missing", node);
            return false;
        };
        if *base_sizeref == new_base_sizeref { return false; }
        *base_sizeref = new_base_sizeref;

        true
    }

    /// Sets a new [`SizeRef`] value and returns `true` if changed.
    fn set_sizeref(&mut self, node: Entity, new_sizeref: SizeRef) -> bool
    {
        let mut query = self.sizeref();
        let Ok(mut sizeref) = query.get_mut(node)
        else
        {
            tracing::warn!("failed setting SizeRef on {:?}, SizeRef component is missing", node);
            return false;
        };
        if *sizeref == new_sizeref { return false; }
        *sizeref = new_sizeref;

        true
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(PartialEq)]
enum ProcessNodeResult
{
    NotANode,
    AbortNoChange,
    NoSizeChange,
    SizeChange,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn get_base_sizeref(params: &mut ProcessNodeParams, node: Entity) -> Option<BaseSizeRef>
{
    let Some(parent) = params.world().get::<bevy::hierarchy::Parent>(node).map(|p| **p) else { return None; };
    params.base_sizeref()
        .get(parent)
        .map(|b| *b)
        .ok()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_node_position(
    params: &mut ProcessNodeParams,
    node: Entity,
    sizeref: SizeRef,
    nodesize: NodeSize,
    position: &Position2d,
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
    dims: &Query<&React<Dims2d>, With<CobwebNode>>,
    //adjuster: &Query<&React<NodeSizeAdjuster>, With<CobwebNode>>,
    position: &Query<&React<Position2d>, With<CobwebNode>>,
    node: Entity,
    _base_sizeref: BaseSizeRef,
    sizeref: SizeRef,
) -> ProcessNodeResult
{
    // Compute `NodeSizeEstimate` for the node.
    let new_nodesize = NodeSize(dims.get(node).map(|d| **d).unwrap().compute(*sizeref));

    //todo: Get NodeSizeAdjustment
    // - NodeSizeAdjuster enum with optional callback: fn(&world, entity, base_sizeref, size_estimate, dims)
    //   - (also FnMut boxed option)
    // - Default to zero.

    // Compute adjusted `NodeSize` value.
    //todo: use NodeSizeAdjustment, NodeSizeEstimate, (SizeRef: MinDims, MaxDims)

    // Update the node's `NodeSize` component.
    let prev_nodesize = params.react().set_nodesize(node, new_nodesize);

    // Update position if this is a `Position2d` node.
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
    nodes: &Query<(), (With<CobwebNode>, With<React<Dims2d>>)>,
    source: &Query<&React<SizeRefSource>, With<CobwebNode>>,
    dims: &Query<&React<Dims2d>, With<CobwebNode>>,
    //adjuster: &Query<&React<NodeSizeAdjuster>, With<CobwebNode>>,
    position: &Query<&React<Position2d>, With<CobwebNode>>,
    children: &Query<&Children, With<CobwebNode>>,
    //frame: &Query<&React<Frame>, With<CobwebNode>>,
    //inframe: &Query<&React<InFrame>, With<CobwebNode>>,
    //derived: &Query<&React<InFrameDerived>, With<CobwebNode>>,
    node: Entity,
    base_sizeref: Option<BaseSizeRef>,
) -> ProcessNodeResult
{
    // Check if this is actually a node.
    if !nodes.contains(node) { return ProcessNodeResult::NotANode; }

    // Mark self non-dirty.
    let node_dirty = params.tracker().remove(node);

    // Compute new `SizeRef` for the node.
    // - Uses `SizeRefSource::default()` if the node has no `SizeRefSource`.
    let new_sizeref = source.get(node)
        .map(|s| &**s).unwrap_or(&SizeRefSource::default())
        .compute(params.world(), base_sizeref, node);

    // If we have no `BaseSizeRef` then this is a base node, so set it to our own SizeRef.
    let base_sizeref = base_sizeref.unwrap_or(BaseSizeRef{ base: node, sizeref: new_sizeref });

    // Save sizerefs.
    let base_sizeref_changed = params.set_base_sizeref(node, base_sizeref);
    let prev_sizeref_changed = params.set_sizeref(node, new_sizeref);

    // We need to update if the node was dirty or its `BaseSizeRef` or `SizeRef` changed.
    let need_update = node_dirty || base_sizeref_changed || prev_sizeref_changed;

    // Abort if we don't need to update and this isn't a full traversal (or there are zero dirty remaining).
    if !need_update && (!is_full_traversal || params.tracker().len() == 0)
    {
        return ProcessNodeResult::AbortNoChange;
    }

    // Update the node size.
    let node_result = match need_update
    {
        true => update_node_size(params, dims, position, node, base_sizeref, new_sizeref),
        false => ProcessNodeResult::NoSizeChange,
    };

    // Update non-derived node children.
    // - Note that node children potentially depend on this node's `NodeSize`, so we must do this after updating this
    //   node's node size.
    let mut _child_changed = false;
    for child in children.get(node).ok().into_iter().map(|c| c.iter()).flatten()
    {
        let child_result = process_node_layout(
            is_full_traversal,
            params,
            nodes,
            source,
            dims,
            //adjuster,
            position,
            children,
            //frame,
            //inframe,
            //derived,
            *child,
            Some(base_sizeref),
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
    root_nodes: Query<&Children, With<Ui2DRoot>>,
    mut params: ProcessNodeParams,
    nodes: Query<(), With<CobwebNode>>,
    source: Query<&React<SizeRefSource>, With<CobwebNode>>,
    dims: Query<&React<Dims2d>, With<CobwebNode>>,
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
            //&adjuster,
            &position,
            &children,
            //&frame,
            //&inframe,
            //&derived,
            *node,
            None,
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
    dims: Query<&React<Dims2d>, With<CobwebNode>>,
    //adjuster: Query<&React<NodeSizeAdjuster>, With<CobwebNode>>,
    position: Query<&React<Position2d>, With<CobwebNode>>,
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

        // Get base sizeref from parent if possible.
        let base_sizeref = get_base_sizeref(&mut params, node);

        process_node_layout(
            false,
            &mut params,
            &nodes,
            &source,
            &dims,
            //&adjuster,
            &position,
            &children,
            //&frame,
            //&inframe,
            //&derived,
            node,
            base_sizeref,
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

const LAYOUT_TRAVERSAL_CAP: usize = 1_000;

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

pub(crate) struct LayoutAlgorithmPlugin;

impl Plugin for LayoutAlgorithmPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            // These systems are broken into separate steps to improve parallelism as much as possible. The improvement
            // is 'not much' because layout uses a `&World` reference, which will prevent most other systems from
            // running in parallel.
            //
            // Note that each step clears the dirty tracker, and then `apply_deferred` executes deferred updates
            // that *may* result in more dirty nodes.
            .add_systems(PostUpdate,
                (
                    apply_deferred,
                    (
                        layout_full_traversal,
                        apply_deferred,
                    )
                        .chain()
                        .run_if(|t: Res<DirtyNodeTracker>| t.len() > 1),
                    (
                        layout_targeted_traversal,
                        apply_deferred,
                    )
                        .chain()
                        .run_if(|t: Res<DirtyNodeTracker>| t.len() > 0),
                    layout_targeted_traversal_loop.run_if(|t: Res<DirtyNodeTracker>| t.len() > 0),
                )
                    .chain()
                    .in_set(LayoutSetCompute));
    }
}

//-------------------------------------------------------------------------------------------------------------------

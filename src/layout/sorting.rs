//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use smallvec::SmallVec;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct OrderEntry
{
    /// (ZLevel, child index)
    key: (ZLevel, usize),
    entity: Entity,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const ORDER_BUFFER_SIZE: usize = 16;

#[derive(Default, Deref, DerefMut)]
struct OrderBuffer(SmallVec<[OrderEntry; ORDER_BUFFER_SIZE]>);

impl OrderBuffer
{
    fn sort(&mut self)
    {
        // The child index is included, which makes this a fast non-allocating stable sort.
        self.sort_unstable_by_key(|e| e.key);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
struct OrderingCache
{
    heap: Vec<OrderBuffer>,
}

impl OrderingCache
{
    fn pop(&mut self, len: usize) -> OrderBuffer
    {
        if len <= ORDER_BUFFER_SIZE { return OrderBuffer::default(); }
        let mut buffer = self.heap.pop().unwrap_or_default();
        buffer.reserve_exact(len);
        buffer
    }

    fn push(&mut self, mut buffer: OrderBuffer)
    {
        if !buffer.spilled() { return; }
        buffer.clear();
        self.heap.push(buffer);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_z_order_node(
    cache      : &mut OrderingCache,
    node       : Entity,
    depth      : f32,
    transforms : &mut Query<&mut Transform, With<CobwebNode>>,
    nodes      : &Query<&Children, With<CobwebNode>>,
    levels     : &Query<Option<&ZLevel>, With<CobwebNode>>,
) -> usize
{
    // Get the node.
    let Ok(mut transform) = transforms.get_mut(node)
    else { tracing::error!("node {:?} missing when updating z-order, be sure to use despawn_recursive", node); return 0; };

    // Update transform depth without triggering change detection needlessly.
    if transform.translation.z != depth { transform.translation.z = depth; }

    // Handle children.
    let Ok(children) = nodes.get(node) else { return 0; };
    update_z_order_children(0., cache, &children, transforms, nodes, levels)
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_z_order_children(
    offset     : f32,
    cache      : &mut OrderingCache,
    children   : &Children,
    transforms : &mut Query<&mut Transform, With<CobwebNode>>,
    nodes      : &Query<&Children, With<CobwebNode>>,
    levels     : &Query<Option<&ZLevel>, With<CobwebNode>>,
) -> usize
{
    // Collect children and stable-sort by ZLevel so the highest nodes are toward the front.
    // - Stable-sorting ensures newer nodes (i.e. nodes at the end of Children) will be higher than older nodes.
    let num_children = children.len();
    let mut sorted_children = cache.pop(num_children);

    for (idx, child) in children.iter().enumerate()
    {
        // Skip non-CobwebNode children.
        let Ok(level) = levels.get(*child) else { continue; };
        let level = level.copied().unwrap_or_default();
        sorted_children.push(OrderEntry{ key: (level, idx), entity: *child });
    }

    sorted_children.sort();

    // Iterate through children from lowest to highest by depth.
    let mut child_count = 0;
    for child in sorted_children.iter()
    {
        // Update the child.
        child_count += 1;
        child_count += update_z_order_node(
            cache,
            child.entity,
            offset + Z_INCREMENT * (child_count as f32),
            transforms,
            nodes,
            levels
        );
    }

    cache.push(sorted_children);
    child_count
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_z_order(
    mut cache      : Local<OrderingCache>,
    roots          : Query<(&UiRoot, &Children)>,
    mut transforms : Query<&mut Transform, With<CobwebNode>>,
    nodes          : Query<&Children, With<CobwebNode>>,
    levels         : Query<Option<&ZLevel>, With<CobwebNode>>,
){
    for (root, children) in roots.iter()
    {
        update_z_order_children(root.base_z_offset, &mut cache, &children, &mut transforms, &nodes, &levels);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Z-increment used for ordering UI nodes relative to their parents.
///
/// All UI nodes are globally ordered by unique z-values separeted by `Z_INCREMENT`.
/// The lowest node in a tree is positioned at [`UiRoot::base_z_offset`] + `Z_INCREMENT`, the next-lowest is at
/// [`UiRoot::base_z_offset`] + `2 * Z_INCREMENT`, and so on.
///
/// A node's non-node children should order themselves using z values smaller than this, otherwise they will
/// z-fight with other nodes.
pub const Z_INCREMENT: f32 = 0.001;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SortingPlugin;

impl Plugin for SortingPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<ZLevel>()
            .add_systems(PostUpdate, update_z_order.in_set(LayoutSetSort));
    }
}

//-------------------------------------------------------------------------------------------------------------------

//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn get_parent_size_ref(world: &World, parent: Entity) -> SizeRef
{
    // Look up parent entity's node size
    let Some(parent_node_size) = world.get::<React<NodeSize>>(parent)
    else
    {
        tracing::warn!("failed getting SizeRef from parent, parent {:?} is missing NodeSize component", parent);
        return SizeRef::default();
    };

    // Update the target node with the parent's size.
    SizeRef(***parent_node_size)
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI node within a specific parent node.
///
/// The node is set as a child of the parent entity.
///
/// Adds [`SpatialBundle`], [`BaseSizeRef`], [`SizeRef`], [`React<SizeRefSource>`](SizeRefSource), and
/// [`React<NodeSize>`](NodeSize) to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Position`] instruction.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deref, DerefMut)]
pub struct Parent(pub Entity);

impl UiInstruction for Parent
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        // Set this node as a child of the parent.
        rc.commands()
            .entity(node)
            .set_parent(self.0)
            .insert(SpatialBundle::default())
            .insert(BaseSizeRef::default())
            .insert(SizeRef::default());

        // Prep entity.
        rc.insert(node, SizeRefSource::Parent);
        rc.insert(node, NodeSize::default());
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ParentPlugin;

impl Plugin for ParentPlugin
{
    fn build(&self, _app: &mut App) { }
}

//-------------------------------------------------------------------------------------------------------------------

//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// The 2D dimensions of the node as a rectangle on the plane of the node's parent.
///
/// Translate by 1/2 of each dimension to reach the edges of the node parent.
#[derive(ReactComponent, Default, Debug, PartialEq, Copy, Clone, Deref, DerefMut)]
pub struct NodeSize(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

/// The z-offset applied between this node and its parent.
//todo: ZLevel
#[derive(ReactComponent, Debug, PartialEq, Copy, Clone, Deref, DerefMut, Default)]
pub struct NodeOffset(pub f32);

//-------------------------------------------------------------------------------------------------------------------

/// Reference data for use in defining the layout of a node.
#[derive(ReactComponent, Debug, PartialEq, Copy, Clone, Default)]
pub struct SizeRef
{
    /// The parent's node size.
    pub parent_size: NodeSize,
    /// The z-offset this node should use relative to its parent.
    pub offset: NodeOffset,
}

//-------------------------------------------------------------------------------------------------------------------

//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Component for UI root entities.
///
/// Note that root entities are *not* UI nodes. Typically they are cameras, textures, or entities in world space.
#[derive(Component, Debug, Copy, Clone)]
pub struct UiRoot
{
    /// Defines the base z-offset applied between the entity and its node children.
    ///
    /// For example, this is set to a negative value for cameras nodes attached to cameras so UI elements will be in-view of the
    /// camera (see [`DEFAULT_CAMERA_Z_OFFSET`]).
    pub base_z_offset: f32,
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for all UI nodes.
#[derive(Component, Default, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct CobwebNode;

//-------------------------------------------------------------------------------------------------------------------

/// Component that records the 2D dimensions of a node as a rectangle on the plane of the node's parent.
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

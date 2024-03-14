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
    /// camera (see [`DEFAULT_CAMERA_Z_OFFSET`](crate::DEFAULT_CAMERA_Z_OFFSET)).
    pub base_z_offset: f32,
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that controls the z-order of nodes relative to their siblings on the same parent.
///
/// Sibling nodes are sorted by `ZLevel` so higher levels are positioned above lower levels.
/// Within a level, sibling nodes are ordered based on their index in the parent's [`Children`] list so that newer
/// nodes default to sorting above older nodes.
///
/// If one node is sorted above another, then the higher node's children will be sorted above all children of the lower,
/// regardless of `ZLevel`.
#[derive(Component, Reflect, Debug, Default, Copy, Clone, Deref, DerefMut, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ZLevel(pub i32);

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for UI nodes.
#[derive(Component, Default, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct CobwebNode;

//-------------------------------------------------------------------------------------------------------------------

/// Component with the size reference of the root ancestor of a node.
///
/// This is updated in [`LayoutSetCompute`].
#[derive(Component, Default, Debug, PartialEq, Copy, Clone, Deref, DerefMut)]
pub struct RootSizeRef(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

/// Reactive component with the size reference for computing the layout of a node.
///
/// Typically this equals the parent's [`NodeSize`], or is derived from a [`UiCamera2D`].
#[derive(ReactComponent, Default, Debug, PartialEq, Copy, Clone, Deref, DerefMut)]
pub struct SizeRef(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

/// Reactive component that records the 2D dimensions of a node as a rectangle on the plane of the node's parent.
#[derive(ReactComponent, Default, Debug, PartialEq, Copy, Clone, Deref, DerefMut)]
pub struct NodeSize(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

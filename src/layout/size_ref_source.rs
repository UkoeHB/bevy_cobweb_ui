//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Reactive component that controls how [`SizeRefs`](SizeRef) are derived during layout computation.
///
/// This component is designed for use in [`UiInstructions`](UiInstruction), not in user code.
///
/// Includes built-in sources for efficieny. The [`Self::Custom`] variant can be used if designing a custom source.
///
/// If a node does not have a `React<SizeRefSource>` component, it will default to [`SizeRefSource::Parent`] during layout
/// computation.
#[derive(ReactComponent, Debug, Default)]
pub enum SizeRefSource
{
    /// Uses the hierarchical parent's [`NodeSize`].
    ///
    /// Does not store the parent entity directly, which enables easier re-parenting of nodes to other nodes.
    #[default]
    Parent,
    /// Uses the hierarchical parent's [`GlobalTransform`] and [`Camera`].
    ///
    /// Does not store the camera entity directly, which enables easier re-parenting of nodes to different cameras.
    Camera,
    //Texture
    //Entity ?? i.e. any entity with GlobalTransform + maybe a marker component
    //Custom(..callback..)
}

impl SizeRefSource
{
    /// Makes a new custom source.
    //pub fn custom(..callback..) -> Self

    /// Computes a [`SizeRef`] for a specific target node.
    ///
    /// We pass in `world` immutably so you can read any data in the world. If you need to read queries,
    /// store [`QueryState`] for your queries in a resource and add a system to [`LayoutSetPrep`] that updates the state
    /// with [`QueryState::update_archetypes`] (however note that this data may become stale after reactions run within
    /// the layout loop). It is generally simpler and more reliable to access entities directly in the world
    /// with [`World::get`] (although less efficient if you have many query terms to access/check).
    pub fn compute(&self, world: &World, target: Entity) -> SizeRef
    {
        match self
        {
            Self::Parent =>
            {
                let Some(parent) = world.get::<bevy::hierarchy::Parent>(target)
                else
                {
                    tracing::warn!("failed computing SizeRef from parent for {:?}, missing parent", target);
                    return SizeRef::default();
                };
                get_parent_size_ref(world, **parent)
            }
            Self::Camera =>
            {
                let Some(camera) = world.get::<bevy::hierarchy::Parent>(target)
                else
                {
                    tracing::warn!("failed computing SizeRef from camera for {:?}, missing parent", target);
                    return SizeRef::default();
                };
                get_camera_size_ref(world, **camera)
            }
            //Self::Custom(callback) =>
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

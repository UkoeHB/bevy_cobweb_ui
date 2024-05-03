use crate::*;

use bevy::prelude::*;
use bevy::ecs::entity::Entities;
use bevy_cobweb::prelude::*;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn detect_sizeref_source(
    mutation    : MutationEvent<SizeRefSource>,
    entities    : &Entities,
    mut tracker : ResMut<DirtyNodeTracker>
){
    let entity = mutation.read().unwrap();
    if entities.get(entity).is_none() { return; }
    tracker.insert(entity);
}

struct DetectSizeRefSource;
impl WorldReactor for DetectSizeRefSource
{
    type StartingTriggers = MutationTrigger::<SizeRefSource>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(detect_sizeref_source) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Reactive component that controls how [`SizeRefs`](SizeRef) are derived during layout computation.
///
/// This component is designed for use in [`UiInstructions`](UiInstruction), not in user code.
///
/// Mutating `SizeRefSource` on a node will automatically mark it [dirty](DirtyNodeTracker) (but not inserting/removing it).
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
    //BaseSizeRef: use this directly
    //Texture
    //Entity ?? i.e. any entity with GlobalTransform + maybe a marker component
    //Entity + reference frame for SizeRefs ?? e.g. a specific camera instead of the entity itself
    //CustomBase(..callback..)
    //Custom(..callback..)
}

//EntityInCamera{camera, entity, scale_configs}: child of camera (orients toward camera position), position is on
// an entity, SizeRef is camera + distance between camera and entity (but UI camera is orthographic projection and entities
// are perspective projection, so 'player position' is probably tied to perspective-projection camera position and
// UI camera is stationariy... maybe need to allow different cameras for positioning and scale? or use the perspective
// camera as an in-world UI camera ?? this would ensure world UI properly respects occlusion but maybe causes interference
// with rendering tricks?)

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
    ///
    /// We also pass in [`BaseSizeRef`]. If `base_sizeref` is `None` then `target` is a base node and its [`SizeRef`] must
    /// be computed directly (e.g. from a camera).
    pub fn compute(&self, world: &World, base_sizeref: Option<BaseSizeRef>, target: Entity) -> SizeRef
    {
        match self
        {
            Self::Parent =>
            {
                if base_sizeref.is_none()
                {
                    tracing::error!("encountered missing BaseSizeRef in SizeRefSource::Parent for {:?}", target);
                }
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

pub(crate) struct SizeRefSourcePlugin;

impl Plugin for SizeRefSourcePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor_with(DetectSizeRefSource, mutation::<SizeRefSource>());
    }
}

//-------------------------------------------------------------------------------------------------------------------

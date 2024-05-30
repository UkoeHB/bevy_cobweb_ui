use crate::*;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn compute_camera_size_ref(
    camera_entity    : Entity,
    camera_transform : &GlobalTransform,
    camera           : &Camera,
) -> Option<SizeRef>
{
    // Get world coordinates of the camera's viewport.
    let Some(rect) = camera.logical_viewport_rect()
    else { tracing::debug!("camera entity {:?} logical viewport rect unavailable", camera_entity); return None; };

    let topleft = rect.min;
    let bottomleft = Vec2{ x: rect.min.x, y: rect.max.y };
    let bottomright = rect.max;

    let Some(topleft) = camera.viewport_to_world(camera_transform, topleft)
    else { tracing::error!("camera entity {:?} viewport transformation broken", camera_entity); return None; };
    let Some(bottomleft) = camera.viewport_to_world(camera_transform, bottomleft)
    else { tracing::error!("camera entity {:?} viewport transformation broken", camera_entity); return None; };
    let Some(bottomright) = camera.viewport_to_world(camera_transform, bottomright)
    else { tracing::error!("camera entity {:?} viewport transformation broken", camera_entity); return None; };

    // Compute dimensions in worldspace.
    let x = bottomleft.origin.distance(bottomright.origin);
    let y = topleft.origin.distance(bottomleft.origin);

    Some(SizeRef(Vec2{ x, y }))
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Handles camera updates.
/// - If the camera is a [`CobwebNode`], marks the camera dirty.
/// - If the camera is a [`UiRoot`], marks children of the camera dirty.
fn camera_update(
    update      : BroadcastEvent<CameraUpdate>,
    mut tracker : ResMut<DirtyNodeTracker>,
    cameras     : Query<&Children, (With<Camera>, With<UiRoot>)>,
    nodes       : Query<(), With<CobwebNode>>,
){
    // Get the camera entity
    let CameraUpdate(camera_entity) = update.read().unwrap();

    // Check if the camera is a node.
    if nodes.contains(*camera_entity)
    {
        tracker.insert(*camera_entity);
        return;
    }

    // Get the camera state if the camera is a UiRoot.
    let Ok(children) = cameras.get(*camera_entity) else { return; };

    // Mark child nodes dirty.
    for child in children.iter()
    {
        if !nodes.contains(*child) { continue; }
        tracker.insert(*child);
    }
}

struct CameraUpdateReactor;
impl WorldReactor for CameraUpdateReactor
{
    type StartingTriggers = BroadcastTrigger<CameraUpdate>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(camera_update) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn get_camera_size_ref(world: &World, camera: Entity) -> SizeRef
{
    // Look up camera entity
    let Some(transform) = world.get::<GlobalTransform>(camera)
    else
    {
        tracing::warn!("failed getting SizeRef from camera, camera {:?} is missing GlobalTransform component", camera);
        return SizeRef::default();
    };
    let Some(camera_ref) = world.get::<Camera>(camera)
    else
    {
        tracing::warn!("failed getting SizeRef from camera, camera {:?} is missing Camera component", camera);
        return SizeRef::default();
    };

    // Get the camera size reference.
    compute_camera_size_ref(camera, transform, camera_ref).unwrap_or_default()
}

//-------------------------------------------------------------------------------------------------------------------

/// The depth of root-level UI nodes tied to a camera, relative to the camera.
pub const DEFAULT_CAMERA_Z_OFFSET: f32 = -100.0;

//-------------------------------------------------------------------------------------------------------------------

/// Bundle for setting up a camera as a UI root node.
///
/// This can be added to existing camera entities.
#[derive(Bundle)]
pub struct UiCameraRootBundle
{
    pub vis  : InheritedVisibility,
    pub root : UiRoot,
}

impl Default for UiCameraRootBundle
{
    fn default() -> Self
    {
        Self{
            vis  : InheritedVisibility::VISIBLE,
            root : UiRoot{ base_z_offset: DEFAULT_CAMERA_Z_OFFSET }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Bundle for creating a 2D camera as a UI root node.
///
/// This creates a completely new camera. Use [`UiCameraRootBundle`] if you already have a camera.
#[derive(Bundle)]
pub struct UiCamera2DBundle
{
    pub camera : Camera2dBundle,
    pub root   : UiCameraRootBundle,
}

impl Default for UiCamera2DBundle
{
    fn default() -> Self
    {
        Self{
            camera: Camera2dBundle{
                transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() },
                ..default()
            },
            root: UiCameraRootBundle::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI root node within a specific camera's viewport.
///
/// Adds [`SpatialBundle`], [`BaseSizeRef`], [`SizeRef`], [`React<SizeRefSource>`](SizeRefSource), and
/// [`React<NodeSize>`](NodeSize) to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Position`] instruction.
///
/// [`BaseSizeRef`] for nodes on cameras will typically equal [`SizeRef`] unless the camera is also a node.
///
/// This currently only works for 2D UI cameras. See [`UiCamera2DBundle`] and [`UiCameraRootBundle`] for setting up a camera.
//todo: support 3D cameras ???
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deref, DerefMut)]
pub struct InCamera(pub Entity);

impl UiInstruction for InCamera
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        let camera_entity = self.0;

        // Set this node as a child of the camera.
        rc.commands()
            .entity(node)
            .set_parent(camera_entity)
            .insert(SpatialBundle::default())
            .insert(BaseSizeRef::default())
            .insert(SizeRef::default());

        // Prep entity.
        rc.insert(node, SizeRefSource::Camera);
        rc.insert(node, NodeSize::default());
        //todo: validate that camera entity contains UiRoot component
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct Camera2DPlugin;

impl Plugin for Camera2DPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor_with(CameraUpdateReactor, broadcast::<CameraUpdate>());
    }
}

//-------------------------------------------------------------------------------------------------------------------

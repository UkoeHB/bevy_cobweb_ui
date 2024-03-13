//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


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
    else { tracing::debug!(?camera_entity, "camera entity logical viewport rect unavailable"); return None; };

    let topleft = rect.min;
    let bottomleft = Vec2{ x: rect.min.x, y: rect.max.y };
    let bottomright = rect.max;

    let Some(topleft) = camera.viewport_to_world(camera_transform, topleft)
    else { tracing::error!(?camera_entity, "camera entity viewport transformation broken"); return None; };
    let Some(bottomleft) = camera.viewport_to_world(camera_transform, bottomleft)
    else { tracing::error!(?camera_entity, "camera entity viewport transformation broken"); return None; };
    let Some(bottomright) = camera.viewport_to_world(camera_transform, bottomright)
    else { tracing::error!(?camera_entity, "camera entity viewport transformation broken"); return None; };

    // Compute dimensions in worldspace.
    let x = bottomleft.origin.distance(bottomright.origin);
    let y = topleft.origin.distance(bottomleft.origin);

    Some(SizeRef(Vec2{ x, y }))
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the layout ref of children of an updated camera.
fn camera_update_reactor(
    update   : BroadcastEvent<CameraUpdate>,
    mut rc   : ReactCommands,
    cameras  : Query<(&Children, &GlobalTransform, &Camera)>,
    mut refs : Query<&mut React<SizeRef>>
){
    // Get the camera entity
    let Some(CameraUpdate(camera_entity)) = update.read()
    else { tracing::error!("failed updating layout ref of in-camera node, event is missing"); return; };

    // Get the camera state.
    let Ok((children, camera_transform, camera)) = cameras.get(*camera_entity)
    else
    {
        tracing::debug!(?camera_entity, "failed updating layout ref of in-camera nodes, camera entity missing");
        return;
    };

    // Get camera layout info.
    let Some(parent_ref) = compute_camera_size_ref(*camera_entity, camera_transform, camera) else { return; };

    // Update children.
    for child in children.iter()
    {
        let Ok(mut layout_ref) = refs.get_mut(*child) else { continue; };
        layout_ref.set_if_not_eq(&mut rc, parent_ref);
    }
}

struct CameraUpdateReactor;
impl WorldReactor for CameraUpdateReactor
{
    type StartingTriggers = BroadcastTrigger<CameraUpdate>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(camera_update_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Refreshes the layout ref of a child of a camera.
fn camera_refresh_reactor(
    finish    : EntityEvent<FinishNode>,
    mut rc    : ReactCommands,
    cameras   : Query<(&GlobalTransform, &Camera)>,
    mut nodes : Query<(&bevy::hierarchy::Parent, &mut React<SizeRef>)>,
){
    // Get the target node.
    let Some((target_node, _)) = finish.read()
    else { tracing::error!("failed updating layout ref of in-camera node, node event missing"); return; };

    // Get the camera entity.
    let Ok((camera_entity, mut size_ref)) = nodes.get_mut(*target_node)
    else
    {
        tracing::debug!(?target_node, "failed updating layout ref of in-camera node, target node has no camera parent");
        return;
    };

    // Get the camera.
    let Ok((camera_transform, camera)) = cameras.get(**camera_entity)
    else
    {
        tracing::debug!(?camera_entity, "failed updating layout ref of in-camera node, camera entity missing");
        return;
    };

    // Get camera layout info.
    let Some(parent_ref) = compute_camera_size_ref(**camera_entity, camera_transform, camera) else { return; };

    // Update the target node.
    // - Note: Since we are refreshing, we don't use set_if_not_eq().
    *size_ref.get_mut(&mut rc) = parent_ref;
}

struct CameraRefreshReactor;
impl WorldReactor for CameraRefreshReactor
{
    type StartingTriggers = ();
    type Triggers = EntityEventTrigger<FinishNode>;
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(camera_refresh_reactor) }
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
pub struct UiCameraRoot
{
    pub vis: InheritedVisibility,
    pub root: UiRoot,
}

impl Default for UiCameraRoot
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
/// This creates a completely new camera. Use [`UiCameraRoot`] if you already have a camera.
#[derive(Bundle)]
pub struct UiCamera2D
{
    pub camera: Camera2dBundle,
    pub root: UiCameraRoot,
}

impl Default for UiCamera2D
{
    fn default() -> Self
    {
        Self{
            camera: Camera2dBundle{
                transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() },
                ..default()
            },
            root: UiCameraRoot::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI root node within a specific camera's viewport.
///
/// Adds a default [`SpatialBundle`], [`React<NodeSize>`](NodeSize), and [`React<SizeRef>`](SizeRef) to the node.
/// Also adds a [`React<SizeRefSource::Camera>`](SizeRefSource::Camera) to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Position`] instruction.
///
/// This currently only works for 2D UI cameras. See [`UiCamera2D`] and [`UiCameraRoot`] for setting up a camera.
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
            .insert(SpatialBundle::default());

        // Prep entity.
        rc.insert(node, NodeSize::default());
        rc.insert(node, SizeRef::default());
        rc.insert(node, SizeRefSource::Camera);

        // Refresh the node's layout ref on node finish.
        rc.commands().syscall(node,
            |
                In(node)    : In<Entity>,
                mut rc      : ReactCommands,
                mut update  : Reactor<ParentUpdateReactor>,
                mut refresh : Reactor<CameraRefreshReactor>
            |
            {
                update.add_triggers(&mut rc, entity_mutation::<NodeSize>(node));
                refresh.add_triggers(&mut rc, entity_event::<FinishNode>(node));
            }
        );

        //todo: validate that camera entity contains UiRoot component
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct Camera2DPlugin;

impl Plugin for Camera2DPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor_with(CameraUpdateReactor, broadcast::<CameraUpdate>())
            .add_reactor(CameraRefreshReactor);
    }
}

//-------------------------------------------------------------------------------------------------------------------

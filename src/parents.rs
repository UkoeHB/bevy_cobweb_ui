//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn get_camera_layout_ref(
    camera_entity    : Entity,
    camera_transform : &GlobalTransform,
    camera           : &Camera,
) -> Option<LayoutRef>
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

    // Update the layout reference of the targets.
    let parent_size = NodeSize(Vec2{ x, y });
    let offset = NodeOffset(DEFAULT_CAMERA_Z_OFFSET);

    Some(LayoutRef{ parent_size, offset })
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the layout ref of children of an updated camera.
fn camera_update_reactor(
    update   : BroadcastEvent<CameraUpdate>,
    mut rc   : ReactCommands,
    cameras  : Query<(&Children, &GlobalTransform, &Camera)>,
    mut refs : Query<&mut React<LayoutRef>>
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
    let Some(parent_ref) = get_camera_layout_ref(*camera_entity, camera_transform, camera) else { return; };

    // Update children.
    for child in children.iter()
    {
        let Ok(mut layout_ref) = refs.get_mut(*child) else { continue; };
        *layout_ref.get_mut(&mut rc) = parent_ref;
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
    mut nodes : Query<(&bevy::hierarchy::Parent, &mut React<LayoutRef>)>,
){
    // Get the target node.
    let Some((target_node, _)) = finish.read()
    else { tracing::error!("failed updating layout ref of in-camera node, node event missing"); return; };

    // Get the camera entity.
    let Ok((camera_entity, mut layout_ref)) = nodes.get_mut(*target_node)
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
    let Some(parent_ref) = get_camera_layout_ref(**camera_entity, camera_transform, camera) else { return; };

    // Update the target node.
    *layout_ref.get_mut(&mut rc) = parent_ref;
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

/// Updates the layout ref of children of a node when the node's size changes.
///
/// Does nothing if a node has no children.
fn parent_update_reactor(
    parent_size : MutationEvent<NodeSize>,
    mut rc      : ReactCommands,
    sizes       : Query<(&Children, &React<NodeSize>)>,
    mut nodes   : Query<&mut React<LayoutRef>>,
){
    let Some(node) = parent_size.read()
    else { tracing::error!("failed updating children layout refs, event is missing"); return; };
    let Ok((children, node_size)) = sizes.get(node) else { return; };

    // Update the children with the parent's size.
    let parent_size = **node_size;
    let offset = NodeOffset(DEFAULT_Z_OFFSET);
    let parent_ref = LayoutRef{ parent_size, offset };

    for child in children.iter()
    {
        let Ok(mut layout_ref) = nodes.get_mut(*child) else { continue; };
        *layout_ref.get_mut(&mut rc) = parent_ref;
    }
}

struct ParentUpdateReactor;
impl WorldReactor for ParentUpdateReactor
{
    type StartingTriggers = ();
    type Triggers = EntityMutationTrigger<NodeSize>;
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(parent_update_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Refreshes the layout ref of a node from its parent.
fn parent_refresh_reactor(
    finish    : EntityEvent<FinishNode>,
    mut rc    : ReactCommands,
    mut nodes : Query<(&bevy::hierarchy::Parent, &mut React<LayoutRef>)>,
    sizes     : Query<&React<NodeSize>>,
){
    let Some((node, _)) = finish.read()
    else { tracing::error!("failed updating parent layout ref, event is missing"); return; };
    let Ok((parent, mut layout_ref)) = nodes.get_mut(*node)
    else { tracing::debug!(?node, "failed updating parent layout ref, node is missing"); return; };
    let Ok(parent_size) = sizes.get(**parent)
    else { tracing::debug!(?node, "failed updating parent layout ref, parent node not found"); return; };

    // Update the target node with the parent's size.
    let parent_size = **parent_size;
    let offset = NodeOffset(DEFAULT_Z_OFFSET);
    let parent_ref = LayoutRef{ parent_size, offset };
    *layout_ref.get_mut(&mut rc) = parent_ref;
}

struct ParentRefreshReactor;
impl WorldReactor for ParentRefreshReactor
{
    type StartingTriggers = ();
    type Triggers = EntityEventTrigger<FinishNode>;
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(parent_refresh_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// The depth of root-level UI nodes tied to a camera, relative to the camera.
pub const DEFAULT_CAMERA_Z_OFFSET: f32 = -100.0;

/// The default vertical offset between a child and parent node.
pub const DEFAULT_Z_OFFSET: f32 = 10.0f32;

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI root node within a specific camera's viewport.
///
/// Adds `SpatialBundle`, `React<`[`NodeSize`]`>`, and `React<`[`LayoutRef`]`>` to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Layout`] instruction.
///
/// This currently only works for 2D cameras.
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
        rc.insert(node, LayoutRef::default());

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
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI node within a specific parent node.
///
/// Adds `SpatialBundle`, `React<`[`NodeSize`]`>`, and `React<`[`LayoutRef`]`>` to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Layout`] instruction.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deref, DerefMut)]
pub struct Parent(pub Entity);

impl UiInstruction for Parent
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        let parent_entity = self.0;

        // Set this node as a child of the parent.
        rc.commands()
            .entity(node)
            .set_parent(parent_entity)
            .insert(SpatialBundle::default());

        // Prep entity.
        rc.insert(node, NodeSize::default());
        rc.insert(node, LayoutRef::default());

        // Refresh the node's layout ref on node finish, and refresh children layouts on update.
        rc.commands().syscall(node,
            |
                In(node)    : In<Entity>,
                mut rc      : ReactCommands,
                mut update  : Reactor<ParentUpdateReactor>,
                mut refresh : Reactor<ParentRefreshReactor>
            |
            {
                update.add_triggers(&mut rc, entity_mutation::<NodeSize>(node));
                refresh.add_triggers(&mut rc, entity_event::<FinishNode>(node));
            }
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ParentsPlugin;

impl Plugin for ParentsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor_with(CameraUpdateReactor, broadcast::<CameraUpdate>())
            .add_reactor(CameraRefreshReactor)
            .add_reactor(ParentUpdateReactor)
            .add_reactor(ParentRefreshReactor);
    }
}

//-------------------------------------------------------------------------------------------------------------------

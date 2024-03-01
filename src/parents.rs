//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// The depth of root-level UI nodes tied to a camera, relative to the camera.
pub const DEFAULT_CAMERA_Z_OFFSET: f32 = -100.0;

/// The default vertical offset between a child and parent node.
pub const DEFAULT_Z_OFFSET: f32 = 10.0f32;

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI root node within a specific camera's viewport.
///
/// Adds `SpatialBundle` and a `React<`[`NodeSize`]`>` component to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Layout`] instruction.
///
/// This currently only works for 2D cameras.
//todo: support 3D cameras ???
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deref, DerefMut)]
pub struct InCamera(pub Entity);

impl UiInstruction for InCamera
{
    fn apply(self, rc: &mut ReactCommands, node: Entity, finishers: &mut UiInstructionFinishers)
    {
        let camera_entity = self.0;

        // Set this node as a child of the camera.
        rc.commands()
            .entity(node)
            .set_parent(camera_entity)
            .insert(SpatialBundle::default());

        // Prep entity.
        rc.insert(node, NodeSize::default());

        // Prepare system to propagate camera mutations.
        let sys_command = rc.commands().spawn_system_command(
            move
            |
                mut rc  : ReactCommands,
                cameras : Query<(&GlobalTransform, &Camera)>
            |
            {
                let Ok((camera_transform, camera)) = cameras.get(camera_entity)
                else { tracing::debug!(?camera_entity, "camera entity missing"); return; };

                // Get world coordinates of the camera's viewport.
                let Some(rect) = camera.logical_viewport_rect()
                else { tracing::debug!(?camera_entity, "camera entity logical viewport rect unavailable"); return; };

                let topleft = rect.min;
                let bottomleft = Vec2{ x: rect.min.x, y: rect.max.y };
                let bottomright = rect.max;

                let Some(topleft) = camera.viewport_to_world(camera_transform, topleft)
                else { tracing::error!(?camera_entity, "camera entity viewport transformation broken"); return; };
                let Some(bottomleft) = camera.viewport_to_world(camera_transform, bottomleft)
                else { tracing::error!(?camera_entity, "camera entity viewport transformation broken"); return; };
                let Some(bottomright) = camera.viewport_to_world(camera_transform, bottomright)
                else { tracing::error!(?camera_entity, "camera entity viewport transformation broken"); return; };

                // Compute dimensions in worldspace.
                let x = bottomleft.origin.distance(bottomright.origin);
                let y = topleft.origin.distance(bottomleft.origin);

                // Update this node with the parent's size.
                let size = NodeSize(Vec2{ x, y });
                let child_offset = NodeOffset(DEFAULT_CAMERA_Z_OFFSET);
                rc.entity_event(node, ParentUpdate{ size, child_offset });
            }
        );
        let token = rc.with(entity_event::<CameraUpdate>(camera_entity), sys_command, ReactorMode::Revokable).unwrap();
        cleanup_reactor_on_despawn(rc, node, token);

        // Pull in the camera's current state.
        finishers.push(sys_command);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI node within a specific parent node.
///
/// Adds `SpatialBundle` and a `React<`[`NodeSize`]`>` component to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Layout`] instruction.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deref, DerefMut)]
pub struct Parent(pub Entity);

impl UiInstruction for Parent
{
    fn apply(self, rc: &mut ReactCommands, node: Entity, finishers: &mut UiInstructionFinishers)
    {
        let parent_entity = self.0;

        // Set this node as a child of the parent.
        rc.commands()
            .entity(node)
            .set_parent(parent_entity)
            .insert(SpatialBundle::default());

        // Prep entity.
        rc.insert(node, NodeSize::default());

        let token = rc.on_revokable(entity_mutation::<NodeSize>(parent_entity),
            move
            |
                mut rc : ReactCommands,
                nodes  : Query<&React<NodeSize>>
            |
            {
                let Ok(parent) = nodes.get(parent_entity)
                else { tracing::debug!(?parent_entity, "parent node missing"); return; };

                // Update this node with the parent's size.
                let size = **parent;
                let child_offset = NodeOffset(DEFAULT_Z_OFFSET);
                rc.entity_event(node, ParentUpdate{ size, child_offset });
            }
        );
        cleanup_reactor_on_despawn(rc, node, token.clone());

        // Pull in the parent's node size.
        finishers.push(token.into());
    }
}

//-------------------------------------------------------------------------------------------------------------------

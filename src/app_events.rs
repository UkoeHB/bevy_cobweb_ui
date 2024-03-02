//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::render::camera::CameraUpdateSystem;
use bevy::transform::TransformSystem::TransformPropagate;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_camera_update(
    cameras : Query<Entity, Or<(Changed<Camera>, Changed<Transform>)>>,
    mut rc  : ReactCommands
){
    for camera_entity in cameras.iter()
    {
        rc.broadcast(CameraUpdate(camera_entity));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn handle_keyboard_inputs(mut inputs: EventReader<KeyboardInput>, mut rc: ReactCommands)
{
    for input in inputs.read()
    {
        rc.broadcast(input.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Event broadcasted whenever a camera's `Camera` or `Transform` changes.
#[derive(Debug, Copy, Clone, Deref, DerefMut)]
pub struct CameraUpdate(pub Entity);

//-------------------------------------------------------------------------------------------------------------------

/// Adds systems that emit events that UI nodes might react to.
///
/// Emits:
/// - Broadcast event for `KeyboardInput`. Runs in `First`.`
///   You can use the [`On`] instruction to listen for this: `On::<KeyboardInput>::new(my_callback)`.
/// - Broadcast event for [`CameraUpdate`]. Runs in `PostUpdate`.
pub struct AppEventsPlugin;

impl Plugin for AppEventsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(First,
                (
                    handle_keyboard_inputs,
                )
                    .chain()
            )
            .add_systems(PostUpdate,
                (
                    handle_camera_update
                )
                    .before(CameraUpdateSystem)
                    .after(TransformPropagate)
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

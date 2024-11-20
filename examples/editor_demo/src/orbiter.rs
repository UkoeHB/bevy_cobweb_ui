use std::f32::consts::TAU;
use std::time::Duration;

use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::ApplyFluxChanges;

//-------------------------------------------------------------------------------------------------------------------

const MAX_RADIUS: f32 = 200.0;
const MIN_RADIUS: f32 = 0.0;
const MAX_VELOCITY: f32 = 25.0;
const MIN_VELOCITY: f32 = -MAX_VELOCITY;

//-------------------------------------------------------------------------------------------------------------------

fn update_oribiters(time: Res<Time>, mut orbiters: Query<(&mut Transform, &mut Orbit, &Orbiter)>)
{
    let delta = time.delta();

    for (mut transform, mut orbit, orbiter) in orbiters.iter_mut() {
        *transform = orbit.update(delta, *orbiter);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component with metadata for an [`Orbiter`].
#[derive(Component)]
#[require(Transform)]
pub struct Orbit
{
    /// Point being orbited around as 2d translation.
    pub origin: Vec2,
    /// Current radial position in radians.
    pub radial_position: f32,
}

impl Orbit
{
    /// Sets the initial oribited point and radial position of the orbiter.
    pub fn new(origin: Vec2, radial_position: f32) -> Self
    {
        Self { origin, radial_position }
    }

    /// Moves an orbiter around the origin and computes a transform for it.
    ///
    /// Updates `self.radial_position` based on the angular distance traveled.
    pub fn update(&mut self, delta: Duration, mut orbiter: Orbiter) -> Transform
    {
        // Repair the orbiter just in case.
        orbiter.normalize();

        // Rotate the orbiter.
        self.radial_position += orbiter.velocity * delta.as_secs_f32();

        // Wrap around.
        while self.radial_position > TAU {
            self.radial_position -= TAU;
        }
        while self.radial_position < 0.0 {
            self.radial_position += TAU;
        }

        // Compute transform for orbiter.
        let mut start_pos = self.origin;
        start_pos.x += orbiter.radius;
        let mut transform = Transform::from_translation(start_pos.extend(0.0));
        transform.rotate_around(self.origin.extend(0.0), Quat::from_rotation_z(self.radial_position));

        transform
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component with config for an entity that orbits a fixed point. See [`Orbit`].
#[derive(Component, Copy, Clone, Reflect, Default, PartialEq)]
pub struct Orbiter
{
    /// Distance from the origin of the orbiter.
    #[reflect(@MIN_RADIUS..=MAX_RADIUS)]
    pub radius: f32,
    /// Angular velocity in radians per second.
    #[reflect(@MIN_VELOCITY..=MAX_VELOCITY)]
    pub velocity: f32,
}

impl Orbiter
{
    /// Clamps inner values to allowed ranges.
    pub fn normalize(&mut self)
    {
        self.radius = self.radius.clamp(MIN_RADIUS, MAX_RADIUS);
        self.velocity = self.velocity.clamp(MIN_VELOCITY, MAX_VELOCITY);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct DemoOrbiterPlugin;

impl Plugin for DemoOrbiterPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_bundle_type::<Orbiter>()
            // TODO: remove ApplyFluxChanges after flux systems reorganized
            .add_systems(Update, update_oribiters.after(ApplyFluxChanges));
    }
}

//-------------------------------------------------------------------------------------------------------------------

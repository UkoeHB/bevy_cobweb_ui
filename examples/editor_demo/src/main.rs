#[cfg(feature = "editor")]
pub mod editor_ext;
pub mod orbiter;
pub mod rng;

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;
use rand::Rng;

//-------------------------------------------------------------------------------------------------------------------

const SCREEN_HALF_WIDTH: f32 = 400.0;
const SCREEN_HALF_HEIGHT: f32 = 300.0;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_scene_simples(
    mut c: Commands,
    mut s: SceneBuilder,
    mut rng: ResMut<rng::DemoRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
)
{
    c.spawn(Camera2d);

    let rng = rng.rng();
    let shape = meshes.add(Circle::new(50.0));
    let color = materials.add(Color::from(bevy::color::palettes::tailwind::ORANGE_600));

    for _ in 0..20 {
        c.spawn_scene(("main.cob", "orbit"), &mut s, |h| {
            // Random starting location and angle.
            let start_x = rng.gen_range(-SCREEN_HALF_WIDTH..=SCREEN_HALF_WIDTH);
            let start_y = rng.gen_range(-SCREEN_HALF_HEIGHT..=SCREEN_HALF_HEIGHT);
            let start_radial = rng.gen_range((0.)..TAU);

            h.insert((
                Mesh2d(shape.clone()),
                MeshMaterial2d(color.clone()),
                orbiter::Orbit::new(Vec2::new(start_x, start_y), start_radial),
            ));
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    let mut app = App::new();

    app.add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            window_theme: Some(bevy::window::WindowTheme::Dark),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(CobwebUiPlugin)
    .add_plugins(orbiter::DemoOrbiterPlugin)
    .insert_resource(rng::DemoRng::new(0))
    .load("main.cob")
    .add_systems(OnEnter(LoadState::Done), spawn_scene_simples);

    #[cfg(feature = "editor")]
    app.add_plugins(editor_ext::DemoEditorExtPlugin);

    app.run();
}

//-------------------------------------------------------------------------------------------------------------------

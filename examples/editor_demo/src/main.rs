#[cfg(feature = "editor")]
mod editor_ext;
mod orbiter;

use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_scenes(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    c.spawn(Camera2d);
    c.load_scene(("main.cob", "orbit"), &mut s);

    // TODO: spawn a bunch of orbiting circles, with radius and velocity configured by COB scene data and editable
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
    .load("main.cob")
    .add_systems(OnEnter(LoadState::Done), build_scenes);

    #[cfg(feature = "editor")]
    app.add_plugins(editor_ext::DemoEditorExtPlugin);

    app.run();
}

//-------------------------------------------------------------------------------------------------------------------

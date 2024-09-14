//! Demonstrates using `PropagateOpacity` to show/hide help text. Also demonstrates opacity layering.
//!
//! Note that this is a fairly unsophisticated help text. It's inserted in-line to the node tree, whereas a mature
//! widget would have precise positioning and reusable styling, would be able to adjust its position based
//! on available screen-space, may adjust its position in response to detected cursor size (when possible),
//! and may have a little arrow pointing from the text to the source.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = SceneRef::new("main.caf.json", "scene");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |_| {});
}

//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    commands.spawn(Camera2dBundle {
        transform: Transform { translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() },
        ..default()
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..default() }),
            ..default()
        }))
        .add_plugins(SickleUiPlugin)
        .add_plugins(ReactPlugin)
        .add_plugins(CobwebUiPlugin)
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

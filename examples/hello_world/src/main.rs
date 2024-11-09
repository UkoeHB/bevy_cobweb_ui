//! A trivial hello world using a cobweb asset file.
//!
//! You can experiment with hot reloading by running the app and modifying the `assets/main.cob` file.
//! Hot-reloading is enabled by default in examples.

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle_ext::ui_builder::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, PartialEq, Default)]
struct TestCommand(String);

impl Command for TestCommand
{
    fn apply(self, _: &mut World)
    {
        println!("TEST: {:?}", self.0.as_str());
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = SceneRef::new("main.cob", "scene");
    c.ui_root().load_scene(&mut s, scene);
}

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..default() }),
            ..default()
        }))
        .add_plugins(CobwebUiPlugin)
        .register_command_type::<TestCommand>()
        .load("main.cob")
        .add_systems(PreStartup, |mut c: Commands| {
            c.spawn(Camera2d);
        })
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

//! A trivial hello world using a cobweb asset file.
//!
//! You can experiment with hot reloading by running the app and modifying the `assets/main.caf.json` file.
//! Hot-reloading is enabled by default in examples.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::ui_builder::*;


#[derive(Reflect, Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct GenericTest<A, B, C>
{
    a: u32,
    #[reflect(ignore)]
    _p: std::marker::PhantomData<(A, B, C)>
}

impl<A, B, C> bevy::ecs::world::Command for GenericTest<A, B, C>
where
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
    C: Send + Sync + 'static
{ fn apply(self, _: &mut World) {
    tracing::warn!("generic success");
}}

#[derive(Reflect, Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct DeeperTest(Vec<u32>);

#[derive(Reflect, Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
enum InnerTest
{
    #[default]
    A,
    B(DeeperTest)
}

#[derive(Reflect, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct Test(u32, f32, InnerTest, GenericTest<u32, DeeperTest, (f32, f32)>, std::collections::HashMap<u32, u32>);

impl Default for Test
{
    fn default() -> Self {
        let mut map = std::collections::HashMap::default();
        map.insert(0, 0);
        Self(0, 0.0, InnerTest::A, GenericTest{a: 0, _p: std::marker::PhantomData}, map)
    }
}

#[derive(Reflect, Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct BasicTest(u32);

impl bevy::ecs::world::Command for BasicTest { fn apply(self, _: &mut World) {
    tracing::warn!("success");
}}
impl bevy::ecs::world::Command for Test { fn apply(self, _: &mut World) {
    tracing::warn!("success");
}}

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
let raw = b"h \n";
let string = String::from_utf8_lossy(raw);
let raw2 = string.as_bytes();
println!("{:?}", raw);
println!("xx\n");
println!("{:?}", string);
println!("xx\n");
println!("{:?}", raw2);


    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..default() }),
            ..default()
        }))
        .add_plugins(CobwebUiPlugin)
        .register_command::<BasicTest>()
        .register_command::<Test>()
        .register_command::<GenericTest<GenericTest<u32, u32, (u32, u32)>, DeeperTest, (f32, f32)>>()
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

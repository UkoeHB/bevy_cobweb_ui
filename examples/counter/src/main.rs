//! Demonstrates building a simple counter as a small reactive scene.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactComponent, Deref, Default, Debug, Clone)]
struct Counter(usize);

impl Counter
{
    fn increment(&mut self)
    {
        self.0 += 1;
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: SceneBuilder)
{
    let scene = ("main.cob", "root");
    c.ui_root().spawn_scene_and_edit(scene, &mut s, |h| {
        h.edit("button", |h| {
            let button_id = h.id();
            h.insert_reactive(Counter(0)).on_pressed(
                move |mut c: Commands, mut counters: ReactiveMut<Counter>| {
                    counters
                        .get_mut(&mut c, button_id)
                        .map(Counter::increment)?;
                    OK
                },
            );

            h.get("text").update_on(
                entity_mutation::<Counter>(button_id),
                move |text_id: UpdateId, mut e: TextEditor, counters: Reactive<Counter>| {
                    let counter = counters.get(button_id)?;
                    write_text!(e, *text_id, "Counter: {}", **counter);
                    OK
                },
            );
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    commands.spawn(Camera2d);
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
        .load("main.cob")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

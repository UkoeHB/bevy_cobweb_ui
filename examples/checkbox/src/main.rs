//! Example demonstrating the checkbox widget.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: SceneBuilder)
{
    let scene = ("main.cob", "scene");
    c.ui_root().spawn_scene(scene, &mut s, |h| {
        h.edit("basic", |h| {
            let text_id = h.get_entity("text").unwrap();

            h.edit("checkbox", |h| {
                let id = h.id();
                // Use update_on so it also initializes the text.
                h.update_on(entity_event::<Uncheck>(id), move |_: TargetId, mut e: TextEditor| {
                    write_text!(e, text_id, "Unchecked");
                })
                .on_check(move |mut e: TextEditor| {
                    write_text!(e, text_id, "Checked");
                });
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn setup(mut c: Commands)
{
    c.spawn(Camera2d);
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

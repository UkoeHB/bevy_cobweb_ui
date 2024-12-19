//! Example demonstrating the checkbox widget.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = ("main.cob", "scene");
    c.ui_root().load_scene_and_edit(scene, &mut s, |l| {
        l.edit("basic", |l| {
            let text_id = l.get_entity("text").unwrap();

            l.edit("checkbox", |l| {
                let id = l.id();
                // Use update_on so it also initializes the text.
                l.update_on(entity_event::<Uncheck>(id), move |_: UpdateId, mut e: TextEditor| {
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

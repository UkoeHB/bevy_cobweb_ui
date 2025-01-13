//! A simple radio button example using the built-in RadioButton widget.
//!
//! Note that in this example we build the button scenes from scratch in `assets/main.cob`. Other examples may use
//! built-in default styling for radio buttons.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb_ui::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: SceneBuilder)
{
    static OPTIONS: [&'static str; 3] = ["A", "B", "C"];

    let scene = ("main.cob", "scene");
    c.ui_root().spawn_scene_and_edit(scene, &mut s, |h| {
        // Get the display text's entity.
        let display_text = h.get("display::text")?.id();

        // Insert radio buttons.
        h.edit("radio_frame", |h| {
            for (i, option) in OPTIONS.iter().enumerate() {
                h.spawn_scene_and_edit(("main.cob", "button"), |h| {
                    // Set display text when a button is selected.
                    h.on_select(move |mut e: TextEditor| {
                        write_text!(e, display_text, "Selected: {}", option);
                    });

                    h.get("text")?.update_text(format!("{}", option));
                    // Select the first option.
                    if i == 0 {
                        let entity = h.id();
                        h.react().entity_event(entity, Select);
                    }

                    OK
                });
            }
        });

        OK
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

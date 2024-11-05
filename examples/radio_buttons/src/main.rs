//! A simple radio button example using the built-in RadioButton widget.
//!
//! Note that in this example we override the default widget spec with some spacing adjustments. Check out
//! the `radio_button` scene in `assets/main.caf.json`.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb_ui::builtin::widgets::radio_button::{RadioButtonBuilder, RadioButtonManager};
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle_ext::ui_builder::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let file = &SceneFile::new("examples.radio_buttons");
    let scene = file + "scene";
    static OPTIONS: [&'static str; 3] = ["A", "B", "C"];

    c.ui_root().load_scene_and_edit(&mut s, scene, |l| {
        // Get the display text's entity.
        let mut display_text = Entity::PLACEHOLDER;
        l.edit("display::text", |l| {
            display_text = l.id();
        });

        // Insert radio buttons.
        l.edit("radio_frame", |l| {
            let n = l.deref_mut();
            let manager_entity = RadioButtonManager::insert(n);

            for (i, option) in OPTIONS.iter().enumerate() {
                // Add radio button.
                let entity = RadioButtonBuilder::custom_with_text(file + "radio_button", *option)
                    .with_indicator()
                    .build(manager_entity, n)
                    .on_select(move |mut e: TextEditor| {
                        write_text!(e, display_text, "Selected: {}", option);
                    })
                    .id();

                // Select the first option.
                if i == 0 {
                    n.react().entity_event(entity, Select);
                }
            }
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn setup(mut c: Commands)
{
    c.spawn(Camera2dBundle {
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
        .add_plugins(CobwebUiPlugin)
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

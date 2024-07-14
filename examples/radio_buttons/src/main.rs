//! A simple radio button example using the built-in RadioButton widget.
//!
//! Note that in this example we override the default widget spec with some spacing adjustments. Check out
//! the `radio_button` scene in `assets/main.caf.json`.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use bevy_cobweb_ui::widgets::radio_button::{RadioButtonBuilder, RadioButtonManager};

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let file = LoadableRef::from_file("examples.radio_buttons");
    let scene = file.e("scene");
    static OPTIONS: [&'static str; 3] = ["A", "B", "C"];

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
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
                let entity = RadioButtonBuilder::custom_with_text(file.e("radio_button"), *option)
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
        .add_plugins(SickleUiPlugin)
        .add_plugins(ReactPlugin)
        .add_plugins(CobwebUiPlugin)
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

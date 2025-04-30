//! Demonstrates building a calculator using grid layout and scene macros.

use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use calc::Context;
use itertools::Itertools;
use rust_decimal::prelude::{Decimal, FromPrimitive};

//-------------------------------------------------------------------------------------------------------------------

const BACKDROP_COLOR: Srgba = SLATE_950;

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactComponent, Default)]
struct Calculator
{
    buffer: String,
}

impl Calculator
{
    fn add_instruction(&mut self, instruction: &str)
    {
        match instruction {
            "C" => {
                self.buffer = "".to_string();
            }
            "=" => {
                let Ok(result) = Context::<f64>::default().evaluate(&self.buffer) else {
                    self.buffer = "error".to_string();
                    return;
                };
                if let Some(result) = Decimal::from_f64((result * 100.).round() / 100.) {
                    self.buffer = result.normalize().to_string();
                } else {
                    self.buffer = result.to_string();
                }
            }
            x => {
                self.buffer.push_str(x);
            }
        }
    }

    fn buffer_display(&self) -> String
    {
        self.buffer.chars().tail(11).collect::<String>()
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: SceneBuilder)
{
    let buttons = vec![
        "C", "", "7", "8", "9", "/", "4", "5", "6", "*",
        "1", "2", "3", "-", "0", ".", "=", "+",
    ];

    let scene = ("main.cob", "scene");
    c.ui_root().spawn_scene(scene, &mut s, |h| {
        h.insert_reactive(Calculator::default());
        let calc_entity = h.id();

        for button in buttons {
            // Insert display at the correct position in the grid
            if button == "" {
                h.spawn_scene(("main.cob", "display"), |h| {
                    h.get("text").update_on(
                        entity_mutation::<Calculator>(calc_entity),
                        move |id: TargetId, calc: Reactive<Calculator>, mut e: TextEditor| {
                            let text = calc.get(calc_entity)?.buffer_display();
                            write_text!(e, id, "{}", text);
                            OK
                        },
                    );
                });

                continue;
            }

            h.spawn_scene(("main.cob", "button"), |h| {
                h.on_pressed(move |mut c: Commands, mut calc: ReactiveMut<Calculator>| {
                    calc.get_mut(&mut c, calc_entity)?.add_instruction(button);
                    OK
                });
                h.get("text").update_text(button);
            });
        }
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
        .insert_resource(ClearColor(BACKDROP_COLOR.into()))
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

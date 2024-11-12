//! Demonstrates building a calculator in-code using a mix of `sickle_ui` and `bevy_cobweb_ui` utilities.

use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle_ext::prelude::*;
use calc::Context;
use itertools::Itertools;
use rust_decimal::prelude::{Decimal, FromPrimitive};

//-------------------------------------------------------------------------------------------------------------------

const BACKDROP_COLOR: Srgba = SLATE_950;
const NORMAL_BUTTON: Srgba = SLATE_500;
const HOVERED_BUTTON: Srgba = SLATE_400;
const PRESSED_BUTTON: Srgba = GREEN_400;
const BORDER_BUTTON: Srgba = SLATE_400;
const BORDER_DISPLAY: Srgba = SKY_950;

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

fn build_ui(mut c: Commands)
{
    let items = vec![
        "C", "", "7", "8", "9", "/", "4", "5", "6", "*",
        "1", "2", "3", "-", "0", ".", "=", "+",
    ];

    c.ui_root().container(Node::default(), |ui| {
        ui.style()
            .display(Display::Grid)
            .grid_template_columns(RepeatedGridTrack::auto(4))
            .margin(UiRect::all(Val::Auto));
        ui.insert_reactive(Calculator::default());
        let calc_entity = ui.id();

        for item in items {
            let is_display = item == "";
            let (span, br_radius, br_color) = match is_display {
                true => (3, Val::Px(0.), BORDER_DISPLAY.into()),
                false => (1, Val::Px(5.), BORDER_BUTTON.into()),
            };

            ui.container(Node::default(), |ui| {
                ui.style()
                    .grid_column(GridPlacement::span(span))
                    .border(UiRect::all(Val::Px(1.)))
                    .padding(UiRect::all(Val::Px(20.)))
                    .margin(UiRect::all(Val::Px(5.)))
                    .border_radius(BorderRadius::all(br_radius))
                    .border_color(br_color)
                    .justify_content(JustifyContent::Center);
                if is_display {
                    ui.style().background_color(NORMAL_BUTTON.into());
                } else {
                    ui.apply(Interactive)
                        .apply(Responsive::<BackgroundColor> {
                            idle: NORMAL_BUTTON.into(),
                            hover: Some(HOVERED_BUTTON.into()),
                            press: Some(PRESSED_BUTTON.into()),
                            ..default()
                        })
                        .on_pressed(move |mut c: Commands, mut calc: ReactiveMut<Calculator>| {
                            calc.get_mut(&mut c, calc_entity)
                                .unwrap()
                                .add_instruction(item);
                        });
                }

                ui.container(Node::default(), |ui| {
                    ui.apply(TextLine { text: item.into(), size: 30.0, ..default() });

                    if is_display {
                        ui.update_on(entity_mutation::<Calculator>(calc_entity), |id| {
                            move |calc: Reactive<Calculator>, mut e: TextEditor| {
                                let text = calc.get(calc_entity).unwrap().buffer_display();
                                write_text!(e, id, "{}", text);
                            }
                        });
                    }
                });
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
        .add_systems(Startup, (setup, build_ui))
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

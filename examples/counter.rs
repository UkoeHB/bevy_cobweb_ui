//! Demonstrates library primitives and features.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use sickle_ui::ui_builder::*;
use sickle_ui::SickleUiPlugin;

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactComponent, Deref)]
struct Counter(usize);

impl Counter
{
    fn increment(&mut self)
    {
        self.0 += 1;
    }

    fn increment_on(entity: Entity) -> impl FnMut(Commands, ReactiveMut<Counter>)
    {
        move |mut c: Commands, mut counters: ReactiveMut<Counter>| {
            counters.get_mut(&mut c, entity).map(Counter::increment);
        }
    }

    fn write(from: Entity, to: Entity) -> impl FnMut(TextEditor, Reactive<Counter>)
    {
        move |mut editor: TextEditor, counters: Reactive<Counter>| {
            let Some(counter) = counters.get(from) else { return };
            editor.write(to, |t| write!(t, "Count: {}", **counter));
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands)
{
    let file = StyleRef::from_file("examples/counter.style.json");

    c.ui_builder(UiRoot).load(file.e("root"), |root, path| {
        root.load(path.e("button"), |button, path| {
            let button_id = button.id();
            button
                .insert_reactive(Counter(0))
                .on_pressed(Counter::increment_on(button_id));

            button.load(path.e("text"), |text, _path| {
                text.update_on(entity_mutation::<Counter>(button_id), |text_id| {
                    Counter::write(button_id, text_id)
                });
            });
        });
    });
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
    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..Default::default() }),
            ..Default::default()
        }))
        .add_plugins(SickleUiPlugin)
        .add_plugins(CobwebUiPlugin)
        .add_style_sheet("examples/counter.style.json")
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

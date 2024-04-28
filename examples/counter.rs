//! Demonstrates building a simple counter.

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
    fn increment_inner(&mut self)
    {
        self.0 += 1;
    }

    fn increment(target: Entity) -> impl FnMut(Commands, ReactiveMut<Counter>)
    {
        move |mut c: Commands, mut counters: ReactiveMut<Counter>| {
            counters.get_mut(&mut c, target).map(Counter::increment_inner);
        }
    }

    fn write(
        pre_text: impl Into<String>,
        post_text: impl Into<String>,
        from: Entity,
        to: Entity,
    ) -> impl FnMut(TextEditor, Reactive<Counter>)
    {
        let pre_text = pre_text.into();
        let post_text = post_text.into();
        move |mut editor: TextEditor, counters: Reactive<Counter>| {
            let Some(counter) = counters.get(from) else { return };
            editor.write(to, |t| {
                write!(t, "{}{}{}", pre_text.as_str(), **counter, post_text.as_str())
            });
        }
    }
}

//TODO: the design will depend on sickle_ui theming since we want the loaded-from-file stuff to be the base theme,
// and CounterWidget-derived stuff to sit on top of the theme
/*
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CounterWidget
{
    /// Overrides the background color loaded from file.
    #[reflect(default)]
    bg: Option<AnimatedBgColor>,
    /// Overrides the text line loaded from file.
    #[reflect(default)]
    text: Option<TextLine>,
    /// Overrides the text style loaded from file.
    #[reflect(default)]
    text_margin: Option<UiRect>,
    #[reflect(default = "CounterWidget::default_pre_text")]
    pre_text: String,
    #[reflect(default)]
    post_text: String,
}

impl CounterWidget
{
    fn default_pre_text() -> String
    {
        "Count: ".into()
    }
}

impl ApplyLoadable for CounterWidget
{
    /// Inserts a counter widget to the entity.
    fn apply(self, ec: &mut EntityCommands)
    {
        let file = LoadableRef::from_file("examples/counter.load.json");

        c.ui_builder(ec.id()).load(file.e("button"), |button, path| {
            let button_id = button.id();
            button
                .insert_reactive(Counter(0))
                .on_pressed(Counter::increment(button_id));

            button.load(path.e("text"), |text, _path| {
                text.update_on(entity_mutation::<Counter>(button_id), |text_id| {
                    Counter::write(self.pre_text, self.post_text, button_id, text_id)
                });
            });
        });

        ec.try_insert(BackgroundColor(self.0.clone()));
    }
}
*/

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands)
{
    let file = LoadableRef::from_file("examples/counter.load.json");

    c.ui_builder(UiRoot).load(file.e("root"), |root, path| {
        root.load(path.e("button"), |button, path| {
            let button_id = button.id();
            button
                .insert_reactive(Counter(0))
                .on_pressed(Counter::increment(button_id));

            button.load(path.e("text"), |text, _path| {
                text.update_on(entity_mutation::<Counter>(button_id), |text_id| {
                    Counter::write("Counter: ", "", button_id, text_id)
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
        .add_load_sheet("examples/counter.load.json")
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

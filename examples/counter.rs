//! Demonstrates building a simple counter.

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use sickle_ui::theme::Theme;
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

    /// Makes callback for incrementing the counter on `target`.
    fn increment(target: Entity) -> impl FnMut(Commands, ReactiveMut<Counter>)
    {
        move |mut c: Commands, mut counters: ReactiveMut<Counter>| {
            counters.get_mut(&mut c, target).map(Counter::increment_inner);
        }
    }

    /// Makes callback for writing the counter value when it changes.
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

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct CounterButtonTheme;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
struct CounterWidget
{
    config: Option<LoadableRef>,
    theme: Option<LoadableRef>,
    pre_text: Option<String>,
    post_text: Option<String>,
}

impl CounterWidget
{
    /// Makes a default counter widget.
    fn new() -> Self
    {
        Self::default()
    }

    /// Adds config adjustments on top of the default config.
    ///
    /// The loaded structure must match the default config's structure.
    fn config(mut self, config: LoadableRef) -> Self
    {
        self.config = config.into();
        self
    }

    /// Adds theme adjustments on top of the default theme.
    fn theme(mut self, theme: LoadableRef) -> Self
    {
        self.theme = theme.into();
        self
    }

    /// Sets the pre-counter text.
    fn pre_text(mut self, pre_text: impl Into<String>) -> Self
    {
        self.pre_text = pre_text.into().into();
        self
    }

    /// Sets the post-counter text.
    fn _post_text(mut self, post_text: impl Into<String>) -> Self
    {
        self.post_text = post_text.into().into();
        self
    }

    fn default_ref() -> LoadableRef
    {
        LoadableRef::new("examples/counter.load.json", "counter_base_config")
    }

    /// Builds the widget on an entity.
    //todo: use BuildWidget trait and extension method on UiBuilder?
    fn build(self, ec: &mut EntityCommands)
    {
        let base = self.config.unwrap_or_else(|| Self::default_ref());
        let pre_text = self.pre_text.unwrap_or_else(|| "Counter: ".into());
        let post_text = self.post_text.unwrap_or_else(|| "".into());
        let entity = ec.id();

        ec.commands().ui_builder(entity).load(base, |button, path| {
            if let Some(theme) = self.theme {
                button.entity_commands().load_theme::<CounterButtonTheme>(theme);
            }

            let button_id = button.id();
            button
                .insert(CounterButtonTheme)
                .insert_reactive(Counter(0))
                .on_pressed(Counter::increment(button_id));

            button.load(path.e("text"), |text, _path| {
                text.update_on(entity_mutation::<Counter>(button_id), |text_id| {
                    Counter::write(pre_text, post_text, button_id, text_id)
                });
            });
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands)
{
    let file = LoadableRef::from_file("examples/counter.load.json");

    c.ui_builder(UiRoot).load(file.e("root"), |root, path| {
        root.entity_commands()
            .load_theme::<CounterButtonTheme>(file.e("counter_base_theme"));

        // Default widget
        CounterWidget::new().build(&mut root.entity_commands());

        // Widget with custom config
        CounterWidget::new()
            .config(path.e("counter_small_config"))
            .pre_text("Small: ")
            .build(&mut root.entity_commands());

        // Widget with theme adjustments
        CounterWidget::new()
            .theme(path.e("counter_bonus_theme"))
            .config(path.e("counter_bonus_config"))
            .pre_text("Themed: ")
            .build(&mut root.entity_commands());

        // Manual counter
        root.load(CounterWidget::default_ref(), |button, path| {
            let button_id = button.id();
            button
                .insert(CounterButtonTheme)
                .insert_reactive(Counter(0))
                .on_pressed(Counter::increment(button_id));

            button.load(path.e("text"), |text, _path| {
                text.update_on(entity_mutation::<Counter>(button_id), |text_id| {
                    Counter::write("Manual: ", "", button_id, text_id)
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
        .add_systems(PostUpdate, Theme::<CounterButtonTheme>::post_update())
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

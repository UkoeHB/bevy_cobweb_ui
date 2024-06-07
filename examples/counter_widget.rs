//! Demonstrates building a counter with a custom widget and theming.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::theme::{ComponentThemePlugin, DefaultTheme, UiContext};
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use sickle::{DefaultTheme, UiContext};

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
    fn increment(target: Entity) -> impl IntoSystem<(), (), ()>
    {
        IntoSystem::into_system(move |mut c: Commands, mut counters: ReactiveMut<Counter>| {
            counters
                .get_mut(&mut c, target)
                .map(Counter::increment_inner);
        })
    }

    /// Makes callback for writing the counter value when it changes.
    fn write(
        pre_text: impl Into<String>,
        post_text: impl Into<String>,
        from: Entity,
        to: Entity,
    ) -> impl IntoSystem<(), (), ()>
    {
        let pre_text = pre_text.into();
        let post_text = post_text.into();

        IntoSystem::into_system(move |mut editor: TextEditor, counters: Reactive<Counter>| {
            let Some(counter) = counters.get(from) else { return };
            editor.write(to, |t| {
                write!(t, "{}{}{}", pre_text.as_str(), **counter, post_text.as_str())
            });
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for the counter theme.
#[derive(Component, DefaultTheme, UiContext, Copy, Clone, Debug)]
struct CounterButton;

//-------------------------------------------------------------------------------------------------------------------

/// Pre-defined widget structure that can be customized.
#[derive(Default)]
struct CounterWidget
{
    button_config: Option<LoadableRef>,
    text_config: Option<LoadableRef>,
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

    /// Adds config adjustments to the button node on top of the default config.
    fn button_config(mut self, config: LoadableRef) -> Self
    {
        self.button_config = config.into();
        self
    }

    /// Adds config adjustments to the text node on top of the default config.
    fn text_config(mut self, config: LoadableRef) -> Self
    {
        self.text_config = config.into();
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

    /// Returns a reference to the base node of the counter widget (the button node).
    fn default_ref() -> LoadableRef
    {
        LoadableRef::new("examples/widgets/counter.load.json", "counter_widget")
    }

    /// Builds the widget on an entity.
    fn build(self, builder: &mut UiBuilder<Entity>)
    {
        let (button_ref, text_ref) = match (self.button_config, self.text_config) {
            (Some(b), Some(t)) => (b, t),
            (Some(b), None) => (b, Self::default_ref().e("text")),
            (None, Some(t)) => (Self::default_ref(), t),
            (None, None) => {
                // Avoid excess allocations in this case.
                let b = Self::default_ref();
                (b.clone(), b.e("text"))
            }
        };
        let pre_text = self.pre_text.unwrap_or_else(|| "Counter: ".into());
        let post_text = self.post_text.unwrap_or_else(|| "".into());

        let theme_ref = self.theme.unwrap_or_else(|| button_ref.clone());
        builder.load_theme::<CounterButton>(theme_ref, button_ref, |button, _path| {
            let button_id = button.id();
            button
                .insert(CounterButton)
                .insert(PropagateControl)
                .insert_reactive(Counter(0))
                .entity_commands()
                .on_pressed(Counter::increment(button_id));

            button.load(text_ref, |text, _path| {
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
    let widget = LoadableRef::from_file("examples/widgets/counter.load.json");
    let example = LoadableRef::from_file("examples/counter_widget.load.json");

    c.ui_builder(UiRoot).load_theme::<CounterButton>(
        widget.e("counter_theme"),
        example.e("root"),
        |mut root, _path| {
            // Default widget
            CounterWidget::new().build(&mut root);

            // Widget with custom text structure.
            CounterWidget::new()
                .text_config(example.e("counter_widget_text_small"))
                .pre_text("Small: ")
                .build(&mut root);

            // Widget with animated text structure.
            CounterWidget::new()
                .text_config(example.e("counter_widget_text_responsive"))
                .pre_text("Responsive: ")
                .build(&mut root);

            // Widget with theme adjustments
            CounterWidget::new()
                .theme(example.e("counter_theme_flexible"))
                .button_config(example.e("counter_widget_button_flexible"))
                .pre_text("Themed: ")
                .build(&mut root);
        },
    );
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
        .add_plugins(ComponentThemePlugin::<CounterButton>::new())
        .load_sheet("examples/widgets/counter.load.json")
        .load_sheet("examples/counter_widget.load.json")
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

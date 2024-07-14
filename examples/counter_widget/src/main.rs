//! Demonstrates building a counter with a custom widget using a cobweb asset file 'specification'.

use std::any::type_name;

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::theme::{ComponentThemePlugin, DefaultTheme, UiContext};
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::{DefaultTheme, SickleUiPlugin};

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
            write_text!(editor, to, "{}{}{}", pre_text.as_str(), counter.0, post_text.as_str());
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(TypeName)]
struct CounterWidgetText;

/// Marker type for the counter theme.
#[derive(Component, DefaultTheme, Copy, Clone, Debug)]
struct CounterWidget
{
    text_entity: Entity,
}

impl UiContext for CounterWidget
{
    fn get(&self, target: &str) -> Result<Entity, String>
    {
        match target {
            CounterWidgetText::NAME => Ok(self.text_entity),
            _ => Err(format!("unknown UI context {target} for {}", type_name::<Self>())),
        }
    }
    fn contexts(&self) -> Vec<&'static str>
    {
        vec![CounterWidgetText::NAME]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Pre-defined widget structure that can be customized.
#[derive(Default)]
struct CounterWidgetBuilder
{
    spec: Option<LoadableRef>,
    pre_text: Option<String>,
    post_text: Option<String>,
}

impl CounterWidgetBuilder
{
    /// Makes a default counter widget.
    fn new() -> Self
    {
        Self::default()
    }

    /// Sets the path where the widget specification should be loaded from.
    fn spec(mut self, spec: LoadableRef) -> Self
    {
        self.spec = spec.into();
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

    /// Returns a reference to the default counter widget file.
    fn default_file() -> LoadableRef
    {
        LoadableRef::from_file("widgets.counter")
    }

    /// Builds the widget on an entity.
    fn build<'a>(self, builder: &'a mut UiBuilder<Entity>) -> UiBuilder<'a, Entity>
    {
        let pre_text = self.pre_text.unwrap_or_else(|| "Counter: ".into());
        let post_text = self.post_text.unwrap_or_else(|| "".into());

        let button = self
            .spec
            .unwrap_or_else(|| Self::default_file().e("counter_widget"));

        let mut core_entity = Entity::PLACEHOLDER;
        let mut text_entity = Entity::PLACEHOLDER;

        builder.load_with_theme::<CounterWidget>(button, &mut core_entity, |button, path| {
            let button_id = button.id();
            button
                .insert_reactive(Counter(0))
                .on_released(Counter::increment(button_id));

            button.load_with_subtheme::<CounterWidget, CounterWidgetText>(
                path.e("text"),
                &mut text_entity,
                |text, _path| {
                    text.update_on(
                        (
                            entity_event::<PressCanceled>(button_id),
                            entity_mutation::<Counter>(button_id),
                        ),
                        |text_id| Counter::write(pre_text, post_text, button_id, text_id),
                    );
                },
            );

            button.insert(CounterWidget { text_entity });
        });

        builder.commands().ui_builder(core_entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let file = LoadableRef::from_file("examples.counter_widget");
    let scene = file.e("root");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        let n = l.deref_mut();

        // Default widget
        CounterWidgetBuilder::new().build(n);

        // Widget with custom text structure.
        CounterWidgetBuilder::new()
            .spec(file.e("counter_widget_small_text"))
            .pre_text("Small: ")
            .build(n);

        // Widget with animated text structure.
        CounterWidgetBuilder::new()
            .spec(file.e("counter_widget_responsive_text"))
            .pre_text("Text: ")
            .build(n)
            .edit_child::<CounterWidget>(CounterWidgetText::NAME, |c, core, text| {
                c.ui_builder(core).on_pressed(move |mut e: TextEditor| {
                    write_text!(e, text, "Pressed");
                });
            });

        // Widget with theme adjustments
        CounterWidgetBuilder::new()
            .spec(file.e("counter_widget_flexible_button"))
            .pre_text("Themed: ")
            .build(n);
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
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..default() }),
            ..default()
        }))
        .add_plugins(SickleUiPlugin)
        .add_plugins(ReactPlugin)
        .add_plugins(CobwebUiPlugin)
        .add_plugins(ComponentThemePlugin::<CounterWidget>::new())
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

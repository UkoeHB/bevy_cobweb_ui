//! Demonstrates building a counter with a custom widget and theming.

use std::any::type_name;

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::theme::{ComponentThemePlugin, DefaultTheme, UiContext};
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use sickle::DefaultTheme;

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
            counter.write_to_editor(&mut editor, to, pre_text.as_str(), post_text.as_str());
        })
    }

    fn write_to_editor(&self, editor: &mut TextEditor, to: Entity, pre_text: &str, post_text: &str)
    {
        editor.write(to, |t| write!(t, "{}{}{}", pre_text, self.0, post_text));
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

impl CounterWidget
{
    fn load_base_theme(node: &mut UiBuilder<Entity>)
    {
        let theme = CounterWidgetBuilder::default_file().e("theme");
        node.load_theme::<CounterWidget>(theme.e("core"));
        node.load_subtheme::<CounterWidget, CounterWidgetText>(theme.e("text"));
    }
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
    button_config: Option<LoadableRef>,
    text_config: Option<LoadableRef>,
    core_theme: Option<LoadableRef>,
    text_theme: Option<LoadableRef>,
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

    /// Adds theme adjustments on top of the default core theme.
    fn core_theme(mut self, core_theme: LoadableRef) -> Self
    {
        self.core_theme = core_theme.into();
        self
    }

    /// Adds theme adjustments on top of the default text theme.
    fn _text_theme(mut self, text_theme: LoadableRef) -> Self
    {
        self.text_theme = text_theme.into();
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

        let button_ref = self
            .button_config
            .unwrap_or_else(|| Self::default_file().e("structure"));
        let text_ref = self
            .text_config
            .unwrap_or_else(|| Self::default_file().e("structure::text"));

        let mut core_entity = Entity::PLACEHOLDER;
        let mut text_entity = Entity::PLACEHOLDER;

        builder.load_with_theme::<CounterWidget>(button_ref, &mut core_entity, |button, _path| {
            // Load extra theme info.
            if let Some(theme) = self.core_theme {
                button.load_theme::<CounterWidget>(theme);
            }

            let button_id = button.id();
            button
                .insert_reactive(Counter(0))
                .on_released(Counter::increment(button_id));

            button.load_with_subtheme::<CounterWidget, CounterWidgetText>(
                text_ref,
                &mut text_entity,
                |text, _path| {
                    // Load extra theme info.
                    if let Some(theme) = self.text_theme {
                        text.load_subtheme::<CounterWidget, CounterWidgetText>(theme);
                    }

                    text.update_on(entity_mutation::<Counter>(button_id), |text_id| {
                        Counter::write(pre_text, post_text, button_id, text_id)
                    });
                },
            );

            button.insert(CounterWidget { text_entity });
        });

        builder.commands().ui_builder(core_entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands)
{
    let example = LoadableRef::from_file("examples.counter_widget");

    c.ui_builder(UiRoot).load(example.e("root"), |root, _path| {
        // Load base themes.
        CounterWidget::load_base_theme(root);

        // Default widget
        CounterWidgetBuilder::new().build(root);

        // Widget with custom text structure.
        CounterWidgetBuilder::new()
            .text_config(example.e("counter_widget_text_small"))
            .pre_text("Small: ")
            .build(root);

        // Widget with animated text structure.
        CounterWidgetBuilder::new()
            .text_config(example.e("counter_widget_text_responsive"))
            .pre_text("Text: ")
            .build(root)
            .edit_child::<CounterWidget, CounterWidgetText>(|c, core, text| {
                c.ui_builder(core).on_pressed(move |mut e: TextEditor| {
                    e.write(text, |t| write!(t, "Pressed"));
                });
            });

        // Widget with theme adjustments
        CounterWidgetBuilder::new()
            .core_theme(example.e("counter_theme_flexible"))
            .button_config(example.e("counter_widget_button_flexible"))
            .pre_text("Themed: ")
            .build(root);
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
        .add_plugins(ComponentThemePlugin::<CounterWidget>::new())
        .load_sheet("examples/counter_widget.load.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

//! Showcases how `sickle_ui` theming can be integrated to override theme attributes.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::theme::{ComponentThemePlugin, DefaultTheme, UiContext};
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::{DefaultTheme, SickleUiPlugin, UiContext};

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for the example text theme.
///
/// Note that we register this in the app with `ComponentThemePlugin::<ExampleText>::new()`.
#[derive(Component, DefaultTheme, UiContext, Copy, Clone, Debug)]
struct ExampleText;

impl ExampleText
{
    fn load_base_theme(ui: &mut UiBuilder<Entity>)
    {
        let theme = SceneRef::new(ExampleTextBuilder::widget_file(), "theme");

        // This loads `Theme<ExampleText>` into the entity without adding an `ExampleText` component. The theme
        // data is stored for inheritence, but is not *used* by this entity.
        ui.load_theme::<ExampleText>(theme);
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct ExampleTextBuilder
{
    text: String,
    theme_override: Option<SceneRef>,
}

impl ExampleTextBuilder
{
    fn new(text: impl Into<String>) -> Self
    {
        Self { text: text.into(), theme_override: None }
    }

    /// Sets the location to load theme override data from.
    fn theme_override(mut self, theme: SceneRef) -> Self
    {
        self.theme_override = Some(theme);
        self
    }

    fn widget_file() -> &'static str
    {
        "examples.widgets.trivial_text"
    }

    /// Builds the text as a child of the builder entity.
    fn build<'a>(mut self, node: &'a mut UiBuilder<Entity>) -> UiBuilder<'a, Entity>
    {
        let structure = SceneRef::new(&Self::widget_file(), "structure");

        let mut core_entity = Entity::PLACEHOLDER;

        node.load(structure, |core, _path| {
            core_entity = core.id();

            // Load override data if provided.
            if let Some(theme_override) = self.theme_override.take() {
                core.load_theme::<ExampleText>(theme_override);
            }

            // Must use update_on so hot-reloads will be detected.
            let text_val = self.text;
            core.update_on((), |id| {
                move |mut e: TextEditor| {
                    write_text!(e, id, "{:?}", text_val);
                }
            });

            // Add the theme component so the theme will be applied to this entity.
            // - If this entity had sub-entities, then those would need to be included in ExampleText, which would
            //   need to implement UiContext manually.
            core.insert(ExampleText);
        });

        // Return UiBuilder for root of widget.
        node.commands().ui_builder(core_entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = SceneRef::new("examples.trivial_text", "scene");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        let n = l.deref_mut();

        // Prepare themes.
        ExampleText::load_base_theme(n);

        // Add text with default theme.
        ExampleTextBuilder::new("Text with default theme.").build(n);

        // Add text with 'large text' theme.
        ExampleTextBuilder::new("Text with 'large text' theme override.")
            .theme_override(SceneRef::new("examples.trivial_text", "large_text_override"))
            .build(n);

        // Add text with 'black text' theme.
        ExampleTextBuilder::new("Text with 'yellow text' theme override.")
            .theme_override(SceneRef::new("examples.trivial_text", "yellow_text_override"))
            .build(n);
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
        .add_plugins(ComponentThemePlugin::<ExampleText>::new())
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

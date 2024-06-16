//! A simple radio button widget.

use std::any::type_name;

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use sickle::theme::pseudo_state::{PseudoState, PseudoStates};
use sickle::theme::{ComponentThemePlugin, DefaultTheme, UiContext};
use sickle::ui_builder::*;
use sickle::widgets::prelude::UiContainerExt;
use sickle::{DefaultTheme, SickleUiPlugin};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct RadioButtonManager
{
    selected: Option<Entity>,
}

impl RadioButtonManager
{
    fn new() -> Self
    {
        Self { selected: None }
    }

    /// Inserts the manager onto the builder entity.
    ///
    /// Returns the entity where the manager is stored.
    fn insert(self, node: &mut UiBuilder<Entity>) -> Entity
    {
        node.insert(self);
        node.id()
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(TypeName)]
struct RadioButtonOutline;
#[derive(TypeName)]
struct RadioButtonIndicator;
#[derive(TypeName)]
struct RadioButtonText;

/// Marker component for the radio button theme.
#[derive(Component, DefaultTheme, Copy, Clone, Debug)]
struct RadioButton
{
    outline: Entity,
    indicator: Entity,
    text: Entity,
}

impl RadioButton
{
    fn load_base_theme(builder: &mut UiBuilder<Entity>)
    {
        let theme = LoadableRef::new(RadioButtonBuilder::default_file(), "theme");
        builder.load_theme::<RadioButton>(theme.e("core"));
        builder.load_subtheme::<RadioButton, RadioButtonOutline>(theme.e("outline"));
        builder.load_subtheme::<RadioButton, RadioButtonIndicator>(theme.e("indicator"));
        builder.load_subtheme::<RadioButton, RadioButtonText>(theme.e("text"));
    }
}

impl UiContext for RadioButton
{
    fn get(&self, target: &str) -> Result<Entity, String>
    {
        match target {
            RadioButtonOutline::NAME => Ok(self.outline),
            RadioButtonIndicator::NAME => Ok(self.indicator),
            RadioButtonText::NAME => Ok(self.text),
            _ => Err(format!("unknown UI context {target} for {}", type_name::<Self>())),
        }
    }
    fn contexts(&self) -> Vec<&'static str>
    {
        vec![RadioButtonOutline::NAME, RadioButtonIndicator::NAME, RadioButtonText::NAME]
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct RadioButtonBuilder
{
    text: String,
}

impl RadioButtonBuilder
{
    fn new(text: impl Into<String>) -> Self
    {
        Self { text: text.into() }
    }

    fn default_file() -> &'static str
    {
        "widgets.radio_button"
    }

    /// Builds the button as a child of the builder entity.
    ///
    /// The `manager_entity` should have a [`RadioButtonManager`] component.
    fn build<'a>(self, manager_entity: Entity, node: &'a mut UiBuilder<Entity>) -> UiBuilder<'a, Entity>
    {
        let structure = LoadableRef::new(&Self::default_file(), "structure");

        let mut core_entity = Entity::PLACEHOLDER;
        let mut outline_entity = Entity::PLACEHOLDER;
        let mut indicator_entity = Entity::PLACEHOLDER;
        let mut text_entity = Entity::PLACEHOLDER;

        node.load_with_theme::<RadioButton>(structure.e("core"), |core, path| {
            core_entity = core.id();
            core
                // Select this button.
                // Note: this callback could be moved to an EntityWorldReactor, with the manager entity as entity
                // data.
                .on_pressed(move |mut c: Commands, states: Query<&PseudoStates>| {
                    if let Ok(states) = states.get(core_entity) {
                        if states.has(&PseudoState::Selected) {
                            return;
                        }
                    }

                    c.react().entity_event(core_entity, Select);
                })
                // Save the newly-selected button and deselect the previously selected.
                .on_select(move |mut c: Commands, mut managers: Query<&mut RadioButtonManager>| {
                    let Ok(mut manager) = managers.get_mut(manager_entity) else { return };
                    if let Some(prev) = manager.selected {
                        c.react().entity_event(prev, Deselect);
                    }
                    manager.selected = Some(core_entity);
                });

            core.load_with_subtheme::<RadioButton, RadioButtonOutline>(path.e("outline"), |outline, path| {
                outline_entity = outline.id();
                outline.load_with_subtheme::<RadioButton, RadioButtonIndicator>(
                    path.e("indicator"),
                    |indicator, _| {
                        indicator_entity = indicator.id();
                    },
                );
            });

            core.load_with_subtheme::<RadioButton, RadioButtonText>(path.e("text"), |text, _| {
                text_entity = text.id();

                // Note: The text needs to be updated on load otherwise it may be overwritten.
                let text_val = self.text;
                text.update_on(entity_event::<Loaded>(text.id()), |id| {
                    move |mut e: TextEditor| {
                        e.write(id, |t| write!(t, "{}", text_val.as_str()));
                    }
                });
            });

            core.insert(RadioButton {
                outline: outline_entity,
                indicator: indicator_entity,
                text: text_entity,
            });
        });

        // Return UiBuilder for root of button where interactions will be detected.
        node.commands().ui_builder(core_entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands)
{
    let file = LoadableRef::from_file("examples.radio_buttons");
    static OPTIONS: [&'static str; 3] = ["A", "B", "C"];

    c.ui_builder(UiRoot).load(file.e("root"), |root, path| {
        // Prepare themes.
        RadioButton::load_base_theme(root);

        // Display the selected option.
        let mut display_text = Entity::PLACEHOLDER;
        root.load(path.e("display"), |display, path| {
            display.load(path.e("text"), |text, _| {
                display_text = text.id();
            });
        });

        // Insert radio buttons.
        root.load(path.e("radio_frame"), |frame, _| {
            let manager_entity = RadioButtonManager::new().insert(frame);

            for (i, option) in OPTIONS.iter().enumerate() {
                // Add radio button.
                let button_entity = RadioButtonBuilder::new(*option)
                    .build(manager_entity, frame)
                    .on_select(move |mut e: TextEditor| {
                        e.write(display_text, |t| write!(t, "Selected: {}", option));
                    })
                    .id();

                // Select the first option.
                if i == 0 {
                    frame.react().entity_event(button_entity, Select);
                }
            }
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn init_loading_display(mut c: Commands)
{
    c.ui_builder(UiRoot)
        .container(NodeBundle::default(), |node| {
            node.insert_reactive(FlexStyle::default())
                .insert_derived(Width(Val::Vw(100.)))
                .insert_derived(Height(Val::Vh(100.)))
                .insert_derived(SetFlexDirection(FlexDirection::Column))
                .insert_derived(SetJustifyMain(JustifyMain::Center))
                .insert_derived(SetJustifyCross(JustifyCross::Center))
                .despawn_on_broadcast::<StartupLoadingDone>();

            node.container(NodeBundle::default(), |node| {
                node.insert_derived(TextLine { text: "Loading...".into(), font: None, size: 75.0 });
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
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..Default::default() }),
            ..Default::default()
        }))
        .add_plugins(SickleUiPlugin)
        .add_plugins(CobwebUiPlugin)
        .add_plugins(ComponentThemePlugin::<RadioButton>::new())
        .load_sheet("examples/radio_buttons.load.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Loading), init_loading_display)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

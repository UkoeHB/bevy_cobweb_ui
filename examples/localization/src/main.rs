//! Demonstrates localization of text (TODO: and fonts, images).

// TODO: The radio button widget is duplicated from the radio_buttons example, but it should be reusable.

use std::any::type_name;

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::theme::pseudo_state::{PseudoState, PseudoStates};
use bevy_cobweb_ui::sickle::theme::{ComponentThemePlugin, DefaultTheme, UiContext};
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::{DefaultTheme, SickleUiPlugin};

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
    outline_entity: Entity,
    indicator_entity: Entity,
    text_entity: Entity,
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
            RadioButtonOutline::NAME => Ok(self.outline_entity),
            RadioButtonIndicator::NAME => Ok(self.indicator_entity),
            RadioButtonText::NAME => Ok(self.text_entity),
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

        node.load_with_theme::<RadioButton>(structure.e("core"), &mut core_entity, |core, path| {
            let core_id = core.id();
            core
                // Select this button.
                // Note: this callback could be moved to an EntityWorldReactor, with the manager entity as entity
                // data.
                .on_pressed(move |mut c: Commands, states: Query<&PseudoStates>| {
                    if let Ok(states) = states.get(core_id) {
                        if states.has(&PseudoState::Selected) {
                            return;
                        }
                    }

                    c.react().entity_event(core_id, Select);
                })
                // Save the newly-selected button and deselect the previously selected.
                .on_select(move |mut c: Commands, mut managers: Query<&mut RadioButtonManager>| {
                    let Ok(mut manager) = managers.get_mut(manager_entity) else { return };
                    if let Some(prev) = manager.selected {
                        c.react().entity_event(prev, Deselect);
                    }
                    manager.selected = Some(core_id);
                });

            core.load_with_subtheme::<RadioButton, RadioButtonOutline>(
                path.e("outline"),
                &mut outline_entity,
                |outline, path| {
                    outline.load_with_subtheme::<RadioButton, RadioButtonIndicator>(
                        path.e("indicator"),
                        &mut indicator_entity,
                        |_, _| {},
                    );
                },
            );

            core.load_with_subtheme::<RadioButton, RadioButtonText>(
                path.e("text"),
                &mut text_entity,
                |text, _| {
                    // Note: The text needs to be updated on load otherwise it may be overwritten.
                    let text_val = self.text;
                    text.update_on((), |id| {
                        move |mut e: TextEditor| {
                            e.write(id, |t| write!(t, "{}", text_val.as_str()));
                        }
                    });
                },
            );

            core.insert(RadioButton { outline_entity, indicator_entity, text_entity });
        });

        // Return UiBuilder for root of button where interactions will be detected.
        node.commands().ui_builder(core_entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("localization", "root");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        // Prep the radio button theme.
        RadioButton::load_base_theme(l.deref_mut());

        // Dropdown for selecting a language.
        l.edit("selection_section::selection_box", |l| {
            // Update the selection whenever the manifest is updated.
            l.update_on(broadcast::<LocalizationManifestUpdated>(), |id| {
                move |mut c: Commands, manifest: ReactRes<LocalizationManifest>| {
                    // Despawn existing buttons.
                    c.entity(id).despawn_descendants();

                    // Spawn new buttons for everything in the manifest.
                    let mut n = c.ui_builder(id);
                    let manager_entity = RadioButtonManager::new().insert(&mut n);
                    let current_lang = &manifest.negotiated()[0];

                    for language in manifest.languages() {
                        let name = language.display_name();
                        let lang = language.id.clone();

                        let button_id = RadioButtonBuilder::new(name)
                            .build(manager_entity, &mut n)
                            .on_select(move |mut locale: ResMut<Locale>| {
                                *locale = Locale::new_from_id(lang.clone());
                            })
                            .id();

                        // Select the current locale.
                        if &language.id == current_lang {
                            n.react().entity_event(button_id, Select);
                        }
                    }
                }
            });
        });

        l.edit("text_section", |l| {
            // Unlocalized text.
            l.edit("unlocalized", |l| {
                l.insert_derived(TextLine::from_text("This text is not localized."));
            });

            // Untranslated text (only localized in the default language).
            l.edit("untranslated", |l| {
                l.insert(LocalizedText::default());
                l.insert_derived(TextLine::from_text("untranslated"));
            });

            // Localized and partly translated text (localized in only some, but not all, alternate languages).
            l.edit("partially_translated", |l| {
                l.insert(LocalizedText::default());
                l.insert_derived(TextLine::from_text("partly-translated"));
            });

            // Localized and fully translated text.
            l.edit("fully_translated", |l| {
                l.insert(LocalizedText::default());
                l.insert_derived(TextLine::from_text("fully-translated"));
            });

            // Localized dynamic text.
            l.edit("dynamic", |l| {
                l.insert(LocalizedText::default());
                l.insert_derived(TextLine::default());
                l.update_on(broadcast::<TextLocalizerUpdated>(), |id| {
                    move |mut count: Local<usize>, mut t: TextEditor| {
                        t.write(id, |t| write!(t, "locale-counter?count={:?}", *count));
                        *count += 1;
                    }
                });
            });

            // Localized text from file (see `assets/main.caf.json`).
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

    //c.insert_resource(Locale::new("de-DE").unwrap());
}

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..default() }),
            ..default()
        }))
        .add_plugins(ReactPlugin)
        .add_plugins(SickleUiPlugin)
        .add_plugins(CobwebUiPlugin)
        .add_plugins(ComponentThemePlugin::<RadioButton>::new())
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

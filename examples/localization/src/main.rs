//! Demonstrates localization of text (TODO: and fonts, images).

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use bevy_cobweb_ui::widgets::radio_buttons::{RadioButtonBuilder, RadioButtonManager};

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("localization", "root");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        // Language selection list.
        l.edit("selection_section::selection_box", |l| {
            // Update the selection whenever the manifest is updated with a new base language list.
            l.update_on(broadcast::<LocalizationManifestUpdated>(), |id| {
                move |mut c: Commands, manifest: ReactRes<LocalizationManifest>| {
                    // Despawn existing buttons.
                    c.entity(id).despawn_descendants();

                    // Spawn new buttons for everything in the manifest.
                    let mut n = c.ui_builder(id);
                    let manager_entity = RadioButtonManager::setup(&mut n);
                    let current_lang = &manifest.negotiated()[0];

                    for language in manifest.languages() {
                        let name = language.display_name();
                        let lang = language.id.clone();

                        let button_id = RadioButtonBuilder::new_in_box(name)
                            .build(manager_entity, &mut n)
                            .on_select(move |mut locale: ResMut<Locale>| {
                                *locale = Locale::new_from_id(lang.clone());
                            })
                            .id();

                        // Select the current locale.
                        if language.id == *current_lang {
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
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
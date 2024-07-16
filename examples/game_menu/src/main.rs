//! An example game menu.

use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowTheme};
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use bevy_cobweb_ui::widgets::radio_button::{RadioButtonBuilder, RadioButtonManager};

//-------------------------------------------------------------------------------------------------------------------

struct SliderChanged;

fn detect_silder_change(mut c: Commands, query: Query<Entity, Changed<Slider>>)
{
    for slider in query.iter() {
        c.react().entity_event(slider, SliderChanged);
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct DropdownChanged;

fn detect_dropdown_change(mut c: Commands, query: Query<Entity, Changed<Dropdown>>)
{
    for slider in query.iter() {
        c.react().entity_event(slider, DropdownChanged);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_home_page_content<'a>(_l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>) {}

//-------------------------------------------------------------------------------------------------------------------

fn build_play_page_content<'a>(_l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>) {}

//-------------------------------------------------------------------------------------------------------------------

fn build_settings_page_content<'a>(l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>)
{
    l.edit("audio::slider", |l| {
        // Slider: sickle_ui built-in widget.
        // TODO: Overwrite default styling.
        l.slider(SliderConfig::horizontal(None, 0.0, 100.0, 0.0, true))
            .on_event::<SliderChanged>()
            .r(|event: EntityEvent<SliderChanged>, sliders: Query<&Slider>| {
                let _slider = sliders.get(event.entity()).unwrap();

                // NOT IMPLEMENTED: Adjust app's audio settings with slider value.
            });
    });

    l.edit("vsync", |l| {
        // Radio buttons: bevy_cobweb_ui built-in widget.
        let manager_entity = RadioButtonManager::insert(l.deref_mut());
        l.edit("options", |l| {
            let button_loc = LoadableRef::new(l.path().file.as_str(), "settings_radio_button");

            // Option: enable vsync
            let enabled = RadioButtonBuilder::custom_with_text(button_loc.clone(), "vsync-on")
                .localized()
                .with_indicator()
                .build(manager_entity, l.deref_mut())
                .on_select(|mut window: Query<&mut Window, With<PrimaryWindow>>| {
                    window.single_mut().present_mode = PresentMode::AutoVsync;
                    tracing::info!("vsync set to on");
                })
                .id();

            // Option: disable vsync
            let disabled = RadioButtonBuilder::custom_with_text(button_loc.clone(), "vsync-off")
                .localized()
                .with_indicator()
                .build(manager_entity, l.deref_mut())
                .on_select(|mut window: Query<&mut Window, With<PrimaryWindow>>| {
                    window.single_mut().present_mode = PresentMode::AutoNoVsync;
                    tracing::info!("vsync set to off");
                })
                .id();

            // Select correct button based on initial value.
            l.commands().syscall_once(
                (),
                move |mut c: Commands, window: Query<&Window, With<PrimaryWindow>>| match window
                    .single()
                    .present_mode
                {
                    PresentMode::AutoNoVsync => c.react().entity_event(disabled, Select),
                    _ => c.react().entity_event(enabled, Select),
                },
            );
        });
    });

    l.edit("localization::dropdown", |l| {
        // Drop-down: sickle_ui built-in widget.
        // TODO: Overwrite default styling.
        l.update_on(broadcast::<LocalizationManifestUpdated>(), |id| {
            move |mut c: Commands, manifest: Res<LocalizationManifest>| {
                // Delete current dropdown node in case we are rebuilding due to a new language list.
                let mut n = c.ui_builder(id);
                n.entity_commands().despawn_descendants();

                // Get languages and identify position of current language.
                let languages: Vec<String> = manifest
                    .languages()
                    .iter()
                    .map(LocalizationMeta::display_name)
                    .collect();

                // Find position of current language.
                let position = manifest
                    .negotiated()
                    .get(0)
                    .map(|main_lang| manifest.languages().iter().position(|m| m.id == *main_lang))
                    .flatten();

                // Add dropdown.
                let mut dropdown = n.dropdown(languages.clone(), position);

                // When the dropdown selection changes, update the locale's requested language.
                let mut selection = position;
                let dropdown_id = dropdown.id();
                dropdown.on_event::<DropdownChanged>().r(
                    move |mut locale: ResMut<Locale>,
                          manifest: Res<LocalizationManifest>,
                          dropdowns: Query<&Dropdown>| {
                        let dropdown = dropdowns.get(dropdown_id).unwrap();
                        if selection == dropdown.value() {
                            return;
                        }
                        selection = dropdown.value();

                        if let Some(selection) = selection {
                            let new_id = manifest.languages()[selection].id.clone();
                            locale.requested = vec![new_id];
                        } else {
                            locale.requested = manifest
                                .get_default()
                                .map(|m| m.id.clone())
                                .into_iter()
                                .collect();
                        }
                    },
                );
            }
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn add_menu_option<'a>(
    l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>,
    file: &LoadableRef,
    content_path: &str,
    button_text: &str,
    page_scene: &str,
    page_content_fn: impl for<'b> FnOnce(&mut LoadedScene<'b, '_, UiBuilder<'b, Entity>>),
    start_selected: bool,
)
{
    let manager_entity = l.id();

    // Load content page for this section.
    let mut page_entity = Entity::PLACEHOLDER;
    l.edit_from_root(content_path, |l| {
        l.load_scene(file.e(page_scene), |l| {
            page_entity = l.id();
            l.insert_reactive(DisplayControl::Hide);

            // Add custom logic to the page.
            (page_content_fn)(l);
        });
    });

    // Add button.
    // - We toggle content visibility on select/deselect.
    let button_entity = RadioButtonBuilder::custom_with_text(file.e("menu_option_button"), button_text)
        .localized()
        .build(manager_entity, l.deref_mut())
        .on_select(move |mut c: Commands| {
            c.entity(page_entity)
                .insert_reactive(DisplayControl::Display);
        })
        .on_deselect(move |mut c: Commands| {
            c.entity(page_entity).insert_reactive(DisplayControl::Hide);
        })
        .id();

    // Select if requested.
    if start_selected {
        l.react().entity_event(button_entity, Select);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let file = LoadableRef::from_file("main.caf.json");
    let scene = file.e("menu_scene");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.edit("menu::options", |l| {
            RadioButtonManager::insert(l.deref_mut());
            add_menu_option(
                l,
                &file,
                "content",
                "menu-option-home",
                "home_page",
                build_home_page_content,
                true,
            );
            add_menu_option(
                l,
                &file,
                "content",
                "menu-option-play",
                "play_page",
                build_play_page_content,
                false,
            );
            add_menu_option(
                l,
                &file,
                "content",
                "menu-option-settings",
                "settings_page",
                build_settings_page_content,
                false,
            );
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
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..default() }),
            ..default()
        }))
        .add_plugins(SickleUiPlugin)
        .add_plugins(ReactPlugin)
        .add_plugins(CobwebUiPlugin)
        .load("main.caf.json")
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        // temporary hack for interop
        // TODO: move to custom schedule between Update and PostUpdate? or add system sets to sickle_ui for
        // ordering in update?
        .add_systems(PostUpdate, (detect_silder_change, detect_dropdown_change))
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

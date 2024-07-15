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

fn build_home_page_content<'a>(_l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>) {}

//-------------------------------------------------------------------------------------------------------------------

fn build_play_page_content<'a>(_l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>) {}

//-------------------------------------------------------------------------------------------------------------------

fn build_settings_page_content<'a>(l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>)
{
    l.edit("audio::slider", |l| {
        // TODO: Overwrite default styling.
        l.slider(SliderConfig::horizontal(None, 0.0, 100.0, 0.0, true))
            .on_event::<SliderChanged>()
            .r(|event: EntityEvent<SliderChanged>, sliders: Query<&Slider>| {
                let _slider = sliders.get(event.read().0).unwrap();

                // NOT IMPLEMENTED: Adjust app's audio settings with slider value.
            });
    });

    l.edit("vsync", |l| {
        let manager_entity = RadioButtonManager::insert(l.deref_mut());
        l.edit("options", |l| {
            let button_loc = LoadableRef::from_file(l.path().file.as_str()).e("settings_radio_button");

            // Option: enable vsync
            let enabled = RadioButtonBuilder::custom_with_text(button_loc.clone(), "On")
                .with_indicator()
                .build(manager_entity, l.deref_mut())
                .on_select(|mut window: Query<&mut Window, With<PrimaryWindow>>| {
                    window.single_mut().present_mode = PresentMode::AutoVsync;
                    tracing::info!("vsync set to on");
                })
                .id();

            // Option: disable vsync
            let disabled = RadioButtonBuilder::custom_with_text(button_loc.clone(), "Off")
                .with_indicator()
                .build(manager_entity, l.deref_mut())
                .on_select(|mut window: Query<&mut Window, With<PrimaryWindow>>| {
                    window.single_mut().present_mode = PresentMode::AutoNoVsync;
                    tracing::info!("vsync set to off");
                })
                .id();

            // Get initial value.
            l.commands().syscall(
                (),
                move |mut c: Commands, window: Query<&Window, With<PrimaryWindow>>| match window
                    .single()
                    .present_mode
                {
                    PresentMode::AutoNoVsync => c.react().entity_event(disabled, Deselect),
                    _ => c.react().entity_event(enabled, Select),
                },
            );
        });
    });

    // TODO: language control (drop-down)
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
            l.insert(Visibility::Hidden);

            // Add custom logic to the page.
            (page_content_fn)(l);
        });
    });

    // Add button.
    // - We toggle content visibility on select/deselect. Content pages should use AbsoluteStyle so their layouts
    //   don't interfere.
    let button_entity = RadioButtonBuilder::custom_with_text(file.e("menu_option_button"), button_text)
        .localized()
        .build(manager_entity, l.deref_mut())
        .on_select(move |mut c: Commands| {
            c.entity(page_entity).insert(Visibility::Inherited);
        })
        .on_deselect(move |mut c: Commands| {
            c.entity(page_entity).insert(Visibility::Hidden);
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
        //todo: move to custom schedule between Update and PostUpdate? or add system set to sickle_ui for ordering
        // in update?
        .add_systems(PostUpdate, detect_silder_change)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

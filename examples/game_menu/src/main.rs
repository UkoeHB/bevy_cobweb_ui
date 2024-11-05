//! An example game menu.
fn main() {}
/*

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowTheme};
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::builtin::widgets::radio_button::{RadioButtonBuilder, RadioButtonManager};
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle_ext::prelude::*;

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

/// Style override for the `sickle_ui` `Slider` widget.
fn adjusted_slider_style(style_builder: &mut StyleBuilder, slider: &Slider, theme_data: &ThemeData)
{
    // This is styling for a horizontal slider.
    {
        style_builder
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
            .width(Val::Percent(100.))
            .height(Val::Px(4.0))
            .padding(UiRect::horizontal(Val::Px(4.0)));

        style_builder
            .switch_target(Slider::LABEL)
            .margin(UiRect::right(Val::Px(0.0)));

        style_builder
            .switch_target(Slider::BAR_CONTAINER)
            .width(Val::Percent(100.));

        style_builder
            .switch_target(Slider::BAR)
            .width(Val::Percent(100.))
            .height(Val::Px(10.0))
            .margin(UiRect::vertical(Val::Px(4.0)));

        style_builder
            .switch_target(Slider::READOUT)
            .min_width(Val::Px(50.0))
            .margin(UiRect::left(Val::Px(5.0)));

        style_builder
            .switch_context(Slider::HANDLE, None)
            .margin(UiRect::px(-2.0, 0., -10.0, 0.));
    }

    style_builder.reset_context();

    style_builder
        .switch_target(Slider::LABEL)
        .sized_font(SizedFont {
            font: "embedded://bevy_cobweb_ui/fonts/FiraSans-Regular.ttf".into(),
            size: 25.0,
        })
        .font_color(Color::WHITE);

    if slider.config().label.is_none() {
        style_builder
            .switch_target(Slider::LABEL)
            .display(Display::None)
            .visibility(Visibility::Hidden);
    } else {
        style_builder
            .switch_target(Slider::LABEL)
            .display(Display::Flex)
            .visibility(Visibility::Inherited);
    }

    if !slider.config().show_current {
        style_builder
            .switch_target(Slider::READOUT_CONTAINER)
            .display(Display::None)
            .visibility(Visibility::Hidden);
    } else {
        style_builder
            .switch_target(Slider::READOUT_CONTAINER)
            .display(Display::Flex)
            .visibility(Visibility::Inherited);
    }

    style_builder
        .switch_target(Slider::READOUT)
        .sized_font(SizedFont {
            font: "embedded://bevy_cobweb_ui/fonts/FiraSans-Regular.ttf".into(),
            size: 25.0,
        })
        .font_color(Color::WHITE);

    style_builder
        .switch_target(Slider::BAR)
        .border(UiRect::px(2., 2.0, 2., 2.0))
        .background_color(Color::Hsla(Hsla {
            hue: 34.0,
            saturation: 0.63,
            lightness: 0.55,
            alpha: 1.0,
        }))
        .border_color(Color::Hsla(Hsla {
            hue: 34.0,
            saturation: 0.55,
            lightness: 0.1,
            alpha: 1.0,
        }))
        .border_radius(BorderRadius::all(Val::Px(3.0)));

    style_builder
        .switch_context(Slider::HANDLE, None)
        .size(Val::Px(26.0))
        .border(UiRect::all(Val::Px(2.0)))
        .border_color(Color::Hsla(Hsla {
            hue: 34.0,
            saturation: 0.55,
            lightness: 0.1,
            alpha: 1.0,
        }))
        .border_radius(BorderRadius::all(Val::Px(13.0)))
        .animated()
        .background_color(AnimatedVals {
            idle: Color::Hsla(Hsla { hue: 34.0, saturation: 0.63, lightness: 0.55, alpha: 1.0 }),
            hover: Color::Hsla(Hsla { hue: 34.0, saturation: 0.7, lightness: 0.45, alpha: 1.0 }).into(),
            ..default()
        })
        .copy_from(theme_data.interaction_animation);
}

//-------------------------------------------------------------------------------------------------------------------

fn adjust_sickle_slider_theme(ui: &mut EntityCommands)
{
    let adjusted_theme = PseudoTheme::deferred_context(None, adjusted_slider_style);
    ui.insert(Theme::new(vec![adjusted_theme]));
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
        let mut ui = l.slider(SliderConfig::horizontal(None, 0.0, 100.0, 100.0, true));
        let mut n =
            ui.on_event::<SliderChanged>()
                .r(|event: EntityEvent<SliderChanged>, sliders: Query<&Slider>| {
                    let _slider = sliders.get(event.entity()).unwrap();

                    // NOT IMPLEMENTED: Adjust app's audio settings with slider value.
                });
        adjust_sickle_slider_theme(&mut n);
    });

    l.edit("vsync", |l| {
        // Radio buttons: bevy_cobweb_ui built-in widget.
        let manager_entity = RadioButtonManager::insert(l.deref_mut());
        l.edit("options", |l| {
            let button_loc = l.path().file.e("settings_radio_button");

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
        // TODO: Overwrite default styling. The dropdown styling is about 3x larger than the slider styling, so
        // for succinctness we did not override it here.
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
    file: &SceneFile,
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
        l.load_scene_and_edit(file + page_scene, |l| {
            page_entity = l.id();
            l.apply(DisplayControl::Hide);

            // Add custom logic to the page.
            (page_content_fn)(l);
        });
    });

    // Add button.
    // - We toggle content visibility on select/deselect.
    let button_entity = RadioButtonBuilder::custom_with_text(file + "menu_option_button", button_text)
        .localized()
        .build(manager_entity, l.deref_mut())
        .on_select(move |mut c: Commands| {
            c.entity(page_entity).apply(DisplayControl::Display);
        })
        .on_deselect(move |mut c: Commands| {
            c.entity(page_entity).apply(DisplayControl::Hide);
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
    let file = &SceneFile::new("main.caf.json");
    let scene = file + "menu_scene";

    c.ui_root().load_scene_and_edit(&mut s, scene, |l| {
        l.edit("menu::options", |l| {
            RadioButtonManager::insert(l.deref_mut());
            add_menu_option(
                l,
                file,
                "content",
                "menu-option-home",
                "home_page",
                build_home_page_content,
                true,
            );
            add_menu_option(
                l,
                file,
                "content",
                "menu-option-play",
                "play_page",
                build_play_page_content,
                false,
            );
            add_menu_option(
                l,
                file,
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

 */

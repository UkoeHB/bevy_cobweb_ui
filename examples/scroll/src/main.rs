//! Demonstrates the built-in scroll widget.

use std::time::Duration;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::builtin::widgets::scroll::MouseScroll;
use bevy_cobweb_ui::builtin::widgets::slider::SliderValue;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::*;
use smol_str::SmolStr;

//-------------------------------------------------------------------------------------------------------------------

const WIDE_PARAM: PseudoState = PseudoState::Custom(SmolStr::new_static("Wide"));
const TALL_PARAM: PseudoState = PseudoState::Custom(SmolStr::new_static("Tall"));
const IS_SCROLLING_PARAM: PseudoState = PseudoState::Custom(SmolStr::new_static("IsScrolling"));
const HOVER_ACTIVATED_PARAM: PseudoState = PseudoState::Custom(SmolStr::new_static("HoverActivated"));
const SUBLIME_SHADOW_FADE_PX: f32 = 30.;
const FIREFOX_FADEOUT_TIMER: Duration = Duration::from_millis(650);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct SublimeShadow;

/// Hack to refresh the shadow entity every tick.
fn ping_shadow_entity(mut c: Commands, q: Query<Entity, With<SublimeShadow>>)
{
    for entity in q.iter() {
        c.react().entity_event(entity, ());
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct FirefoxTimer(Timer);

/// Refreshes timers for the Firefox-like scrollbar, which needs to fade away after a delay.
fn check_firefox_timer(
    time: Res<Time>,
    mut c: Commands,
    ps: PseudoStateParam,
    mut timer: Query<(Entity, &Parent, &mut FirefoxTimer)>,
)
{
    let Ok((scrollbar_entity, gutter_entity, mut timer)) = timer.get_single_mut() else { return };
    timer.0.tick(time.delta());

    if timer.0.finished() {
        c.entity(scrollbar_entity).remove::<FirefoxTimer>();
        c.react().entity_event(scrollbar_entity, Disable);
        ps.try_remove(&mut c, **gutter_entity, IS_SCROLLING_PARAM.clone());
        ps.try_remove(&mut c, **gutter_entity, HOVER_ACTIVATED_PARAM.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn add_blob(l: &mut LoadedSceneUi, scene: &str)
{
    l.load_scene_and_edit(("main.cob", scene), |l| {
        let id = l.id();
        l.on_pressed(move |mut c: Commands, ps: PseudoStateParam| {
            // The blob alternates between 'tall', 'tall + wide' and 'none'.
            if !ps.entity_has(id, TALL_PARAM.clone()) {
                ps.try_insert(&mut c, id, TALL_PARAM.clone());
                return;
            }
            if !ps.entity_has(id, WIDE_PARAM.clone()) {
                ps.try_insert(&mut c, id, WIDE_PARAM.clone());
                return;
            }
            ps.try_remove(&mut c, id, WIDE_PARAM.clone());
            ps.try_remove(&mut c, id, TALL_PARAM.clone());
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    c.spawn(Camera2d);
    let file = SceneFile::new("main.cob");
    c.ui_root()
        .load_scene_and_edit(&file + "scene", &mut s, |l| {
            l.edit("view::shim::row1", |l| {
                l.load_scene_and_edit(&file + "basic", |l| {
                    let mut content = l.get("scroll::view::shim");
                    add_blob(&mut content, "blob");
                });

                l.load_scene_and_edit(&file + "overlay", |l| {
                    let mut content = l.get("scroll::view_shim::view::shim");
                    add_blob(&mut content, "blob");
                });
            });

            l.edit("view::shim::row2", |l| {
                l.load_scene_and_edit(&file + "inset", |l| {
                    let mut content = l.get("scroll::view_shim::view::shim");
                    add_blob(&mut content, "blob");
                });

                l.load_scene_and_edit(&file + "sublime", |l| {
                    let mut content = l.get("scroll::view_shim::view::shim");
                    add_blob(&mut content, "blob_sublime");

                    // Shadow visibility is affected by scroll value via PropagateOpacity.
                    let view_entity = l.get_entity("scroll::view_shim::view").unwrap();
                    let content_entity = l.get_entity("scroll::view_shim::view::shim").unwrap();
                    let bar_entity = l.get_entity("scroll::horizontal::bar").unwrap();
                    l.edit("scroll::view_shim::vertical::shadow_shim", |l| {
                        l.insert(SublimeShadow);
                        let shadow_id = l.id();
                        l.update_on(
                            entity_event::<()>(shadow_id),
                            move |//
                                    id: UpdateId,
                                    mut c: Commands,
                                    vals: Reactive<SliderValue>,
                                    nodes: Query<&ComputedNode>//
                                | {
                                let val = vals.get(bar_entity)?.single().result()?;
                                let view_node = nodes.get(view_entity)?;
                                let content_node = nodes.get(content_entity)?;
                                let scrollable_distance =
                                    (content_node.size() - view_node.size()).max(Vec2::default());
                                let unscrolled_distance = (1. - val) * scrollable_distance.x;
                                let opacity = (unscrolled_distance / SUBLIME_SHADOW_FADE_PX).clamp(0., 1.);

                                let mut ec = c.get_entity(*id).result()?;
                                ec.insert(PropagateOpacity(opacity));

                                OK
                            },
                        );
                    });
                });
            });

            l.edit("bar_shim::gutter::vertical", |l| {
                let id = l.id();
                l.on_event::<MouseScroll>().r(
                    move |//
                        mut c: Commands,
                        ps: PseudoStateParam,
                        mut q: Query<(&Parent, Option<&mut FirefoxTimer>, &Interaction)>,//
                    | {
                        let (gutter_entity, maybe_timer, interaction) = q.get_mut(id)?;
                        if let Some(mut timer) = maybe_timer {
                            timer.0.reset();
                        } else {
                            let gutter = **gutter_entity;
                            c.entity(id).insert(FirefoxTimer(Timer::new(FIREFOX_FADEOUT_TIMER, TimerMode::Once)));
                            c.react().entity_event(id, Enable);
                            ps.try_insert(&mut c, gutter, IS_SCROLLING_PARAM.clone());
                            // This is a bit of a hack to handle enabling the entity since we don't have a good
                            // strategy for interactions on enable boundaries yet.
                            match *interaction {
                                Interaction::None => (),
                                Interaction::Hovered => {
                                    c.react().entity_event(id, PointerEnter);
                                }
                                Interaction::Pressed => {
                                    c.react().entity_event(id, Pressed);
                                }
                            }
                        }
                        OK
                    }
                );
                // These don't activate unless in 'IS_SCROLLING' because we only enable the entity in that
                // state.
                l.on_pointer_enter(
                    move |//
                        mut c: Commands,
                        ps: PseudoStateParam,
                        mut q: Query<(&Parent, Option<&mut FirefoxTimer>)>,//
                    | {
                        let (gutter_entity, maybe_timer) = q.get_mut(id)?;
                        if let Some(mut timer) = maybe_timer {
                            timer.0.pause();
                        }
                        ps.try_insert(&mut c, **gutter_entity, HOVER_ACTIVATED_PARAM.clone());
                        OK
                    },
                );
                // Add this to handle very fast press that bypasses hover.
                l.on_pressed(
                    move |//
                        mut c: Commands,
                        ps: PseudoStateParam,
                        mut q: Query<(&Parent, Option<&mut FirefoxTimer>)>,//
                    | {
                        let (gutter_entity, maybe_timer) = q.get_mut(id)?;
                        if let Some(mut timer) = maybe_timer {
                            timer.0.pause();
                        }
                        ps.try_insert(&mut c, **gutter_entity, HOVER_ACTIVATED_PARAM.clone());
                        OK
                    },
                );
                l.on_pointer_leave(move |mut q: Query<&mut FirefoxTimer>| {
                    let mut timer = q.get_mut(id)?;
                    timer.0.unpause();
                    timer.0.reset();
                    DONE
                });
                l.on_press_canceled(move |mut q: Query<&mut FirefoxTimer>| {
                    let mut timer = q.get_mut(id)?;
                    timer.0.unpause();
                    timer.0.reset();
                    DONE
                });
            });
        });
}

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                window_theme: Some(bevy::window::WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CobwebUiPlugin)
        .load("main.cob")
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .add_systems(Update, (ping_shadow_entity, check_firefox_timer))
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

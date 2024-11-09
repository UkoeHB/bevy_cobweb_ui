//! Demonstrates using `PropagateOpacity` to show/hide help text. Also demonstrates opacity layering.
//!
//! Note that this is a fairly unsophisticated help text. It's inserted in-line to the node tree, whereas a mature
//! widget would have precise positioning and reusable styling, would be able to adjust its position based
//! on available screen-space, may adjust its position in response to detected cursor size (when possible),
//! and may have a little arrow pointing from the text to the source.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle_ext::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>) {
    let scene = SceneRef::new("main.caf", "scene");
    c.ui_root().load_scene(&mut s, scene);
}

//-------------------------------------------------------------------------------------------------------------------

fn main() {
    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..default() }),
            ..default()
        }))
        .add_plugins(CobwebUiPlugin)
        .load("main.caf")
        .add_systems(PreStartup, |mut c: Commands| {
            c.spawn(Camera2d);
        })
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

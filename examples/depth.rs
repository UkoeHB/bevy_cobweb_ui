//! Visual test for depth controls.

//local shortcuts
use bevy_cobweb_ui::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Blocks are stacked from left to right. Stacking is 'natural' based on registration order.
fn add_plain_blocks(ui: &mut UiCommands, parent: Entity, quantity: usize, offset: f32, height: f32)
{
    let section = ui.build((Parent(parent), Dims::Overlay)).id();
    for i in 0..quantity
    {
        let color = match i % 3
        {
            0 => Color::BLACK,
            1 => Color::DARK_GRAY,
            _ => Color::GRAY,
        };
        ui.build((
                Block{ color },
                Parent(section),
                Position::topleft()
                    .percent(Vec2{ x: (i * (100 / quantity)) as f32 - offset/4., y: offset + (i * 3) as f32 }),
                Dims::Percent(Vec2{ x: (2 * (100 / quantity)) as f32, y: height })
            ));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Blocks are stacked from right to left. Stacking is 'natural' based on registration order.
///
/// These should be on top of the previous set of blocks.
fn add_reverse_blocks(ui: &mut UiCommands, parent: Entity, quantity: usize, offset: f32, height: f32)
{
    let section = ui.build((Parent(parent), Dims::Overlay)).id();
    for i in (0..quantity).rev()
    {
        let color = match i % 3
        {
            0 => Color::DARK_GREEN,
            1 => Color::MIDNIGHT_BLUE,
            _ => Color::MAROON,
        };
        ui.build((
                Block{ color },
                Parent(section),
                Position::topleft()
                    .percent(Vec2{ x: (i * (100 / quantity)) as f32 - offset/4., y: offset + (i * 3) as f32 }),
                Dims::Percent(Vec2{ x: (2 * (100 / quantity)) as f32, y: height })
            ));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Blocks are stacked from left to right. Stacking is ZLevel-controled.
///
/// These should be on top of the previous set of blocks.
fn add_reverse_blocks_zlevel(ui: &mut UiCommands, parent: Entity, quantity: usize, offset: f32, height: f32)
{
    let section = ui.build((Parent(parent), Dims::Overlay)).id();
    for i in 0..quantity
    {
        let color = match i % 3
        {
            0 => Color::BLACK,
            1 => Color::DARK_GRAY,
            _ => Color::GRAY,
        };
        ui.build((
                Block{ color },
                Parent(section),
                Position::topleft()
                    .percent(Vec2{ x: (i * (100 / quantity)) as f32 - offset/4., y: offset + (i * 3) as f32 }),
                Dims::Percent(Vec2{ x: (2 * (100 / quantity)) as f32, y: height })
            ))
            .insert(ZLevel((quantity - i) as i32));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Blocks are stacked alternating. Stacking is ZLevel-controled.
///
/// These should be on top of the previous set of blocks.
fn add_alternating_blocks(ui: &mut UiCommands, parent: Entity, quantity: usize, offset: f32, height: f32)
{
    let section = ui.build((Parent(parent), Dims::Overlay)).id();
    for i in 0..quantity
    {
        let color = match i % 3
        {
            0 => Color::DARK_GREEN,
            1 => Color::MIDNIGHT_BLUE,
            _ => Color::MAROON,
        };
        ui.build((
                Block{ color },
                Parent(section),
                Position::topleft()
                    .percent(Vec2{ x: (i * (100 / quantity)) as f32 - offset/4., y: offset + (i * 3) as f32 }),
                Dims::Percent(Vec2{ x: (2 * (100 / quantity)) as f32, y: height })
            ))
            .insert(ZLevel((i % 2) as i32));
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut ui: UiCommands, camera: Query<Entity, (With<Camera>, With<UiRoot>)>)
{
    let root = ui.build((InCamera(camera.single()), Dims::Overlay)).id();

    add_plain_blocks(&mut ui, root, 5, 0., 20.);
    add_reverse_blocks(&mut ui, root, 5, 10., 20.);
    add_reverse_blocks_zlevel(&mut ui, root, 5, 20., 20.);
    add_alternating_blocks(&mut ui, root, 5, 30., 20.);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    commands.spawn(UiCamera2D::default());
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(
            bevy::DefaultPlugins.set(
                WindowPlugin{
                    primary_window: Some(Window{ window_theme: Some(WindowTheme::Dark), ..Default::default() }),
                    ..Default::default()
                }
            )
        )
        .add_plugins(CobwebUiPlugin)
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

//! Demonstrates library primitives and features.

//local shortcuts
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::window::WindowTheme;
use sickle_ui::ui_builder::*;

//standard shortcuts

/*
//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the justification of a node based on arrow-key inputs.
fn handle_keyboard_input_for_node(
    mut cache : Local<Option<KeyboardInput>>,
    mut event : SystemEvent<UiEvent<KeyboardInput>>,
    mut rc    : ReactCommands,
    mut nodes : Query<&mut React<Position>>
){
    let Some(event) = event.take() else { return; };
    let Ok(mut position) = nodes.get_mut(event.node) else { return; };

    let mut check_cache = |input: KeyboardInput| -> bool
    {
        if cache.is_none()
        {
            *cache = Some(input);
            return true;
        }
        let cached = cache.as_ref().unwrap().clone();
        *cache = Some(input.clone());

        // Return true when a key is just pressed.
        if input.state == ButtonState::Released { return false; }
        if input.logical_key != cached.logical_key { return true; }
        if cached.state == ButtonState::Pressed { return false; }
        true
    };

    match event.event.logical_key
    {
        Key::ArrowDown =>
        {
            if !check_cache(event.event) { return; }
            match position.y_justify
            {
                Justify::Min    => { position.get_mut(&mut rc).y_justify = Justify::Center; }
                Justify::Center => { position.get_mut(&mut rc).y_justify = Justify::Max; }
                Justify::Max    => (),
            }
        }
        Key::ArrowUp =>
        {
            if !check_cache(event.event) { return; }
            match position.y_justify
            {
                Justify::Min    => (),
                Justify::Center => { position.get_mut(&mut rc).y_justify = Justify::Min; }
                Justify::Max    => { position.get_mut(&mut rc).y_justify = Justify::Center; }
            }
        }
        Key::ArrowLeft =>
        {
            if !check_cache(event.event) { return; }
            match position.x_justify
            {
                Justify::Min    => (),
                Justify::Center => { position.get_mut(&mut rc).x_justify = Justify::Min; }
                Justify::Max    => { position.get_mut(&mut rc).x_justify = Justify::Center; }
            }
        }
        Key::ArrowRight =>
        {
            if !check_cache(event.event) { return; }
            match position.x_justify
            {
                Justify::Min    => { position.get_mut(&mut rc).x_justify = Justify::Center; }
                Justify::Center => { position.get_mut(&mut rc).x_justify = Justify::Max; }
                Justify::Max    => (),
            }
        }
        _ => (),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_blocks(ui: &mut UiCommands, path: &StyleRef, parent: Entity)
{
    // Build a block in the center of its parent.
    let outer_block = path.extend("outer_block");
    let outer = ui.build((
            Block{ color: Color::BLACK },
            Parent(parent),
            Justified::load(&outer_block),
            Dims::load(&outer_block),
        ))
        .id();

    // Build a block inside the other block.
    let inner_block = outer_block.extend("inner_block");
    let inner = ui.build((
            Block{ color: Color::DARK_GRAY },
            Parent(outer),
            Justified::load(&inner_block),
            Dims::load(&inner_block),
            On::<KeyboardInput>::new(handle_keyboard_input_for_node),  //todo: OnBroadcast
        ))
        .id();

    // Build another block inside the previous.
    let final_block = inner_block.extend("final_block");
    ui.build((
            Block::load(&final_block),
            Parent(inner),
            Justified::load(&final_block),
            Dims::load(&final_block),
            On::<KeyboardInput>::new(handle_keyboard_input_for_node),
        ));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn add_images(ui: &mut UiCommands, path: &StyleRef, parent: Entity)
{
    // Top left image
    ui.build((
            BasicImage::new("examples/green_rectangle.png"),
            Parent(parent),
            Position::topleft(),
            Dims::Percent(Vec2{ x: 20.0, y: 20.0 }),
        ));

    // Top right image
    let upper_right_img = path.extend("upper_right_img");
    ui.build((
            BasicImage::load(&upper_right_img),
            Parent(parent),
            Justified::load(&upper_right_img),
            Dims::load(&upper_right_img),
        ));
}
*/
//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut cmds: Commands)
{
    let file = StyleRef::from_file("examples/sample.style.json");

    cmds.ui_builder(UiRoot).load(file.e("root"), |root, path| {
        root.load(path.e("a"), |_n, _p|{})
            .load(path.e("b"), |_n, _p|{})
            .on_event::<u32>().r(||{})
            .update_on(despawn(Entity::PLACEHOLDER), |id| move || { println!("success {:?}", id); });
    });
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    commands.spawn(Camera2dBundle{
        transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() },
        ..default()
    });
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
        .add_style_sheet("examples/sample.style.json")
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_systems(PreStartup, setup)
        .add_systems(Startup, build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

//local shortcuts
use bevy_cobweb_ui::prelude::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the justification of a node based on arrow-key inputs.
fn handle_keyboard_input_for_node(
    mut cache : Local<Option<KeyboardInput>>,
    mut event : SystemEvent<UiEvent<KeyboardInput>>,
    mut rc    : ReactCommands,
    mut nodes : Query<&mut React<Layout>>
){
    let Some(event) = event.take() else { return; };
    let Ok(mut layout) = nodes.get_mut(event.node) else { return; };

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
            match layout.y_justify
            {
                Justify::Min    => { layout.get_mut(&mut rc).y_justify = Justify::Center; }
                Justify::Center => { layout.get_mut(&mut rc).y_justify = Justify::Max; }
                Justify::Max    => (),
            }
        }
        Key::ArrowUp =>
        {
            if !check_cache(event.event) { return; }
            match layout.y_justify
            {
                Justify::Min    => (),
                Justify::Center => { layout.get_mut(&mut rc).y_justify = Justify::Min; }
                Justify::Max    => { layout.get_mut(&mut rc).y_justify = Justify::Center; }
            }
        }
        Key::ArrowLeft =>
        {
            if !check_cache(event.event) { return; }
            match layout.x_justify
            {
                Justify::Min    => (),
                Justify::Center => { layout.get_mut(&mut rc).x_justify = Justify::Min; }
                Justify::Max    => { layout.get_mut(&mut rc).x_justify = Justify::Center; }
            }
        }
        Key::ArrowRight =>
        {
            if !check_cache(event.event) { return; }
            match layout.x_justify
            {
                Justify::Min    => { layout.get_mut(&mut rc).x_justify = Justify::Center; }
                Justify::Center => { layout.get_mut(&mut rc).x_justify = Justify::Max; }
                Justify::Max    => (),
            }
        }
        _ => (),
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut uc: UiCommands, camera: Query<Entity, With<Camera>>)
{
    // Build a block in the center of the camera.
    let center = uc.build((
            Block{ color: Color::BLACK },
            InCamera(camera.single()),
            JustifiedLayout::Center(Size::from(Relative::new(50., 50.))),
        ))
        .id();

    // Build a block inside the other block.
    let inner = uc.build((
            Block{ color: Color::DARK_GRAY },
            Parent(center),
            JustifiedLayout::Center(Size::from(Relative::new(25., 25.))),
            On::<KeyboardInput>::new(handle_keyboard_input_for_node)  //todo: OnBroadcast
        ))
        .id();

    // Build another block inside the previous.
    let file = StyleRef::from_file("examples/sample.style.json");
    let style = file.extend("outer_block::inner_block::final_block");
    uc.build((
            Block::load(&style),
            Parent(inner),
            JustifiedLayout::Center(Size::from(Relative::new(50., 50.))),
        ));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    // prepare 2D camera
    commands.spawn(
            Camera2dBundle{ transform: Transform{ translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() }, ..default() }
        )
        .insert(InheritedVisibility::VISIBLE);
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

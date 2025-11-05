use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

const SLIDER_ZOOM_PSEUDO_STATE: PseudoState = PseudoState::Custom(SmolStr::new_static("SliderZoom"));
const SLIDER_ZOOM_ATTR: &'static str = "sliderzoom";

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, PartialEq, Default, Debug, Clone)]
struct SliderZoom(SliderValue);

impl SliderZoom
{
    fn apply_zoom(
        In((entity, mut val)): In<(Entity, SliderValue)>,
        mut c: Commands,
        mut r: ReactiveMut<SliderValue>,
    )
    {
        val.normalize();
        r.set_if_neq(&mut c, entity, val);
    }
}

impl Instruction for SliderZoom
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        world.syscall((entity, self.0), Self::apply_zoom);
    }

    /// Reverting SliderValue is handled by Slider::revert.
    fn revert(_: Entity, _: &mut World) {}
}

impl StaticAttribute for SliderZoom
{
    type Value = SliderValue;

    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}

impl AnimatedAttribute for SliderZoom
{
    fn get_value(entity: Entity, world: &World) -> Option<SliderValue>
    {
        let val = world.get::<React<SliderValue>>(entity)?;
        Some(val.get().clone())
    }

    fn extract(
        entity: Entity,
        world: &mut World,
        ref_vals: &AnimatedVals<Self::Value>,
        state: &AnimationState,
    ) -> Self::Value
    {
        let val = ref_vals.to_value(state);

        // Clean up state when done zooming.
        // - This prepares us for the next zoom, which requires 'entering' the SliderZoom state.
        let Ok(mut emut) = world.get_entity_mut(entity) else { return val };
        if *state.result() == AnimationResult::Hold(InteractionStyle::Idle) {
            emut.remove_pseudo_state(SLIDER_ZOOM_PSEUDO_STATE.clone());
        }

        // Update the slider value.
        val
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default)]
struct SliderDragReference
{
    /// If set, then drag events should be ignored. Used to manage drags on inert bars where we only want drags
    /// on handles to be respected.
    invalid_press: bool,
    /// Logical offset between handle and pointer during a drag. Add this to the pointer to get the target
    /// handle position.
    ///
    /// Used when drag starts on top of the handle, so the pointer needs to be offset from the handle center
    /// during drag.
    offset: Vec2,
}

//-------------------------------------------------------------------------------------------------------------------

// NOTE: If the slider bar moves during a slider handle drag, then the handle will appear to glitch away from the
// pointer until more drag is applied.
// One solution would be to visually adjust the handle position between UiSystems::Layout and PropagateTransforms,
// then update the value in Update in the next tick (if it changed due to handle adjustment). The visual position
// would always be correct, however there would be a one-frame delay in slider values.
#[derive(Component)]
struct ComputedSlider
{
    config: Slider,

    /// Drag reference for the latest drag event.
    drag_reference: SliderDragReference,

    /// Cached reactor ids for cleanup on instruction revert.
    press_observer: Entity,
    drag_observer: Entity,
}

impl ComputedSlider
{
    fn revoke(self, world: &mut World)
    {
        world.despawn(self.press_observer);
        world.despawn(self.drag_observer);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn get_camera_scale_factor(
    ui_camera: &DefaultUiCamera,
    cameras: &Query<&Camera>,
    maybe_slider_camera: Option<&UiTargetCamera>,
) -> Option<f32>
{
    let camera_entity = maybe_slider_camera
        .map(|t| t.entity())
        .or_else(|| ui_camera.get())?;
    let Ok(camera) = cameras.get(camera_entity) else { return None };
    Some(camera.target_scaling_factor().unwrap_or(1.))
}

//-------------------------------------------------------------------------------------------------------------------

/// Computes the 'standard' value that will cause the handle to be centered over a target position in physical
/// coordinates.
fn compute_value_for_target_position(
    mut target_position_physical: Vec2,
    slider_transform: &UiGlobalTransform,
    bar_size: Vec2,
    handle_size: Vec2,
    axis: SliderAxis,
) -> SliderValue
{
    let mut bar_location = slider_transform.translation;

    // Invert y-axis to point up.
    target_position_physical.y = -target_position_physical.y;
    bar_location.y = -bar_location.y;

    let bar_bottom = bar_location - (bar_size / 2.); // Physical bottom.
    let bar_action_size = (bar_size - handle_size).max(Vec2::splat(0.)); // Size of bar where slider values are applied.
    let adjusted_target = target_position_physical - (handle_size / 2.); // Adjusted down to center the handle.
    let diff = (adjusted_target - bar_bottom).max(Vec2::splat(0.)); // Distance from target to bottom.
    let mut computed_val = Vec2::default();
    if bar_action_size.x > 0. {
        computed_val.x = (diff.x / bar_action_size.x).min(1.);
    }
    if bar_action_size.y > 0. {
        computed_val.y = (diff.y / bar_action_size.y).min(1.);
    }

    match axis {
        SliderAxis::X => SliderValue::Single(computed_val.x),
        SliderAxis::Y => SliderValue::Single(computed_val.y),
        SliderAxis::Planar => SliderValue::Planar(computed_val),
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn slider_bar_ptr_down(
    mut event: On<Pointer<Press>>,
    mut iter_children: ResMut<IterChildren>,
    mut c: Commands,
    ps: PseudoStateParam,
    cameras: Query<&Camera>,
    ui_camera: DefaultUiCamera,
    mut sliders: Query<(
        &mut ComputedSlider,
        &mut React<SliderValue>,
        Option<&mut NodeAttributes>,
        &ComputedNode,
        &UiGlobalTransform,
        &Children,
        Option<&UiTargetCamera>,
    )>,
    children_query: Query<&Children>,
    handles: Query<(Entity, &ComputedNode, &UiGlobalTransform), (With<SliderHandle>, Without<ComputedSlider>)>,
)
{
    // Look up the slider and its handle.
    let slider_entity = event.entity;
    let Ok((
        mut slider,
        mut slider_value,
        maybe_attrs,
        slider_node,
        slider_transform,
        slider_children,
        maybe_slider_camera,
    )) = sliders.get_mut(slider_entity)
    else {
        return;
    };

    // Prevent propagation, we are consuming this event.
    event.propagate(false);

    let maybe_handle =
        iter_children.search_descendants(slider_children, &children_query, |child| handles.get(child).ok());

    let Some((handle_entity, handle_node, handle_transform)) = maybe_handle else {
        tracing::warn!("failed finding a SliderHandle on a descendant of Slider entity {:?}", slider_entity);
        return;
    };

    // Get slider bar and handle sizes (in physical pixels).
    let bar_size = slider_node.size();
    let handle_size = handle_node.size();

    // Get camera scale factor and pointer physical position.
    let Some(camera_scale_factor) = get_camera_scale_factor(&ui_camera, &cameras, maybe_slider_camera) else {
        return;
    };
    let pointer_position = event.event().pointer_location.position;
    let pointer_position_physical = pointer_position * camera_scale_factor;

    // Check if pointer targets the handle or any of its descendants.
    let pointer_target = event.original_event_target();
    let targets_handle = iter_children
        .search(handle_entity, &children_query, |entity| {
            if entity == pointer_target {
                Some(())
            } else {
                None
            }
        })
        .is_some();

    // If the point targets the handle, we initiate drag.
    if targets_handle {
        // Calculate logical offset between pointer and center of handle.
        let handle_position_logical = handle_transform.translation / camera_scale_factor.max(0.0001);
        let offset = handle_position_logical - pointer_position;

        slider.drag_reference = SliderDragReference { invalid_press: false, offset };
        return;
    }

    // Inert bars cannot be pressed.
    if slider.config.bar_press == SliderPress::Inert {
        slider.drag_reference.invalid_press = true;
        return;
    }

    // Compute value.
    let standard_val = compute_value_for_target_position(
        pointer_position_physical,
        slider_transform,
        bar_size,
        handle_size,
        slider.config.axis,
    );

    let target_val = slider
        .config
        .direction
        .flip_direction(standard_val, slider.config.axis);

    // Update drag reference.
    slider.drag_reference = SliderDragReference { invalid_press: false, offset: Vec2::default() };

    // Update value.
    match slider.config.bar_press {
        SliderPress::Jump => {
            React::set_if_neq(&mut slider_value, &mut c, target_val);
        }
        SliderPress::Animate(_) => {
            // If adding state fails, we are already in this state. The animation framework does not support
            // changing reference values in the middle of an animation, so we fall back to 'jump to position'.
            if !ps.try_insert(&mut c, slider_entity, SLIDER_ZOOM_PSEUDO_STATE) {
                ps.try_remove(&mut c, slider_entity, SLIDER_ZOOM_PSEUDO_STATE);
                React::set_if_neq(&mut slider_value, &mut c, target_val);
            } else if let Some(zoom) = maybe_attrs.and_then(|a| {
                a.into_inner()
                    .animated_vals_mut::<SliderZoom>(SLIDER_ZOOM_ATTR)
            }) {
                zoom.idle = target_val;
            }
        }
        SliderPress::Inert => unreachable!(),
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn slider_bar_drag(
    mut event: On<Pointer<Drag>>,
    mut iter_children: ResMut<IterChildren>,
    mut c: Commands,
    ps: PseudoStateParam,
    cameras: Query<&Camera>,
    ui_camera: DefaultUiCamera,
    mut sliders: Query<(
        &ComputedSlider,
        &mut React<SliderValue>,
        &ComputedNode,
        &UiGlobalTransform,
        &Children,
        Option<&UiTargetCamera>,
    )>,
    children_query: Query<&Children>,
    handles: Query<&ComputedNode, (With<SliderHandle>, Without<ComputedSlider>)>,
)
{
    // Look up the slider.
    let slider_entity = event.entity;
    let Ok((slider, mut slider_value, slider_node, slider_transform, slider_children, maybe_slider_camera)) =
        sliders.get_mut(slider_entity)
    else {
        return;
    };

    // Prevent propagation, we are consuming this event.
    event.propagate(false);

    // Prevent no-movement drags from doing anything. There is a bevy bug where pointer-up causes a drag event even
    // if the cursor didn't move.
    if event.event().distance == Vec2::default() {
        return;
    }

    // Drags require a valid press event.
    if slider.drag_reference.invalid_press {
        return;
    }

    // Look up the handle.
    let maybe_handle =
        iter_children.search_descendants(slider_children, &children_query, |child| handles.get(child).ok());

    let Some(handle_node) = maybe_handle else {
        tracing::warn!("failed finding a SliderHandle on a descendant of Slider entity {:?}", slider_entity);
        return;
    };

    // Get slider bar and handle sizes (in physical pixels).
    let bar_size = slider_node.size();
    let handle_size = handle_node.size();

    // Correct the pointer location based on where we want the handle to go relative to the pointer.
    let pointer_position = event.event().pointer_location.position;
    let target_position_corrected = pointer_position + slider.drag_reference.offset;

    // Get camera scale factor and target physical position.
    let Some(camera_scale_factor) = get_camera_scale_factor(&ui_camera, &cameras, maybe_slider_camera) else {
        return;
    };
    let target_position_physical = target_position_corrected * camera_scale_factor;

    // Compute value.
    let standard_val = compute_value_for_target_position(
        target_position_physical,
        slider_transform,
        bar_size,
        handle_size,
        slider.config.axis,
    );

    let target_val = slider
        .config
        .direction
        .flip_direction(standard_val, slider.config.axis);

    // Update value.
    React::set_if_neq(&mut slider_value, &mut c, target_val);

    // Cleanup zoom effect.
    if matches!(slider.config.bar_press, SliderPress::Animate(_)) {
        ps.try_remove(&mut c, slider_entity, SLIDER_ZOOM_PSEUDO_STATE);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn update_slider_handle_positions(
    mut iter_children: ResMut<IterChildren>,
    mut sliders: Query<(&ComputedSlider, &React<SliderValue>, &Node, &ComputedNode, &Children)>,
    children_q: Query<&Children>,
    handles: Query<(Entity, &ComputedNode), (With<SliderHandle>, Without<ComputedSlider>)>,
    mut transforms: Query<&mut UiGlobalTransform>,
)
{
    for (slider, slider_value, slider_node, slider_computed_node, children) in sliders.iter_mut() {
        // Skip sliders that won't be displayed.
        // - Note: ViewVisibility updates *after* TransformPropagate, so we can't use it here.
        if slider_node.display == Display::None {
            continue;
        }

        // Look up handle.
        let Some((handle_entity, handle_node)) =
            iter_children.search_descendants(children, &children_q, |c| handles.get(c).ok())
        else {
            continue;
        };

        let axis = slider.config.axis;

        // Get slider bar and handle sizes (in physical pixels).
        let bar_size = slider_computed_node.size();
        let handle_size = handle_node.size();
        let bar_action_size = (bar_size - handle_size).max(Vec2::splat(0.));

        // Get standardized current value.
        let mut value = slider_value.get().clone();
        value.normalize();
        let standard_val = slider.config.direction.flip_direction(value, axis);
        let val_vec2 = standard_val.to_vec2(axis);

        // Get transform offset between bar and handle.
        let mut val_pos = val_vec2 * bar_action_size;
        val_pos.y = -(val_pos.y - bar_action_size.y); // Correction because y-axis is down and handle defaults to top of bar.
        let transform_offset_corrected = match axis {
            SliderAxis::X => {
                let y_offset = (bar_size.y - handle_size.y) / 2.;
                val_pos.with_y(y_offset)
            }
            SliderAxis::Y => {
                let x_offset = (bar_size.x - handle_size.x) / 2.;
                val_pos.with_x(x_offset)
            }
            SliderAxis::Planar => val_pos,
        };

        // Update handle's position relative to the slider bar.
        // NOTE: This position adjustment may not be 'correct' if the handle isn't a direct child of the slider.
        update_handle_transform_recursive(handle_entity, transform_offset_corrected, &mut transforms, &children_q);
    }
}

fn update_handle_transform_recursive(
    entity: Entity,
    offset: Vec2,
    transforms: &mut Query<&mut UiGlobalTransform>,
    children_q: &Query<&Children>,
)
{
    let Ok(mut transform) = transforms.get_mut(entity) else { return };
    let mut temp = **transform;
    temp.translation += offset;
    *transform = temp.into();

    let Ok(children) = children_q.get(entity) else { return };
    for child in children.iter() {
        update_handle_transform_recursive(child, offset, transforms, children_q);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive component with a slider value.
///
/// Values are in the range `[0.0..1.0]`.
///
/// See [`Slider`].
#[derive(ReactComponent, Debug, Copy, Clone, PartialEq, Reflect)]
pub enum SliderValue
{
    Single(f32),
    /// The horizontal and vertical slider values for sliders with [`SliderAxis::Planar`].
    Planar(Vec2),
}

impl SliderValue
{
    /// Gets the value if it is `Self::Single`.
    pub fn single(&self) -> Option<f32>
    {
        match self {
            Self::Single(val) => Some(*val),
            Self::Planar(_) => None,
        }
    }

    /// Gets the value if it is `Self::Planar`.
    pub fn planar(&self) -> Option<Vec2>
    {
        match self {
            Self::Single(_) => None,
            Self::Planar(val) => Some(*val),
        }
    }

    /// Clamps the value to the range `[0.0..1.0]`.
    pub fn normalize(&mut self)
    {
        match self {
            Self::Single(v) => {
                *v = v.min(1.0).max(0.);
            }
            Self::Planar(v) => {
                v.x = v.x.min(1.0).max(0.);
                v.y = v.y.min(1.0).max(0.);
            }
        }
    }

    pub fn to_vec2(&self, axis: SliderAxis) -> Vec2
    {
        match axis {
            SliderAxis::X => match *self {
                Self::Single(v) => Vec2 { x: v, y: 0. },
                Self::Planar(Vec2 { x, y: _ }) => Vec2 { x, y: 0. },
            },
            SliderAxis::Y => match *self {
                Self::Single(v) => Vec2 { x: 0., y: v },
                Self::Planar(Vec2 { x: _, y }) => Vec2 { x: 0., y },
            },
            SliderAxis::Planar => match *self {
                Self::Single(v) => Vec2 { x: v, y: v },
                Self::Planar(v) => v,
            },
        }
    }
}

impl Default for SliderValue
{
    fn default() -> Self
    {
        Self::Single(0.)
    }
}

impl Lerp for SliderValue
{
    fn lerp(&self, to: Self, t: f32) -> Self
    {
        let mut res = match (*self, to) {
            (Self::Single(a), Self::Single(b)) => Self::Single(a.lerp(b, t)),
            (Self::Planar(a), Self::Planar(b)) => Self::Planar(a.lerp(b, t)),
            (Self::Single(a), Self::Planar(b)) => Self::Planar(Vec2::splat(a).lerp(b, t)),
            (Self::Planar(a), Self::Single(b)) => Self::Planar(a.lerp(Vec2::splat(b), t)),
        };
        res.normalize();
        res
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// The axis of a slider.
///
/// See [`Slider`].
#[derive(Reflect, Default, PartialEq, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum SliderAxis
{
    #[default]
    X,
    Y,
    /// The slider moves both horizontally and vertically (i.e. in a rectangle).
    Planar,
}

//-------------------------------------------------------------------------------------------------------------------

/// The direction of a slider's axis/axes.
///
/// See [`Slider`].
#[derive(Reflect, Default, PartialEq, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum SliderDirection
{
    /// - **Horizontal**: Left-to-right
    /// - **Vertical**: Bottom-to-top
    /// - **Planar**: Left-to-right and bottom-to-top
    #[default]
    Standard,
    /// - **Horizontal**: Right-to-left
    /// - **Vertical**: Top-to-bottom
    /// - **Planar**: Right-to-left and top-to-bottom
    Reverse,
    /// - **Horizontal**: `Self::Reverse`
    /// - **Vertical**: `Self::Standard`
    /// - **Planar**: Right-to-left and bottom-to-top
    ReverseHorizontal,
    /// - **Horizontal**: `Self::Standard`
    /// - **Vertical**: `Self::Reverse`
    /// - **Planar**: Left-to-right and top-to-bottom
    ReverseVertical,
}

impl SliderDirection
{
    /// Applies the slider direction to a value.
    ///
    /// If the value is 'standard' then it will be returned with the direction applied. If the value
    /// is 'directed', then the direction will be undone and it will be returned as a 'standard' value.
    pub fn flip_direction(&self, value: SliderValue, axis: SliderAxis) -> SliderValue
    {
        match self {
            Self::Standard => value,
            Self::Reverse => match value {
                SliderValue::Single(val) => SliderValue::Single(1. - val),
                SliderValue::Planar(val) => SliderValue::Planar(Vec2::splat(1.) - val),
            },
            Self::ReverseHorizontal => match value {
                SliderValue::Single(val) => match axis {
                    SliderAxis::X => SliderValue::Single(1. - val),
                    SliderAxis::Y => SliderValue::Single(val),
                    SliderAxis::Planar => SliderValue::Planar(Vec2::new(1. - val, val)),
                },
                SliderValue::Planar(Vec2 { x, y }) => SliderValue::Planar(Vec2::new(1. - x, y)),
            },
            Self::ReverseVertical => match value {
                SliderValue::Single(val) => match axis {
                    SliderAxis::X => SliderValue::Single(val),
                    SliderAxis::Y => SliderValue::Single(1. - val),
                    SliderAxis::Planar => SliderValue::Planar(Vec2::new(val, 1. - val)),
                },
                SliderValue::Planar(Vec2 { x, y }) => SliderValue::Planar(Vec2::new(x, 1. - y)),
            },
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Configuration for pressing a slider's bar.
///
/// See [`Slider`].
#[derive(Reflect, Default, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SliderPress
{
    /// Pressing the slider bar does nothing.
    Inert,
    /// Pressing the slider bar causes the handle to jump to the cursor.
    #[default]
    Jump,
    /// Pressing the slider bar causes the handle to move to the cursor using an animation.
    Animate(AnimationConfig),
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for setting up a slider widget.
///
/// This should be placed on the entity with the 'slider bar' of the slider.
///
/// Inserts a [`SliderValue`] reactive component to the entity. Also inserts an internal `ComputedSlider`
/// component.
///
/// The primary button of all pointers will be able to drag the slider handle and press the slider bar to move
/// the handle.
///
/// Use [`SliderHandle`] on the node that will own the slider handle.
#[derive(Reflect, Default, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Slider
{
    #[reflect(default)]
    pub axis: SliderAxis,
    #[reflect(default)]
    pub direction: SliderDirection,
    /// Configures the handle's behavior when pressing the slider bar.
    ///
    /// Defaults to [`SliderPress::Jump`].
    #[reflect(default)]
    pub bar_press: SliderPress,
    // TODO: consider configuring what pointers are allowed to drag the handle and press on the bar
    // TODO: how to allow 'cursor scroll' or e.g. arrow keys (with keyboard focus?) to move the slider handle?
    // - this may need to be added via higher-level abstractions
}

impl Instruction for Slider
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        let initial_slider_value = match self.axis {
            SliderAxis::X | SliderAxis::Y => SliderValue::Single(0.),
            SliderAxis::Planar => SliderValue::Planar(Vec2::default()),
        };

        let computed = emut.world_scope(|world| {
            // Set up animation for pressing the bar outside the handle.
            if let SliderPress::Animate(enter_idle_with) = self.bar_press.clone() {
                let animation = Animated::<SliderZoom> {
                    name: Some(SmolStr::new_static(SLIDER_ZOOM_ATTR)),
                    state: Some(SmallVec::from_elem(SLIDER_ZOOM_PSEUDO_STATE.clone(), 1)),
                    enter_idle_with: Some(enter_idle_with),
                    idle: SliderValue::default(), // We override the idle value as needed.
                    delete_on_entered: true,
                    ..default()
                };
                animation.apply(entity, world);
            }

            // Set up observers.
            let press_observer = world
                .spawn(Observer::new(slider_bar_ptr_down).with_entity(entity))
                .id();
            let drag_observer = world
                .spawn(Observer::new(slider_bar_drag).with_entity(entity))
                .id();

            ComputedSlider {
                config: self,
                drag_reference: SliderDragReference::default(),
                press_observer,
                drag_observer,
            }
        });

        emut.insert(computed);

        world.react(|rc| rc.insert(entity, initial_slider_value));
    }

    fn revert(entity: Entity, world: &mut World)
    {
        Animated::<SliderZoom>::revert(entity, world);

        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.remove::<React<SliderValue>>();
        emut.remove_pseudo_state(SLIDER_ZOOM_PSEUDO_STATE.clone());
        if let Some(computed) = emut.take::<ComputedSlider>() {
            computed.revoke(world);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component loadable for setting up a slider widget's handle.
///
/// The handle node should be absolutely-positioned (see [`AbsoluteNode`]).
///
/// One of the node's ancestors must have `ComputedSlider` (see [`Slider`]). It is recommended, but not required,
/// for the handle to be a direct child of the slider. The handle's `Transform` is automatically adjusted to
/// position it correctly relative to the slider, so if the handle isn't a direct child that calculation may be
/// off.
///
/// If the handle node has a width or height, then those dimensions will be respected by the slider bar. For
/// example if you have a vertical scrollbar, then the 'slider range' will equal the bar height minus the handle
/// height, and the 'current slider value' equals the distance between the bottom of the bar and the bottom
/// of the handle, divided by the 'slider range'.
#[derive(Reflect, Component, Default, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct SliderHandle;

//-------------------------------------------------------------------------------------------------------------------

/// Extension trait for interacting with [`SliderValue`] in a COB scene.
pub trait SliderWidgetExt
{
    /// Adds a callback for initializing the `React<SliderValue>` component on the current entity from world state.
    ///
    /// For example, if you have a slider for audio, use this to set the initial slider value equal to the current
    /// audio level (or get the initial value from a setting in the app).
    /**
    ```rust
    ui_builder.initialize_slider(
        |
            id: TargetId,
            mut c: Commands,
            settings: Res<Settings>,
            mut value: ReactiveMut<SliderValue>,
        | {
            let val = value.get_mut(&mut c, *id)?;
            *val = SliderValue::Single(settings.audio_level / 100.0);
            val.normalize();
            OK
        }
    );
    ```
    */
    ///
    /// Equivalent to:
    /// ```rust
    /// ui_builder.update_on(entity_insertion::<SliderValue>(entity), callback)
    /// ```
    fn initialize_slider<M, C, R: CobwebResult>(&mut self, callback: C) -> &mut Self
    where
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static;

    /// Adds a callback for reacting to changes in the `React<SliderValue>` component on the current entity.
    ///
    /// For example, if you have a slider for audio, use this to refresh the current audio level
    /// whenever the slider changes.
    /**
    ```rust
    ui_builder.on_slider(
        |
            id: TargetId,
            mut settings: ResMut<Settings>,
            value: Reactive<SliderValue>,
        | {
            let val = value.get(*id)?.single().result()?;
            settings.audio_level = val * 100.0;
            OK
        }
    );
    ```
    */
    ///
    /// Equivalent to:
    /// ```rust
    /// ui_builder.update_on(entity_mutation::<SliderValue>(entity), callback)
    /// ```
    fn on_slider<M, C, R: CobwebResult>(&mut self, callback: C) -> &mut Self
    where
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static;
}

impl SliderWidgetExt for UiBuilder<'_, Entity>
{
    fn initialize_slider<M, C, R: CobwebResult>(&mut self, callback: C) -> &mut Self
    where
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static,
    {
        self.update_on(entity_insertion::<SliderValue>(self.id()), callback)
    }

    fn on_slider<M, C, R: CobwebResult>(&mut self, callback: C) -> &mut Self
    where
        C: IntoSystem<TargetId, R, M> + Send + Sync + 'static,
    {
        self.update_on(entity_mutation::<SliderValue>(self.id()), callback)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System set in `PostUpdate` where slider widgets are updated.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct SliderUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebSliderPlugin;

impl Plugin for CobwebSliderPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<Slider>()
            .register_component_type::<SliderHandle>()
            .configure_sets(PostUpdate, SliderUpdateSet.in_set(UiSystems::PostLayout))
            .add_systems(PostUpdate, update_slider_handle_positions.in_set(SliderUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------

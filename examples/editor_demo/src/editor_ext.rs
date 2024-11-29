use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy::reflect::TypeInfo;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::editor::*;
use bevy_cobweb_ui::prelude::*;

use super::orbiter::Orbiter;

//-------------------------------------------------------------------------------------------------------------------

/// Entity event sent when a draggable number zone was dragged.
struct DragValue(f32);

impl DragValue
{
    fn get(&self) -> f32
    {
        self.0
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive component that tracks the current value of a draggable number zone.
#[derive(ReactComponent, PartialEq)]
struct FieldValue<T: Send + Sync + 'static>(T);

impl<T: Send + Sync + 'static> FieldValue<T>
{
    fn new(val: T) -> Self
    {
        Self(val)
    }

    fn get(&self) -> &T
    {
        &self.0
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct DragZoneDragDistance(f32);

//-------------------------------------------------------------------------------------------------------------------

const DRAG_VELOCITY_MODIFIER: f32 = 1.0 / 200.0;

// TODO: This is in a separate system because bevy's Drag event only fires when the cursor moves, but we need
// to get values every tick.
// TODO: don't hard-code the modifier
fn extract_drag_values(time: Res<Time>, mut c: Commands, distances: Query<(Entity, &DragZoneDragDistance)>)
{
    for (widget_id, distance) in distances.iter() {
        // TODO: consider making this exponential w/ respect to drag distance
        let delta = time.delta().as_secs_f32();
        let change = distance.0 * delta * DRAG_VELOCITY_MODIFIER;

        // Notify the entity about the value change.
        c.react().entity_event(widget_id, DragValue(change));
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn setup_drag(ec: &mut EntityCommands, widget_id: Entity)
{
    // TODO: drag reacting should be integrated with bevy_cobweb_ui's interaction extensions (need a fully
    // unified and clear input interface from bevy)
    ec.observe(move |drag: Trigger<Pointer<Drag>>, mut c: Commands| {
        c.entity(widget_id)
            .try_insert(DragZoneDragDistance(drag.distance.x));
    });
    ec.observe(move |_: Trigger<Pointer<DragEnd>>, mut c: Commands| {
        c.entity(widget_id).remove::<DragZoneDragDistance>();
    });
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: how to make this generic? might need a trait
fn make_draggable_field_widget<'a>(
    l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>,
    name: &'static str,
    initial_value: f32,
    bounds: RangeInclusive<f32>,
) -> Entity
{
    let mut widget_id = Entity::PLACEHOLDER;
    l.load_scene_and_edit(("editor_ext", "field_widget"), |l| {
        widget_id = l.id();

        l.insert_reactive(FieldValue::new(initial_value));

        l.get("name")
            .update(move |id: UpdateId, mut e: TextEditor| {
                write_text!(e, *id, "{name}:");
            });

        let bounds_start = *bounds.start();
        l.get("lower_bound")
            .update(move |id: UpdateId, mut e: TextEditor| {
                write_text!(e, *id, "{}", bounds_start);
            });

        // Set up the drag zone to modify the DragValue.
        let mut zone = l.get("value");
        let mut ec = zone.entity_commands();
        setup_drag(&mut ec, widget_id);

        l.get("value::text").update_on(
            entity_mutation::<FieldValue<f32>>(widget_id),
            move |id: UpdateId, mut e: TextEditor, vals: Reactive<FieldValue<f32>>| {
                let Some(val) = vals.get(widget_id) else { return };
                write_text!(e, *id, "{:.1}", val.get());
            },
        );

        let bounds_end = *bounds.end();
        l.get("upper_bound")
            .update(move |id: UpdateId, mut e: TextEditor| {
                write_text!(e, *id, "{}", bounds_end);
            });

        // Convert drag values to field value changes.
        l.on_event::<DragValue>().r(
            move |event: EntityEvent<DragValue>, mut c: Commands, mut fields: ReactiveMut<FieldValue<f32>>| {
                let Some((_, delta)) = event.try_read() else { return };
                let Some(val) = fields.get(widget_id) else { return };
                let bounds_width = *bounds.end() - *bounds.start();
                let mut new_val = delta.get() * bounds_width + val.get();
                if new_val > *bounds.end() {
                    new_val = *bounds.end();
                }
                if new_val < *bounds.start() {
                    new_val = *bounds.start();
                }
                fields.set_if_neq(&mut c, widget_id, FieldValue::new(new_val));
            },
        );
    });

    widget_id
}

//-------------------------------------------------------------------------------------------------------------------

struct DemoOrbiterWidget;

impl CobEditorWidget for DemoOrbiterWidget
{
    type Value = Orbiter;

    fn try_spawn(
        c: &mut Commands,
        s: &mut SceneLoader,
        parent: Entity,
        editor_ref: &CobEditorRef,
        value: &(dyn PartialReflect + 'static),
    ) -> bool
    {
        // Get the current orbiter value.
        let Some(initial_orbiter) = Orbiter::from_reflect(value) else { return false };

        // Extract field bounds from reflected value.
        // - We access everything before loading the widget scene so we can abort without side effects.
        //TODO: would rather auto-extract these, and auto-produce field widgets
        let Some(TypeInfo::Struct(t_struct)) = value.get_represented_type_info() else { return false };
        let Some(t_radius) = t_struct.field("radius") else { return false };
        let Some(t_velocity) = t_struct.field("velocity") else { return false };
        let Some(radius_bounds) = t_radius.get_attribute::<RangeInclusive<f32>>().clone() else { return false };
        let Some(velocity_bounds) = t_velocity.get_attribute::<RangeInclusive<f32>>().clone() else {
            return false;
        };

        // Build the widget.
        c.ui_builder(parent)
            .load_scene_and_edit(("editor_ext", "orbiter_widget"), s, |l| {
                // Field widget for radius.
                let radius_id =
                    make_draggable_field_widget(l, "radius", initial_orbiter.radius, radius_bounds.clone());

                // Field widget for velocity.
                let velocity_id =
                    make_draggable_field_widget(l, "velocity", initial_orbiter.velocity, velocity_bounds.clone());

                // Send updated values back to the editor.
                //TODO: update_on is overkill here, but we need to make sure the reactor's lifetime is tied
                //to this widget
                let mut orbiter_tracked = initial_orbiter;
                let editor_ref = editor_ref.clone();
                l.update_on(
                    (
                        entity_mutation::<FieldValue<f32>>(radius_id),
                        entity_mutation::<FieldValue<f32>>(velocity_id),
                    ),
                    move |//
                    _: UpdateId,
                    mutation: MutationEvent<FieldValue<f32>>,
                    mut c: Commands,
                    vals: Reactive<FieldValue<f32>>//
                | {
                    let Some(entity) = mutation.get() else { return };
                    let Some(val) = vals.get(entity) else { return };

                    let mut new_orbiter = orbiter_tracked;
                    if entity == radius_id {
                        new_orbiter.radius = *val.get();
                    }
                    if entity == velocity_id {
                        new_orbiter.velocity = *val.get();
                    }
                    if new_orbiter == orbiter_tracked {
                        return;
                    }
                    orbiter_tracked = new_orbiter;

                    // Submit new value.
                    c.queue(SubmitPatch {
                        editor_ref: editor_ref.clone(),
                        value: Box::new(new_orbiter),
                    });
                },
                );
            });

        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct DemoEditorExtPlugin;

impl Plugin for DemoEditorExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_editor_widget::<DemoOrbiterWidget>()
            .add_systems(Update, extract_drag_values);
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::ui::UiSystem;

use crate::*;

pub struct DynamicStylePlugin;

impl Plugin for DynamicStylePlugin
{
    fn build(&self, app: &mut App)
    {
        app.configure_sets(PostUpdate, DynamicStylePostUpdate.before(UiSystem::Prepare))
            .add_systems(
                PostUpdate,
                (
                    update_dynamic_style_static_attributes,
                    update_dynamic_style_on_flux_change,
                    tick_dynamic_style_stopwatch,
                    update_dynamic_style_on_stopwatch_change,
                    // Cleanup in a separate step in case of stopwatches that only exist for 1 tick.
                    cleanup_dynamic_style_stopwatch,
                )
                    .chain()
                    .in_set(DynamicStylePostUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct DynamicStylePostUpdate;

fn update_dynamic_style_static_attributes(
    mut q_styles: Query<(Entity, &mut DynamicStyle), Changed<DynamicStyle>>,
    mut commands: Commands,
)
{
    for (entity, mut style) in &mut q_styles {
        let mut had_static = false;
        for context_attribute in &style.attributes {
            let DynamicStyleAttribute::Static(style) = &context_attribute.attribute else {
                continue;
            };

            let target = match context_attribute.target {
                Some(context) => context,
                None => entity,
            };

            style.apply(&mut commands.style(target));
            had_static = true;
        }

        if had_static {
            let style = style.bypass_change_detection();
            style.attributes.retain(|csa| !csa.attribute.is_static());

            if style.attributes.len() == 0 {
                commands.entity(entity).remove::<DynamicStyle>();
            }
        }
    }
}

fn update_dynamic_style_on_flux_change(
    mut q_styles: Query<
        (
            Entity,
            Ref<DynamicStyle>,
            &FluxInteraction,
            Option<&mut DynamicStyleStopwatch>,
        ),
        Or<(Changed<DynamicStyle>, Changed<FluxInteraction>)>,
    >,
    mut commands: Commands,
)
{
    for (entity, style, interaction, stopwatch) in &mut q_styles {
        let mut lock_needed = StopwatchLock::None;
        let mut keep_stop_watch = false;

        for context_attribute in &style.attributes {
            match &context_attribute.attribute {
                DynamicStyleAttribute::Responsive(style) => {
                    let target = match context_attribute.target {
                        Some(context) => context,
                        None => entity,
                    };

                    style.apply(*interaction, &mut commands.style(target));
                }
                DynamicStyleAttribute::Animated { controller, .. } => {
                    let animation_lock = if !controller.is_entered() {
                        keep_stop_watch = true;

                        controller.animation.lock_duration(&FluxInteraction::None)
                            + controller.animation.lock_duration(interaction)
                    } else {
                        controller.animation.lock_duration(interaction)
                    };

                    if animation_lock > lock_needed {
                        lock_needed = animation_lock;
                    }
                }
                _ => continue,
            }
        }

        if let Some(mut stopwatch) = stopwatch {
            if !keep_stop_watch || style.is_changed() {
                stopwatch.0.reset();
            }
            stopwatch.1 = lock_needed;
        } else {
            commands
                .entity(entity)
                .insert(DynamicStyleStopwatch(Stopwatch::new(), lock_needed));
        }
    }
}

fn tick_dynamic_style_stopwatch(time: Res<Time<Real>>, mut q_stopwatches: Query<&mut DynamicStyleStopwatch>)
{
    for mut style_stopwatch in &mut q_stopwatches {
        style_stopwatch.0.tick(time.delta());
    }
}

fn update_dynamic_style_on_stopwatch_change(
    mut p: ParamSet<(
        &World,
        Query<
            (
                Entity,
                &mut DynamicStyle,
                &FluxInteraction,
                Option<&DynamicStyleStopwatch>,
            ),
            Or<(
                Changed<DynamicStyle>,
                Changed<FluxInteraction>,
                Changed<DynamicStyleStopwatch>,
            )>,
        >,
    )>,
    mut commands: Commands,
)
{
    let world_ptr: *const World = std::ptr::from_ref(p.p0());

    for (entity, mut style, interaction, stopwatch) in p.p1().iter_mut() {
        let style_changed = style.is_changed();
        let style = style.bypass_change_detection();
        let mut enter_completed = true;
        let mut filter_entered = false;

        for context_attribute in &mut style.attributes {
            let DynamicStyleAttribute::Animated { attribute, controller } = &mut context_attribute.attribute
            else {
                continue;
            };

            if let Some(stopwatch) = stopwatch {
                controller.update(interaction, stopwatch.0.elapsed_secs());
            }

            if style_changed || controller.dirty() {
                let target = match context_attribute.target {
                    Some(context) => context,
                    None => entity,
                };

                // Initialize the attribute's enter_ref value immediately before the first time we apply the
                // attribute.
                // - We need to do this here so the initialized value gets saved in the DynamicStyle component for
                //   reuse.
                if controller.just_started_entering() {
                    let mut init_attribute = attribute.clone();
                    {
                        // SAFETY:
                        // - The current system is exclusive because of the &World parameter.
                        // - The current iterator is not parallel, so there are no concurrent mutable accesses.
                        // - This world reference is read-only, so it can't invalidate the current iterator.
                        // - The attribute being mutated is a local variable that's not in the world.
                        let world = unsafe { &*world_ptr };
                        init_attribute.initialize_enter(target, world);
                    }
                    *attribute = init_attribute;
                }

                attribute.apply(controller.current_state(), &mut commands.style(target));
            }

            if !controller.is_entered() {
                enter_completed = false;
            } else if controller.animation.delete_on_entered {
                filter_entered = true;
            }
        }

        if !style.enter_completed && enter_completed {
            style.enter_completed = true;
        }

        if filter_entered {
            style.attributes.retain(|csa| {
                let DynamicStyleAttribute::Animated { controller, .. } = &csa.attribute else { return true };

                !(controller.animation.delete_on_entered && controller.is_entered())
            });

            if style.attributes.len() == 0 {
                commands.entity(entity).remove::<DynamicStyle>();
            }
        }
    }
}

fn cleanup_dynamic_style_stopwatch(
    mut q_stopwatches: Query<(Entity, &DynamicStyleStopwatch)>,
    mut commands: Commands,
)
{
    for (entity, style_stopwatch) in &mut q_stopwatches {
        let remove_stopwatch = match style_stopwatch.1 {
            StopwatchLock::None => true,
            StopwatchLock::Infinite => false,
            StopwatchLock::Duration(length) => style_stopwatch.0.elapsed() > length,
        };

        if remove_stopwatch {
            commands.entity(entity).remove::<DynamicStyleStopwatch>();
        }
    }
}

#[derive(Component, Clone, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct DynamicStyleStopwatch(pub Stopwatch, pub StopwatchLock);

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct DynamicStyleEnterState
{
    completed: bool,
}

impl DynamicStyleEnterState
{
    pub fn completed(&self) -> bool
    {
        self.completed
    }
}

#[derive(Clone, Debug)]
pub struct ContextStyleAttribute
{
    target: Option<Entity>,
    attribute: DynamicStyleAttribute,
}

impl LogicalEq for ContextStyleAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self.target == other.target && self.attribute.logical_eq(&other.attribute)
    }
}

impl ContextStyleAttribute
{
    pub fn new(context: impl Into<Option<Entity>>, attribute: DynamicStyleAttribute) -> Self
    {
        Self { target: context.into(), attribute }
    }
}

// TODO: Consider moving to sparse set. Static styles are removed in
// the same frame they are added, so only interaction animations stay long term.
// Measure impact
//#[component(storage = "SparseSet")]
#[derive(Component, Clone, Debug, Default)]
pub struct DynamicStyle
{
    attributes: Vec<ContextStyleAttribute>,
    enter_completed: bool,
}

impl DynamicStyle
{
    pub fn new(attributes: Vec<DynamicStyleAttribute>) -> Self
    {
        Self {
            attributes: attributes
                .iter()
                .map(|attribute| ContextStyleAttribute { target: None, attribute: attribute.clone() })
                .collect(),
            enter_completed: false,
        }
    }

    pub fn enter_completed(&self) -> bool
    {
        self.enter_completed
    }

    pub fn copy_from(attributes: Vec<ContextStyleAttribute>) -> Self
    {
        Self { attributes, enter_completed: false }
    }

    pub fn merge(mut self, mut other: DynamicStyle) -> Self
    {
        self.merge_in_place(&mut other);
        self
    }

    pub fn merge_in_place(&mut self, other: &mut DynamicStyle)
    {
        self.merge_in_place_from_iter(other.attributes.drain(..));
        other.enter_completed = false;
    }

    pub fn merge_in_place_from_iter(&mut self, other_attrs: impl Iterator<Item = ContextStyleAttribute>)
    {
        for attribute in other_attrs {
            if !self.attributes.iter().any(|csa| csa.logical_eq(&attribute)) {
                self.attributes.push(attribute);
            } else {
                // Safe unwrap: checked in if above
                let index = self
                    .attributes
                    .iter()
                    .position(|csa| csa.logical_eq(&attribute))
                    .unwrap();
                self.attributes[index] = attribute;
            }
        }

        self.enter_completed = false;
    }

    pub fn copy_controllers(&mut self, other: &DynamicStyle)
    {
        for context_attribute in self.attributes.iter_mut() {
            if !context_attribute.attribute.is_animated() {
                continue;
            }

            let Some(old_attribute) = other
                .attributes
                .iter()
                .find(|csa| csa.logical_eq(context_attribute))
            else {
                continue;
            };

            let DynamicStyleAttribute::Animated { controller: old_controller, attribute: old_attribute } =
                &old_attribute.attribute
            else {
                continue;
            };

            let ContextStyleAttribute {
                attribute: DynamicStyleAttribute::Animated { ref mut controller, attribute },
                ..
            } = context_attribute
            else {
                continue;
            };

            if attribute == old_attribute && controller.animation == old_controller.animation {
                controller.copy_state_from(old_controller);
            }
        }
    }

    pub fn is_interactive(&self) -> bool
    {
        self.attributes
            .iter()
            .any(|csa| csa.attribute.is_responsive())
    }

    pub fn is_animated(&self) -> bool
    {
        self.attributes
            .iter()
            .any(|csa| csa.attribute.is_animated())
    }

    /// Extracts the inner attribute buffer.
    ///
    /// Allows re-using the buffer via [`Self::copy_from`]. See [`StyleBuilder::convert_to_iter_with_buffers`].
    pub fn take_inner(self) -> Vec<ContextStyleAttribute>
    {
        self.attributes
    }
}

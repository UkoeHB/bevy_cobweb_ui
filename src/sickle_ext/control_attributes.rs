use std::any::{type_name, TypeId};

use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if the attribute was applied to a control map on the entity.
///
/// We do *not* support setting the 'target' of an attribute because it is not easy to correctly revert
/// attribute instructions when they are applied to other entities. It's also not clear what use the target has.
pub(super) fn add_attribute(
    In((origin, mut source, state, attribute, type_name)): In<(
        Entity,
        Option<SmolStr>,
        Option<SmallVec<[PseudoState; 3]>>,
        DynamicStyleAttribute,
        &'static str,
    )>,
    mut c: Commands,
    parents: Query<&Parent>,
    labels: Query<&ControlLabel>,
    entities: &Entities,
    mut control_maps: Query<&mut ControlMap>,
) -> bool
{
    if !entities.contains(origin) {
        return false;
    }

    // Get the current entity's control label.
    let maybe_label = labels.get(origin).ok().map(|l| (**l).clone());

    // Check if self has ControlMap.
    if let Ok(control_map) = control_maps.get_mut(origin) {
        let control_map_mut = control_map.into_inner();
        if maybe_label.is_none() {
            if !control_map_mut.is_anonymous() {
                // Replace with anonymous map.
                let mut old_control_map = std::mem::replace(control_map_mut, ControlMap::new_anonymous());

                // Repair - re-apply attributes from the map.
                c.queue(move |world: &mut World| {
                    // Labels
                    // - Re-applying these forces the entities to re-register in the correct control maps.
                    for (label, label_entity) in old_control_map.remove_all_labels() {
                        // The current entity doesn't have a label - all attributes are being reapplied
                        // anonymously.
                        if label_entity == origin {
                            continue;
                        }
                        ControlLabel(label).apply(label_entity, world);
                    }

                    // Attrs
                    // Note: target not needed, it is always set to self.
                    let attrs = old_control_map.remove_all_attrs();
                    for (origin, source, _target, state, attribute) in attrs {
                        world.syscall((origin, source, state, attribute, "unknown"), super::add_attribute);
                    }
                });

                // We are not actually dying, we are converting from a normal map to an anonymous map.
                c.entity(origin).remove::<ControlMapDying>();
            }
        } else if control_map_mut.is_anonymous() {
            tracing::error!("failed inserting attribute {type_name} to {origin:?}; the entity unexpectedly has \
                an anonymous control map with a control label");
            return false;
        }

        let target = match maybe_label {
            Some(target) => target,
            None => {
                let label = SmolStr::new_static("__anon");
                control_map_mut.insert(label.clone(), origin);
                label
            }
        };

        // Fixup source based on assumed user intention.
        if let Some(src) = &source {
            if *src == target {
                // Clear source if it points to self.
                // TODO: why is this necessary? Something weird in sickle_ui means if the root node sources itself
                // then child nodes' attributes won't properly respond to interactions on the root.
                source = None;
            }
        }

        control_map_mut.set_attribute(origin, state, source, Some(target), attribute);
        return true;
    }

    // Add anonymous ControlMap if there's no label.
    let Some(target) = maybe_label else {
        let mut control_map = ControlMap::new_anonymous();
        let label = SmolStr::new_static("__anon");
        control_map.insert(label.clone(), origin);
        control_map.set_attribute(origin, state, None, Some(label), attribute);
        c.entity(origin).try_insert(control_map);
        return true;
    };

    // Find ancestor with non-anonymous ControlMap.
    for ancestor in parents.iter_ancestors(origin) {
        let Ok(mut control_map) = control_maps.get_mut(ancestor) else { continue };
        if control_map.is_anonymous() {
            continue;
        }
        control_map.set_attribute(origin, state, source, Some(target), attribute);
        return false;
    }

    tracing::error!(
        "failed adding controlled dynamic attribute {type_name} to {origin:?} with label {target:?}, \
        no ancestor with ControlRoot"
    );
    false
}

//-------------------------------------------------------------------------------------------------------------------

fn revert_attributes(
    In(entity): In<Entity>,
    mut c: Commands,
    parents: Query<&Parent>,
    entities: &Entities,
    mut control_maps: Query<&mut ControlMap>,
)
{
    if !entities.contains(entity) {
        return;
    }

    // Check if self has ControlMap.
    if let Ok(mut control_map) = control_maps.get_mut(entity) {
        if control_map.is_anonymous() {
            c.entity(entity).remove::<ControlMap>();
        } else {
            control_map.remove(entity);
        }

        return;
    }

    // Find ancestor with non-anonymous ControlMap.
    for ancestor in parents.iter_ancestors(entity) {
        let Ok(mut control_map) = control_maps.get_mut(ancestor) else { continue };
        if control_map.is_anonymous() {
            continue;
        }
        control_map.remove(entity);
        return;
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_static_value<T: StaticAttribute>(ref_val: T::Value) -> impl Fn(Entity, &mut World)
{
    move |entity: Entity, world: &mut World| {
        T::update(entity, world, ref_val.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_responsive_value<T: ResponsiveAttribute>(
    ref_vals: InteractiveVals<T::Value>,
) -> impl Fn(Entity, FluxInteraction, &mut World)
{
    move |entity: Entity, state: FluxInteraction, world: &mut World| {
        let value = T::extract(entity, world, &ref_vals, state);
        T::update(entity, world, value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_animation_value<T: AnimatedAttribute>(
    ref_vals: AnimatedVals<T::Value>,
) -> impl Fn(Entity, AnimationState, &mut World)
{
    move |entity: Entity, state: AnimationState, world: &mut World| {
        let value = T::extract(entity, world, &ref_vals, &state);
        T::update(entity, world, value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl<T> StaticAttribute for Splat<T>
where
    T: Splattable + StaticAttribute,
{
    type Value = T::Splat;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl<T> ResponsiveAttribute for Splat<T> where T: Splattable + ResponsiveAttribute {}
impl<T> AnimatedAttribute for Splat<T>
where
    T: Splattable + AnimatedAttribute,
    <T as Splattable>::Splat: Lerp,
{
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for static values.
///
/// Useful for values in widgets that should change based on the widget's [`PseudoStates`](PseudoState).
//TODO: how to properly add Serialize/Deserialize derives when `serde` feature is enabled? we don't want to
// require that T::Value implements Serialize/Deserialize unless necessary
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Static<T: StaticAttribute>
{
    /// Specifies which [`PseudoStates`](PseudoState) the root node of the control group this entity is a member
    /// of must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlLabel`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,
    /// The value that will be applied to the entity with `T`.
    pub value: T::Value,
}

impl<T: StaticAttribute> Instruction for Static<T>
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
            CustomStaticStyleAttribute::new(TypeId::of::<Self>(), extract_static_value::<T>(self.value)),
        ));

        world.syscall(
            (entity, None, self.state, attribute, type_name::<Self>()),
            add_attribute,
        );
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Revert instruction.
        T::revert(entity, world);

        // Revert attributes.
        world.syscall(entity, revert_attributes);
        let _ = world.get_entity_mut(entity).map(|mut emut| {
            emut.remove::<DynamicStyle>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for responsive values.
///
/// Note that the `InteractiveVals::idle` field must always be set, which means it is effectively the 'default'
/// value for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Responsive<T: ResponsiveAttribute>
{
    /// Specifies which [`PseudoStates`](PseudoState) the root node of the control group this entity is a member
    /// of must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlLabel`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,

    /// The [`ControlLabel`] of an entity in the current widget. This attribute responds to interactions on
    /// that entity.
    ///
    /// If `None`, then:
    /// - If the current entity has no [`ControlLabel`], then interactions on the current entity will control the
    ///   value.
    /// - If the current entity *does* have a [`ControlLabel`], then interactions on the nearest [`ControlRoot`]
    ///   entity will control the value.
    #[reflect(default)]
    pub respond_to: Option<SmolStr>,

    /// The value to display when the source is idle.
    pub idle: T::Value,
    /// The value to display when the source is hovered.
    #[reflect(default)]
    pub hover: Option<T::Value>,
    /// The value to display when the source is pressed.
    #[reflect(default)]
    pub press: Option<T::Value>,
    /// The value to display when the source is canceled.
    #[reflect(default)]
    pub cancel: Option<T::Value>,
}

impl<T: ResponsiveAttribute> Instruction for Responsive<T>
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let ref_vals = InteractiveVals::<T::Value> {
            idle: self.idle,
            hover: self.hover,
            press: self.press,
            cancel: self.cancel,
        };

        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Interactive(InteractiveStyleAttribute::new(
            TypeId::of::<Self>(),
            extract_responsive_value::<T>(ref_vals),
        ));

        if world.syscall(
            (entity, self.respond_to, self.state, attribute, type_name::<Self>()),
            add_attribute,
        ) {
            // Interactive if the attribute was applied directly to self.
            Interactive.apply(entity, world);
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Revert instruction.
        T::revert(entity, world);

        // Revert attributes.
        world.syscall(entity, revert_attributes);
        Interactive::revert(entity, world);
        let _ = world.get_entity_mut(entity).map(|mut emut| {
            emut.remove::<DynamicStyle>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for animatable values.
///
/// Note that the `AnimatedVals::idle` field must always be set, which means it is effectively the 'default' value
/// for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Animated<T: AnimatedAttribute>
{
    /// Specifies which [`PseudoStates`](PseudoState) the root node of the control group this entity is a member
    /// of must be in for this animation to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlLabel`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,

    /// The [`ControlLabel`] of an entity in the current widget. Interactions on that entity will control this
    /// value.
    ///
    /// If `None`, then:
    /// - If the current entity has no [`ControlLabel`], then interactions on the current entity will control the
    ///   value.
    /// - If the current entity *does* have a [`ControlLabel`], then interactions on the nearest [`ControlRoot`]
    ///   entity will control the value.
    #[reflect(default)]
    pub respond_to: Option<SmolStr>,

    /// Reference value to use when animating to [`Self::idle`] with [`Self::enter_idle_with`] when the attribute
    /// is first applied.
    #[reflect(default)]
    pub enter_ref: Option<T::Value>,

    /// The value when idle.
    pub idle: T::Value,
    /// Controls animation to the [`Self::idle`] value when the attribute is first applied.
    #[reflect(default)]
    pub enter_idle_with: Option<AnimationConfig>,
    /// Controls animation to the [`Self::idle`] value when the entity becomes disabled.
    #[reflect(default)]
    pub disable_with: Option<AnimationConfig>,

    /// The value when hovered.
    #[reflect(default)]
    pub hover: Option<T::Value>,
    #[reflect(default)]
    pub hover_with: Option<AnimationConfig>,
    /// Controls animation to the [`Self::idle`] value when the pointer leaves the entity.
    #[reflect(default)]
    pub unhover_with: Option<AnimationConfig>,

    /// The value when pressed.
    #[reflect(default)]
    pub press: Option<T::Value>,
    #[reflect(default)]
    pub press_with: Option<AnimationConfig>,
    /// Controls animation from [`Self::press`] to [`Self::hover`].
    #[reflect(default)]
    pub release_with: Option<AnimationConfig>,

    /// The value when a press was recently canceled.
    #[reflect(default)]
    pub cancel: Option<T::Value>,
    #[reflect(default)]
    pub cancel_with: Option<AnimationConfig>,
    /// Controls animation from [`Self::cancel`] to [`Self::idle`].
    #[reflect(default)]
    pub cancel_end_with: Option<AnimationConfig>,

    /// Secondary value that idle loops ping-pong to.
    #[reflect(default)]
    pub idle_secondary: Option<T::Value>,
    #[reflect(default)]
    pub idle_loop: Option<LoopedAnimationConfig>,

    /// Secondary value that hover loops ping-pong to.
    #[reflect(default)]
    pub hover_secondary: Option<T::Value>,
    #[reflect(default)]
    pub hover_loop: Option<LoopedAnimationConfig>,

    /// Secondary value that press loops ping-pong to.
    #[reflect(default)]
    pub press_secondary: Option<T::Value>,
    #[reflect(default)]
    pub press_loop: Option<LoopedAnimationConfig>,

    /// Indicates whether the attribute should be removed from the entity after [`Self::enter_idle_with`]
    /// executes.
    #[reflect(default)]
    pub delete_on_entered: bool,
}

impl<T: AnimatedAttribute> Instruction for Animated<T>
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let ref_vals = AnimatedVals::<T::Value> {
            enter_ref: self.enter_ref,
            idle: self.idle,
            hover: self.hover,
            press: self.press,
            cancel: self.cancel,
            idle_secondary: self.idle_secondary,
            hover_secondary: self.hover_secondary,
            press_secondary: self.press_secondary,
        };
        let settings = AnimationSettings {
            enter_idle_with: self.enter_idle_with,
            hover_with: self.hover_with,
            unhover_with: self.unhover_with,
            press_with: self.press_with,
            release_with: self.release_with,
            cancel_with: self.cancel_with,
            cancel_end_with: self.cancel_end_with,
            disable_with: self.disable_with,
            idle_loop: self.idle_loop,
            hover_loop: self.hover_loop,
            press_loop: self.press_loop,
            delete_on_entered: self.delete_on_entered,
        };

        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::new(TypeId::of::<Self>(), extract_animation_value::<T>(ref_vals)),
            controller: DynamicStyleController::new(settings, AnimationState::default()),
        };

        if world.syscall(
            (entity, self.respond_to, self.state, attribute, type_name::<Self>()),
            add_attribute,
        ) {
            // Interactive if the attribute was applied directly to self.
            Interactive.apply(entity, world);
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Revert instruction.
        T::revert(entity, world);

        // Revert attributes.
        world.syscall(entity, revert_attributes);
        Interactive::revert(entity, world);
        let _ = world.get_entity_mut(entity).map(|mut emut| {
            emut.remove::<DynamicStyle>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

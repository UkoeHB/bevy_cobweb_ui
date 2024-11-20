use std::any::type_name;

use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use bevy::reflect::{GetTypeRegistration, Typed};
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_attribute_to_dynamic_style(
    entity: Entity,
    attribute: DynamicStyleAttribute,
    c: &mut Commands,
    dynamic_styles: &mut Query<Option<&mut DynamicStyle>>,
)
{
    // Insert directly to this entity.
    let Some(maybe_style) = dynamic_styles.get_mut(entity).ok() else { return };

    // Contextualize the attribute.
    let contextual_attribute = ContextStyleAttribute::new(entity, attribute);

    // Add this attribute directly.
    // - NOTE: If the entity has a themed component or is given a ControlLabel at a later time, then these changes
    //   MAY be overwritten.
    if let Some(mut existing) = maybe_style {
        existing.merge_in_place_from_iter([contextual_attribute].into_iter());
    } else {
        c.entity(entity)
            .try_insert(DynamicStyle::copy_from(vec![contextual_attribute]));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns `true` if the attribute is added directly, without a control map.
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
    mut dynamic_styles: Query<Option<&mut DynamicStyle>>,
) -> bool
{
    if !entities.contains(origin) {
        return false;
    }

    // Get the current entity's control label.
    let Ok(label) = labels.get(origin) else {
        if let Some(state) = &state {
            if !state.is_empty() {
                tracing::error!(
                    "failed adding attribute {type_name} to {origin:?}, pseudo states are not supported for \
                    non-controlled dynamic sytle attributes (state: {:?})",
                    state
                );
                return false;
            }
        }

        if let Some(source) = &source {
            tracing::warn!(
                "ignoring control source {source:?} for attribute {type_name} on {origin:?} that doesn't \
                have a ControlLabel"
            );
        }

        // Fall back to inserting as a plain dynamic style attribute.
        tracing::debug!("{origin:?} is not controlled by a widget, inserting attribute {type_name} to dynamic style \
            instead");
        add_attribute_to_dynamic_style(origin, attribute, &mut c, &mut dynamic_styles);
        return true;
    };

    // Always target self.
    let target = Some((**label).clone());

    // Check if self has ControlMap.
    if let Ok(mut control_map) = control_maps.get_mut(origin) {
        // Fixup source/target based on assumed user intention.
        if let Some(src) = &source {
            if *src == **label {
                // Clear source if it points to self.
                // TODO: why is this necessary? Something weird in sickle_ui means if the root node sources itself
                // then child nodes' attributes won't properly respond to interactions on the root.
                source = None;
            }
        }

        control_map.set_attribute(origin, state, source, target, attribute);
        return false;
    }

    // Find ancestor with ControlMap.
    for ancestor in parents.iter_ancestors(origin) {
        let Ok(mut control_map) = control_maps.get_mut(ancestor) else { continue };
        control_map.set_attribute(origin, state, source, target, attribute);
        return false;
    }

    tracing::error!(
        "failed adding controlled dynamic attribute {type_name} to {origin:?} with {label:?}, \
        no ancestor with ControlRoot"
    );
    false
}

//-------------------------------------------------------------------------------------------------------------------

fn revert_attributes(
    In(entity): In<Entity>,
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
        // Target falls back to None, which is implicitly the root entity.
        control_map.remove(entity);
        return;
    }

    // Find ancestor with ControlMap.
    for ancestor in parents.iter_ancestors(entity) {
        let Ok(mut control_map) = control_maps.get_mut(ancestor) else { continue };
        control_map.remove(entity);
        return;
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_static_value<T: StaticAttribute>(val: T::Value) -> impl Fn(Entity, &mut World)
{
    move |entity: Entity, world: &mut World| {
        T::construct(val.clone()).apply(entity, world);
        world.flush();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_responsive_value<T: ResponsiveAttribute + StaticAttribute>(
    vals: InteractiveVals<T::Value>,
) -> impl Fn(Entity, FluxInteraction, &mut World)
{
    move |entity: Entity, state: FluxInteraction, world: &mut World| {
        // Compute new value.
        let new_value = vals.to_value(state);

        // Apply the value to the entity.
        T::construct(new_value).apply(entity, world);
        world.flush();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_animation_value<T: AnimatableAttribute + StaticAttribute>(
    vals: AnimatedVals<T::Value>,
) -> impl Fn(Entity, AnimationState, &mut World)
where
    <T as StaticAttribute>::Value: Lerp,
{
    move |entity: Entity, state: AnimationState, world: &mut World| {
        // Compute new value.
        let new_value = vals.to_value(&state);

        // Apply the value to the entity.
        T::construct(new_value).apply(entity, world);
        world.flush();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that specify a value for a theme.
pub trait StaticAttribute: Instruction + Typed
{
    /// Specifies the value-type of the theme attribute.
    type Value: Loadable + Typed + Clone;

    /// Converts [`Self::Value`] into `Self`.
    fn construct(value: Self::Value) -> Self;
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that respond to interactions.
///
/// Use [`Interactive`] to make an entity interactable.
pub trait ResponsiveAttribute: Loadable + Typed {}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
///
/// Use [`Interactive`] to make an entity interactable.
pub trait AnimatableAttribute: Loadable + Typed {}

//-------------------------------------------------------------------------------------------------------------------

impl<T> StaticAttribute for Splat<T>
where
    T: Instruction + Splattable + StaticAttribute + GetTypeRegistration,
{
    type Value = T::Splat;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl<T> ResponsiveAttribute for Splat<T> where T: Splattable + ResponsiveAttribute {}
impl<T> AnimatableAttribute for Splat<T> where T: Splattable + AnimatableAttribute {}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for static values.
///
/// Useful for values in widgets that should change based on the widget's [`PseudoStates`](PseudoState).
//TODO: how to properly add Serialize/Deserialize derives when `serde` feature is enabled? we don't want to
// require that T::Value implements Serialize/Deserialize unless necessary
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Static<T: StaticAttribute>
where
    <T as StaticAttribute>::Value: GetTypeRegistration,
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
where
    <T as StaticAttribute>::Value: GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
            CustomStaticStyleAttribute::new(extract_static_value::<T>(self.value)),
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
pub struct Responsive<T: ResponsiveAttribute + StaticAttribute>
where
    <T as StaticAttribute>::Value: GetTypeRegistration,
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

impl<T: ResponsiveAttribute + StaticAttribute> Instruction for Responsive<T>
where
    <T as StaticAttribute>::Value: GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let values = InteractiveVals::<T::Value> {
            idle: self.idle,
            hover: self.hover,
            press: self.press,
            cancel: self.cancel,
        };

        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Interactive(InteractiveStyleAttribute::new(
            extract_responsive_value::<T>(values),
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
pub struct Animated<T: AnimatableAttribute + StaticAttribute>
where
    <T as StaticAttribute>::Value: Lerp + GetTypeRegistration,
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

impl<T: AnimatableAttribute + StaticAttribute> Instruction for Animated<T>
where
    <T as StaticAttribute>::Value: Lerp + GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let values = AnimatedVals::<T::Value> {
            idle: self.idle,
            hover: self.hover,
            press: self.press,
            cancel: self.cancel,
            idle_alt: self.idle_secondary,
            hover_alt: self.hover_secondary,
            press_alt: self.press_secondary,
            enter_from: self.enter_ref,
        };
        let settings = AnimationSettings {
            enter: self.enter_idle_with,
            pointer_enter: self.hover_with,
            pointer_leave: self.unhover_with,
            press: self.press_with,
            release: self.release_with,
            cancel: self.cancel_with,
            cancel_reset: self.cancel_end_with,
            disable: self.disable_with,
            idle: self.idle_loop,
            hover: self.hover_loop,
            pressed: self.press_loop,
            delete_on_entered: self.delete_on_entered,
        };

        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::new(extract_animation_value::<T>(values)),
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

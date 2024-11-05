use attributes::custom_attrs::CustomStaticStyleAttribute;
use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_cobweb::prelude::*;
use sickle_ui_scaffold::attributes::custom_attrs::{AnimatedStyleAttribute, InteractiveStyleAttribute};
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle_ext::attributes::dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController};
use crate::sickle_ext::attributes::pseudo_state::PseudoState;
use crate::sickle_ext::attributes::style_animation::{AnimationSettings, AnimationState};
use crate::sickle_ext::lerp::Lerp;
use crate::sickle_ext::prelude::*;

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

pub(super) fn add_attribute(
    In((origin, mut source, mut target, state, attribute)): In<(
        Entity,
        Option<SmolStr>,
        Option<SmolStr>,
        Option<SmallVec<[PseudoState; 3]>>,
        DynamicStyleAttribute,
    )>,
    mut c: Commands,
    parents: Query<&Parent>,
    labels: Query<&ControlLabel>,
    entities: &Entities,
    mut control_maps: Query<&mut ControlMap>,
    mut dynamic_styles: Query<Option<&mut DynamicStyle>>,
)
{
    if !entities.contains(origin) {
        return;
    }

    // Get the current entity's control label.
    let Ok(label) = labels.get(origin) else {
        if let Some(state) = &state {
            if !state.is_empty() {
                tracing::error!(
                    "failed adding attribute to {origin:?}, pseudo states are not supported for \
                    non-controlled dynamic sytle attributes (state: {:?})",
                    state
                );
                return;
            }
        }

        if let Some(source) = &source {
            tracing::warn!(
                "ignoring control source {source:?} for dynamic style attribute on {origin:?} that doesn't \
                have a ControlLabel"
            );
        }

        if let Some(target) = &target {
            tracing::warn!(
                "ignoring control target {target:?} for dynamic style attribute on {origin:?} that doesn't \
                have a ControlLabel"
            );
        }

        // Fall back to inserting as a plain dynamic style attribute.
        tracing::debug!("{origin:?} is not controlled by a widget, inserting attribute to dynamic style instead");
        add_attribute_to_dynamic_style(origin, attribute, &mut c, &mut dynamic_styles);
        return;
    };

    // Check if self has ControlMap.
    if let Ok(mut control_map) = control_maps.get_mut(origin) {
        // Fixup source/target based on assumed user intention.
        if let Some(src) = &source {
            if *src == **label {
                // Clear source if it points to self.
                // TODO: why is this necessary? Something weird in sickle_ui means if the root node sources itself
                // then child nodes' attributes won't properly respond to interactions on the root.
                source = None;
            } else {
                // Target self if the source is not self.
                // TODO: without this, the target is set to be the source by sickle_ui.
                if target.is_none() {
                    target = Some((**label).clone());
                }
            }
        }

        control_map.set_attribute(origin, state, source, target, attribute);
        return;
    }

    // Find ancestor with ControlMap.
    for ancestor in parents.iter_ancestors(origin) {
        let Ok(mut control_map) = control_maps.get_mut(ancestor) else { continue };
        // Target falls back to self.
        target = target.or_else(|| Some(label.deref().clone()));
        control_map.set_attribute(origin, state, source, target, attribute);
        return;
    }

    tracing::error!(
        "failed adding controlled dynamic attribute to {origin:?} with {label:?}, \
        no ancestor with ControlRoot"
    );
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

fn extract_static_value<T: ThemedAttribute>(val: T::Value) -> impl Fn(Entity, &mut World)
{
    move |entity: Entity, world: &mut World| {
        T::construct(val.clone()).apply(entity, world);
        world.flush();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_responsive_value<T: ResponsiveAttribute + ThemedAttribute>(
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

fn extract_animation_value<T: AnimatableAttribute + ThemedAttribute>(
    vals: AnimatedVals<T::Value>,
) -> impl Fn(Entity, AnimationState, &mut World)
where
    <T as ThemedAttribute>::Value: Lerp,
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
pub trait ThemedAttribute: Instruction + TypePath
{
    /// Specifies the value-type of the theme attribute.
    type Value: Loadable + TypePath + Clone;

    /// Converts [`Self::Value`] into `Self`.
    fn construct(value: Self::Value) -> Self;
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that respond to interactions.
///
/// Use [`Interactive`] to make an entity interactable.
pub trait ResponsiveAttribute: Loadable + TypePath {}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
///
/// Use [`Interactive`] to make an entity interactable.
pub trait AnimatableAttribute: Loadable + TypePath {}

//-------------------------------------------------------------------------------------------------------------------

impl<T> ThemedAttribute for Splat<T>
where
    T: Instruction + Splattable + ThemedAttribute + GetTypeRegistration,
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

/// Loadable type for theme values.
///
/// Primarily useful for values in widgets that should change based on the widget's [`PseudoStates`](PseudoState).
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Themed<T: ThemedAttribute>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlLabel`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,
    /// The value that will be applied to the entity with `T`.
    pub value: T::Value,

    /// The [`ControlLabel`] of an entity in the current widget. The value will be applied to that entity.
    ///
    /// If `None`, then the value will be applied to the current entity.
    #[reflect(default)]
    pub target: Option<SmolStr>,
}

impl<T: ThemedAttribute> Instruction for Themed<T>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
            CustomStaticStyleAttribute::new(extract_static_value::<T>(self.value)),
        ));

        world.syscall((entity, None, self.target, self.state, attribute), add_attribute);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Revert instruction.
        T::revert(entity, world);

        // Revert attributes.
        world.syscall(entity, revert_attributes);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for responsive values.
///
/// Note that the `InteractiveVals::idle` field must always be set, which means it is effectively the 'default'
/// value for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Responsive<T: ResponsiveAttribute + ThemedAttribute>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlLabel`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,
    /// The values that are toggled in response to interaction changes.
    pub values: InteractiveVals<T::Value>,

    /// The [`ControlLabel`] of an entity in the current widget. Interactions on that entity will control this
    /// value.
    ///
    /// If `None`, then:
    /// - If the current entity has no [`ControlLabel`], then interactions on the current entity will control the
    ///   value.
    /// - If the current entity *does* have a [`ControlLabel`], then interactions on the nearest [`ControlRoot`]
    ///   entity will control the value.
    #[reflect(default)]
    pub source: Option<SmolStr>,
    /// The [`ControlLabel`] of an entity in the current widget. The value will be applied to that entity.
    ///
    /// If `None`, then the value will be applied to the current entity.
    #[reflect(default)]
    pub target: Option<SmolStr>,
}

impl<T: ResponsiveAttribute + ThemedAttribute> Instruction for Responsive<T>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Interactive(InteractiveStyleAttribute::new(
            extract_responsive_value::<T>(self.values),
        ));

        world.syscall((entity, self.source, self.target, self.state, attribute), add_attribute);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Revert instruction.
        T::revert(entity, world);

        // Revert attributes.
        world.syscall(entity, revert_attributes);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for animatable values.
///
/// Note that the `AnimatedVals::idle` field must always be set, which means it is effectively the 'default' value
/// for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Animated<T: AnimatableAttribute + ThemedAttribute>
where
    <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration,
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this animation to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlLabel`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,
    /// The values that are end-targets for each animation.
    pub values: AnimatedVals<T::Value>,
    /// Settings that control how values are interpolated.
    #[reflect(default)]
    pub settings: AnimationSettings,

    /// The [`ControlLabel`] of an entity in the current widget. Interactions on that entity will control this
    /// value.
    ///
    /// If `None`, then:
    /// - If the current entity has no [`ControlLabel`], then interactions on the current entity will control the
    ///   value.
    /// - If the current entity *does* have a [`ControlLabel`], then interactions on the nearest [`ControlRoot`]
    ///   entity will control the value.
    #[reflect(default)]
    pub source: Option<SmolStr>,
    /// The [`ControlLabel`] of an entity in the current widget. The value will be applied to that entity.
    ///
    /// If `None`, then the value will be applied to the current entity.
    #[reflect(default)]
    pub target: Option<SmolStr>,
}

impl<T: AnimatableAttribute + ThemedAttribute> Instruction for Animated<T>
where
    <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration,
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::new(extract_animation_value::<T>(self.values)),
            controller: DynamicStyleController::new(self.settings, AnimationState::default()),
        };

        world.syscall((entity, self.source, self.target, self.state, attribute), add_attribute);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Revert instruction.
        T::revert(entity, world);

        // Revert attributes.
        world.syscall(entity, revert_attributes);
    }
}

//-------------------------------------------------------------------------------------------------------------------

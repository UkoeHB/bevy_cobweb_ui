use std::any::type_name;

use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::*;

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
///
/// Adds a [`NodeAttribute`] to the [`NodeAttributes`] component on the target entity.
//TODO: how to properly add Serialize/Deserialize derives when `serde` feature is enabled? we don't want to
// require that T::Value implements Serialize/Deserialize unless necessary
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Static<T: StaticAttribute>
{
    /// Sets the attribute name.
    ///
    /// Can be used to look up the attribute in the [`NodeAttributes`] component.
    #[reflect(default)]
    pub name: Option<SmolStr>,

    /// Specifies which [`PseudoStates`](PseudoState) the root node of the control group this entity is a member
    /// of must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlMember`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,

    /// The value that will be applied to the entity with `T`.
    pub value: T::Value,
}

impl<T: StaticAttribute> Instruction for Static<T>
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Add attribute.
        let attr = NodeAttribute::new_static::<T>(self.name, self.value);

        if let Some(mut attrs) = emut.get_mut::<NodeAttributes>() {
            if let Some(_) = attrs.insert(self.state, attr) {
                tracing::warn!("overwriting attribute {} on {:?}", type_name::<Self>(), entity);
            }
        } else {
            let mut attrs = NodeAttributes::default();
            attrs.insert(self.state, attr);
            emut.insert(attrs);
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Revert instruction.
        T::revert(entity, world);

        // Revert attributes.
        world.syscall(entity, revert_attributes);
        let _ = world.get_entity_mut(entity).map(|mut emut| {
            emut.remove::<(NodeAttributes, DynamicStyle)>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for responsive values.
///
/// Note that the `ResponsiveVals::idle` field must always be set, which means it is effectively the 'default'
/// value for `T` that will be applied to the entity and override any value you set elsewhere.
///
/// Adds a [`NodeAttribute`] to the [`NodeAttributes`] component on the target entity.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Responsive<T: ResponsiveAttribute>
{
    /// Sets the attribute name.
    ///
    /// Can be used to look up the attribute in the [`NodeAttributes`] component.
    #[reflect(default)]
    pub name: Option<SmolStr>,

    /// Specifies which [`PseudoStates`](PseudoState) the root node of the control group this entity is a member
    /// of must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlMember`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,

    /// The [`ControlMember`] of an entity in the current widget. This attribute responds to interactions on
    /// that entity.
    ///
    /// If `None`, then:
    /// - If the current entity has no [`ControlMember`], then interactions on the current entity will control the
    ///   value.
    /// - If the current entity *does* have a [`ControlMember`], then interactions on the nearest [`ControlRoot`]
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
        let ref_vals = ResponsiveVals::<T::Value> {
            idle: self.idle,
            hover: self.hover,
            press: self.press,
            cancel: self.cancel,
        };

        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Interactive if the attribute listens to interactions on self.
        let needs_interactive = emut.get::<ControlMember>().map(|m| &m.id) == self.respond_to.as_ref();

        // Add attribute.
        let attr = NodeAttribute::new_responsive::<T>(self.name, self.respond_to, ref_vals);

        if let Some(mut attrs) = emut.get_mut::<NodeAttributes>() {
            if let Some(_) = attrs.insert(self.state, attr) {
                tracing::warn!("overwriting attribute {} on {:?}", type_name::<Self>(), entity);
            }
        } else {
            let mut attrs = NodeAttributes::default();
            attrs.insert(self.state, attr);
            emut.insert(attrs);
        }

        if needs_interactive {
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
            emut.remove::<(NodeAttributes, DynamicStyle)>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction for animatable values.
///
/// Note that the `AnimatedVals::idle` field must always be set, which means it is effectively the 'default' value
/// for `T` that will be applied to the entity and override any value you set elsewhere.
///
/// Adds a [`NodeAttribute`] to the [`NodeAttributes`] component on the target entity.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Animated<T: AnimatedAttribute>
{
    /// Sets the attribute name.
    ///
    /// Can be used to look up the attribute in the [`NodeAttributes`] component.
    #[reflect(default)]
    pub name: Option<SmolStr>,

    /// Specifies which [`PseudoStates`](PseudoState) the root node of the control group this entity is a member
    /// of must be in for this animation to become active.
    ///
    /// Only used if this struct is applied to an entity with a [`ControlMember`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,

    /// The [`ControlMember`] of an entity in the current widget. Interactions on that entity will control this
    /// value.
    ///
    /// If `None`, then:
    /// - If the current entity has no [`ControlMember`], then interactions on the current entity will control the
    ///   value.
    /// - If the current entity *does* have a [`ControlMember`], then interactions on the nearest [`ControlRoot`]
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

        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Interactive if the attribute listens to interactions on self.
        let needs_interactive = emut.get::<ControlMember>().map(|m| &m.id) == self.respond_to.as_ref();

        // Add attribute.
        let attr = NodeAttribute::new_animated::<T>(self.name, self.respond_to, ref_vals, settings);

        if let Some(mut attrs) = emut.get_mut::<NodeAttributes>() {
            if let Some(_) = attrs.insert(self.state, attr) {
                tracing::warn!("overwriting attribute {} on {:?}", type_name::<Self>(), entity);
            }
        } else {
            let mut attrs = NodeAttributes::default();
            attrs.insert(self.state, attr);
            emut.insert(attrs);
        }

        if needs_interactive {
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
            emut.remove::<(NodeAttributes, DynamicStyle)>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

use crate::*;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::{lerp::Lerp, theme::{dynamic_style::DynamicStyle, dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController}, pseudo_state::PseudoState, style_animation::{AnimationSettings, AnimationState}}, ui_style::{AnimatedStyleAttribute, AnimatedVals}};

//-------------------------------------------------------------------------------------------------------------------

fn extract_animation_value<T: Animatable>(entity: Entity, state: AnimationState, world: &mut World)
{
    // Get the animation bundle if possible.
    let Some(bundle) = world.get_entity(entity).and_then(|e| e.get::<CachedAnimatedVals<T>>()) else { return };
    let new_value = bundle.cached.to_value(&state);

    // Apply the value to the entity.
    world.syscall((entity, new_value), |In((entity, new_val)): In<(Entity, T::Value)>, mut c: Commands| {
        let Some(mut ec) = c.get_entity(entity) else { return };
        T::update(&mut ec, new_val);
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn update_animation<T: Animatable>(
    In((entity, animation)): In<(Entity, Animated<T>)>,
    mut commands: Commands,
    mut query: Query<(Option<&mut DynamicStyle>, Option<&mut LoadedThemes>)>,
)
{
    // Store the AnimatedVals on the entity.
    let Some(mut ec) = commands.get_entity(entity) else { return };
    ec.try_insert(CachedAnimatedVals::<T>{ cached: animation.values });

    // Prepare an updated DynamicStyleAttribute.
    let mut controller = DynamicStyleController::default();
    controller.animation = animation.settings;

    let attribute = DynamicStyleAttribute::Animated {
        attribute: AnimatedStyleAttribute::Custom(extract_animation_value::<T>),
        controller,
    };

    // Access the entity.
    let Ok((maybe_style, maybe_themes)) = query.get_mut(entity) else { return };

    // If there is a loaded theme, then add this animation to the theme.
    if let Some(mut themes) = maybe_themes {
        themes.update(animation.state, attribute);
        commands.add(RefreshLoadedTheme{ entity });
        return;
    }

    // If there is no loaded theme, add this animation directly.
    let style = DynamicStyle::new(vec![attribute]);
    if let Some(mut existing) = maybe_style {
        let mut temp = DynamicStyle::new(Vec::default());
        std::mem::swap(&mut *existing, &mut temp);
        *existing = temp.merge(style);
    } else {
        ec.try_insert(style);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
struct CachedAnimatedVals<T: Animatable>
{
    cached: AnimatedVals<T::Value>
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
pub trait Animatable: Loadable + TypePath
{
    /// Specifies the value-type that will be animated on the target (e.g. `f32`, `Color`, etc.).
    type Value: Lerp + Loadable + TypePath;
    /// Specifies the interactivity loadable used by this animatable.
    ///
    /// Can be used to hook target entities up to custom interactivity.
    ///
    /// For example: [`Interactive`] (for `bevy_ui`), [`Interactive2d`] (for 2d UI).
    type Interactive: Default + ApplyLoadable;

    /// Specifies how values should be updated on an entity for this animatable attribute.
    fn update(entity_commands: &mut EntityCommands, value: Self::Value);
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for animatable values.
///
/// Note that the `AnimatedVals::idle` field must always be set, which means it is effectively the 'default' value
/// for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animated<T: Animatable>
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this animation to become active.
    ///
    /// Only used if this struct is applied to an entity with a loaded theme.
    pub state: Option<Vec<PseudoState>>,
    /// The values that are end-targets for each animation.
    pub values: AnimatedVals<T::Value>,
    /// Settings that control how values are interpolated.
    pub settings: AnimationSettings,
}

impl<T: Animatable> ApplyLoadable for Animated<T>
{
    fn apply(self, ec: &mut EntityCommands)
    {
        T::Interactive::default().apply(ec);

        let id = ec.id();
        ec.commands().syscall((id, self), update_animation::<T>);
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: Responsive<T>, same as Animated<T> but only toggles states
// TODO: Themed<T>, just adds a value directly to the theme

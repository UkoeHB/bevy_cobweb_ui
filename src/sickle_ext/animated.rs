use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::lerp::Lerp;
use sickle_ui::theme::dynamic_style::DynamicStyle;
use sickle_ui::theme::dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController};
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::style_animation::{AnimationSettings, AnimationState};
use sickle_ui::ui_style::{AnimatedStyleAttribute, AnimatedVals, CustomAnimatedStyleAttribute};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn extract_animation_value<T: AnimatableAttribute>(
    vals: AnimatedVals<T::Value>,
) -> impl Fn(Entity, AnimationState, &mut World)
{
    move |entity: Entity, state: AnimationState, world: &mut World| {
        // Compute new value.
        let new_value = vals.to_value(&state);

        // Apply the value to the entity.
        world.syscall(
            (entity, new_value),
            |In((entity, new_val)): In<(Entity, T::Value)>, mut c: Commands| {
                let Some(mut ec) = c.get_entity(entity) else { return };
                T::update(&mut ec, new_val);
            },
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn update_animation<T: AnimatableAttribute>(
    In((entity, animation)): In<(Entity, Animated<T>)>,
    mut commands: Commands,
    mut query: Query<(Option<&mut DynamicStyle>, Option<&mut LoadedThemes>)>,
)
{
    // Store the AnimatedVals on the entity.
    let Some(mut ec) = commands.get_entity(entity) else { return };

    // Prepare an updated DynamicStyleAttribute.
    let attribute = DynamicStyleAttribute::Animated {
        attribute: AnimatedStyleAttribute::Custom(CustomAnimatedStyleAttribute::new(
            extract_animation_value::<T>(animation.values),
        )),
        controller: DynamicStyleController::new(animation.settings, AnimationState::default()),
    };

    // Access the entity.
    let Ok((maybe_style, maybe_themes)) = query.get_mut(entity) else { return };

    // If there is a loaded theme, then add this animation to the theme.
    if let Some(mut themes) = maybe_themes {
        themes.update(animation.state, attribute);
        commands.add(RefreshLoadedTheme { entity });
        return;
    }

    // If there is no loaded theme, add this animation directly.
    // - NOTE: If the entity has a theme ancestor, then these changes MAY be overwritten.
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

/// Trait for loadable types that can be animated in response to interactions.
pub trait AnimatableAttribute: Loadable + TypePath
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
pub struct Animated<T: AnimatableAttribute>
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this animation to become active.
    ///
    /// Only used if this struct is applied to an entity with a loaded theme.
    #[reflect(default)]
    pub state: Option<Vec<PseudoState>>,
    /// The values that are end-targets for each animation.
    pub values: AnimatedVals<T::Value>,
    /// Settings that control how values are interpolated.
    pub settings: AnimationSettings,
}

impl<T: AnimatableAttribute> ApplyLoadable for Animated<T>
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

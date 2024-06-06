use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::lerp::Lerp;
use sickle_ui::theme::dynamic_style::{ContextStyleAttribute, DynamicStyle};
use sickle_ui::theme::dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController};
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::style_animation::{AnimationSettings, AnimationState};
use sickle_ui::ui_style::{
    AnimatedStyleAttribute, AnimatedVals, CustomAnimatedStyleAttribute, CustomInteractiveStyleAttribute,
    CustomStaticStyleAttribute, InteractiveStyleAttribute, InteractiveVals, StaticStyleAttribute,
};
use sickle_ui::FluxInteraction;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_attribute_to_theme_inner(
    entity: Entity,
    state: Option<Vec<PseudoState>>,
    attribute: DynamicStyleAttribute,
    commands: &mut Commands,
    store_entity: Entity,
    maybe_style: Option<&mut DynamicStyle>,
    maybe_themes: Option<&mut LoadedThemes>,
)
{
    // If there is a loaded theme, then add this attribute to the theme.
    let context = if entity != store_entity {
        Some(entity)
    } else {
        None
    };
    let contextual_attribute = ContextStyleAttribute::new(context, attribute);

    if let Some(themes) = maybe_themes {
        themes.update(state, contextual_attribute);
        commands.add(RefreshLoadedTheme { entity: store_entity });
        return;
    }

    // If there is no loaded theme, add this attribute directly.
    // - NOTE: If the entity has a theme ancestor, then these changes MAY be overwritten.
    let style = DynamicStyle::copy_from(vec![contextual_attribute]);
    if let Some(existing) = maybe_style {
        let mut temp = DynamicStyle::new(Vec::default());
        std::mem::swap(&mut *existing, &mut temp);
        *existing = temp.merge(style);
    } else {
        commands.entity(store_entity).try_insert(style);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn add_attribute_to_theme(
    In((entity, inherit_control, state, attribute)): In<(
        Entity,
        bool,
        Option<Vec<PseudoState>>,
        DynamicStyleAttribute,
    )>,
    mut commands: Commands,
    parents: Query<&Parent>,
    mut query: Query<(
        Entity,
        Option<&mut DynamicStyle>,
        Option<&mut LoadedThemes>,
        Has<PropagateControl>,
    )>,
)
{
    if !inherit_control {
        // Insert directly to this entity.
        let Some((store_entity, maybe_style, maybe_themes, _)) = query.get_mut(entity).ok() else { return };
        add_attribute_to_theme_inner(
            entity,
            state,
            attribute,
            &mut commands,
            store_entity,
            maybe_style.map(|i| i.into_inner()),
            maybe_themes.map(|i| i.into_inner()),
        );
    } else {
        // Find parent where attribute should be saved.
        let mut count = 0;
        for parent in parents.iter_ancestors(entity) {
            count += 1;
            let Some((store_entity, maybe_style, maybe_themes, has_propagate)) = query.get_mut(parent).ok() else {
                continue;
            };
            if !has_propagate {
                continue;
            }
            add_attribute_to_theme_inner(
                entity,
                state,
                attribute,
                &mut commands,
                store_entity,
                maybe_style.map(|i| i.into_inner()),
                maybe_themes.map(|i| i.into_inner()),
            );
            return;
        }

        if count > 0 {
            tracing::warn!("failed adding theme attribute with inherited interaction to {entity:?}, \
                no ancestor with PropagateControl");
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_static_value<T: ThemedAttribute>(val: T::Value) -> impl Fn(Entity, &mut World)
{
    move |entity: Entity, world: &mut World| {
        // Apply the value to the entity.
        world.syscall(
            (entity, val.clone()),
            |In((entity, new_val)): In<(Entity, T::Value)>, mut c: Commands| {
                let Some(mut ec) = c.get_entity(entity) else { return };
                T::update(&mut ec, new_val);
            },
        );
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

/// Trait for loadable types that specify a value for a theme.
pub trait ThemedAttribute: Loadable + TypePath
{
    /// Specifies the value-type of the theme attribute.
    type Value: Loadable + TypePath;

    /// Specifies how values should be updated on an entity for this themed attribute.
    fn update(entity_commands: &mut EntityCommands, value: Self::Value);
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that respond to interactions.
pub trait ResponsiveAttribute: Loadable + TypePath
{
    /// Specifies the interactivity loadable used by this responsive.
    ///
    /// Can be used to hook target entities up to custom interactivity.
    ///
    /// For example [`Interactive`] (for `bevy_ui`).
    type Interactive: Default + ApplyLoadable;
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
pub trait AnimatableAttribute: Loadable + TypePath
{
    /// Specifies the interactivity loadable used by this animatable.
    ///
    /// Can be used to hook target entities up to custom interactivity.
    ///
    /// For example [`Interactive`] (for `bevy_ui`).
    type Interactive: Default + ApplyLoadable;
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for entities that control the dynamic styles of descendents.
///
/// This component must be manually added to entities, since it can't be reliably loaded due to race conditions
/// around entity updates in the loader. Specifically, it's possible for a child to
/// load its dynamic styles before its parent with `PropagateControl` is loaded, in which case the child's
/// styles would fail to load since they need to be saved in the propagator's theme.
#[derive(Component)]
pub struct PropagateControl;

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for theme values.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Themed<T: ThemedAttribute>
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a loaded theme.
    #[reflect(default)]
    pub state: Option<Vec<PseudoState>>,
    /// The value that will be applied to the entity with `T`.
    pub value: T::Value,
    /// Controls whether this value should be controlled by an ancestor with [`PropagateControl`].
    ///
    /// `false` by default.
    #[reflect(default)]
    pub inherit_control: bool,
}

impl<T: ThemedAttribute> ApplyLoadable for Themed<T>
{
    fn apply(self, ec: &mut EntityCommands)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
            CustomStaticStyleAttribute::new(extract_static_value::<T>(self.value)),
        ));

        let id = ec.id();
        ec.commands().syscall(
            (id, self.inherit_control, self.state, attribute),
            add_attribute_to_theme,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for responsive values.
///
/// Note that the `InteractiveVals::idle` field must always be set, which means it is effectively the 'default'
/// value for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Responsive<T: ResponsiveAttribute + ThemedAttribute>
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a loaded theme.
    #[reflect(default)]
    pub state: Option<Vec<PseudoState>>,
    /// The values that are toggled in response to interaction changes.
    pub values: InteractiveVals<T::Value>,
    /// Controls whether this value should be controlled by an ancestor with [`PropagateControl`].
    ///
    /// `false` by default.
    #[reflect(default)]
    pub inherit_control: bool,
}

impl<T: ResponsiveAttribute + ThemedAttribute> ApplyLoadable for Responsive<T>
{
    fn apply(self, ec: &mut EntityCommands)
    {
        if !self.inherit_control {
            T::Interactive::default().apply(ec);
        }

        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Interactive(InteractiveStyleAttribute::Custom(
            CustomInteractiveStyleAttribute::new(extract_responsive_value::<T>(self.values)),
        ));

        let id = ec.id();
        ec.commands().syscall(
            (id, self.inherit_control, self.state, attribute),
            add_attribute_to_theme,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for animatable values.
///
/// Note that the `AnimatedVals::idle` field must always be set, which means it is effectively the 'default' value
/// for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animated<T: AnimatableAttribute + ThemedAttribute>
where
    <T as ThemedAttribute>::Value: Lerp,
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
    /// Controls whether this value should be controlled by an ancestor with [`PropagateControl`].
    ///
    /// `false` by default.
    #[reflect(default)]
    pub inherit_control: bool,
}

impl<T: AnimatableAttribute + ThemedAttribute> ApplyLoadable for Animated<T>
where
    <T as ThemedAttribute>::Value: Lerp,
{
    fn apply(self, ec: &mut EntityCommands)
    {
        if !self.inherit_control {
            T::Interactive::default().apply(ec);
        }

        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::Custom(CustomAnimatedStyleAttribute::new(
                extract_animation_value::<T>(self.values),
            )),
            controller: DynamicStyleController::new(self.settings, AnimationState::default()),
        };

        let id = ec.id();
        ec.commands().syscall(
            (id, self.inherit_control, self.state, attribute),
            add_attribute_to_theme,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

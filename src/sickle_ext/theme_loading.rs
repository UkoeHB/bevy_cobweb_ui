use std::any::{type_name, TypeId};

use bevy::ecs::component::Components;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use sickle_ui::lerp::Lerp;
use sickle_ui::prelude::attribute::{
    CustomAnimatedStyleAttribute, CustomInteractiveStyleAttribute, CustomStaticStyleAttribute,
};
use sickle_ui::prelude::*;
use sickle_ui::theme::dynamic_style_attribute::{DynamicStyleAttribute, DynamicStyleController};
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::style_animation::{AnimationSettings, AnimationState};
use sickle_ui::theme::ThemeRegistry;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn theme_adder_fn<C: DefaultTheme + Component>(loaded_themes: &mut LoadedThemes) -> &mut LoadedTheme
{
    loaded_themes.add::<C>()
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
struct ThemeLoadContext
{
    /// Type id of the theme component of the theme that the entity is updating/just updated.
    marker: TypeId,
    /// Type name of the theme component.
    marker_name: &'static str,
    /// Context string for the sub-theme that is updating/just updated.
    context: Option<&'static str>,
    /// Type-erased callback for adding a theme to `LoadedThemes` if it's missing.
    theme_adder_fn: fn(&mut LoadedThemes) -> &mut LoadedTheme,
}

impl ThemeLoadContext
{
    fn add_theme<'a>(&self, loaded_themes: &'a mut LoadedThemes) -> &'a mut LoadedTheme
    {
        (self.theme_adder_fn)(loaded_themes)
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn add_attribute_to_dynamic_style_inner(
    entity: Entity,
    attribute: DynamicStyleAttribute,
    commands: &mut Commands,
    store_entity: Entity,
    maybe_style: Option<&mut DynamicStyle>,
)
{
    // Contextualize the attribute.
    let context = if entity != store_entity {
        Some(entity)
    } else {
        None
    };
    let contextual_attribute = ContextStyleAttribute::new(context, attribute);

    // Add this attribute directly.
    // - NOTE: If the entity has a themed component or is given a loaded theme at a later time, then these changes
    //   MAY be overwritten.
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

fn add_attribute_to_dynamic_style(
    In((entity, inherit_control, attribute)): In<(Entity, bool, DynamicStyleAttribute)>,
    mut commands: Commands,
    parents: Query<&Parent>,
    mut query: Query<(Entity, Option<&mut DynamicStyle>, Has<PropagateControl>)>,
)
{
    if !inherit_control {
        // Insert directly to this entity.
        let Some((store_entity, maybe_style, _)) = query.get_mut(entity).ok() else { return };
        add_attribute_to_dynamic_style_inner(
            entity,
            attribute,
            &mut commands,
            store_entity,
            maybe_style.map(|i| i.into_inner()),
        );
    } else {
        // Find parent where attribute should be saved.
        let mut count = 0;
        for parent in parents.iter_ancestors(entity) {
            count += 1;
            let Some((store_entity, maybe_style, has_propagate)) = query.get_mut(parent).ok() else {
                continue;
            };
            if !has_propagate {
                continue;
            }
            add_attribute_to_dynamic_style_inner(
                entity,
                attribute,
                &mut commands,
                store_entity,
                maybe_style.map(|i| i.into_inner()),
            );
            return;
        }

        if count > 0 {
            tracing::warn!("failed adding non-theme dynamic attribute with inherited interaction to {entity:?}, \
                no ancestor with PropagateControl");
        }
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
    theme_registry: Res<ThemeRegistry>,
    contexts: Query<&ThemeLoadContext>,
    parents: Query<&Parent>,
    components: &Components,
    mut params: ParamSet<(&World, Query<&mut LoadedThemes>, Commands)>,
)
{
    // Get load context for entity.
    let Ok(load_context) = contexts.get(entity) else {
        // Fall back to inserting as a plain dynamic style attribute.

        if let Some(state) = &state {
            if !state.is_empty() {
                tracing::error!("failed adding attribute to {entity:?}, pseudo states are not supported for non-theme \
                    dynamic sytle attributes (state: {:?}", state);
                return;
            }
        }

        tracing::debug!("no themes are loaded to {entity:?}, inserting attribute to dynamic style instead");
        params
            .p2()
            .syscall((entity, inherit_control, attribute), add_attribute_to_dynamic_style);

        return;
    };

    // Check that the theme was registered.
    if !theme_registry.contains_by_id(load_context.marker) {
        tracing::error!("failed adding attribute to theme for {entity:?}, the target theme {} was not registered (use \
            ComponentThemePlugin)", load_context.marker_name);
        return;
    }

    // Convert marker id to component id.
    let maybe_component_id = components.get_id(load_context.marker);

    // Find target entity and insert the attribute.
    for entity in [entity]
        .iter()
        .cloned()
        .chain(parents.iter_ancestors(entity))
    {
        // Check if the entity has the theme component.
        let has_theme_component = maybe_component_id
            .and_then(|component_id| params.p0().get_by_id(entity, component_id))
            .is_some();

        // Check if the entity has LoadedThemes with the theme component entry.
        if let Ok(mut loaded_themes) = params.p1().get_mut(entity) {
            // Check if the marker is known.
            if let Some(loaded_theme) = loaded_themes.get_mut(load_context.marker) {
                // Update the existing loaded themes.
                loaded_theme.set_attribute(state, load_context.context, attribute);
                return;
            }

            // Check if the entity has the theme component.
            if has_theme_component {
                // Insert to the existing loaded themes.
                let loaded_theme = load_context.add_theme(loaded_themes.into_inner());
                loaded_theme.set_attribute(state, load_context.context, attribute);
                return;
            }
        }

        // Check if the entity has the theme component.
        if has_theme_component {
            // Make new LoadedThemes with new theme and insert to the entity.
            let mut loaded_themes = LoadedThemes::new();
            let loaded_theme = load_context.add_theme(&mut loaded_themes);
            loaded_theme.set_attribute(state, load_context.context, attribute);
            let mut c = params.p2();
            let mut ec = c.entity(entity);
            ec.insert(loaded_themes);
            return;
        }
    }

    tracing::error!("failed adding attribute to theme for {entity:?}, could not find any ancestor with the theme \
        component or where the theme is loaded");
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_static_value<T: ThemedAttribute>(val: T::Value) -> impl Fn(Entity, &mut World)
{
    move |entity: Entity, world: &mut World| {
        // Apply the value to the entity.
        let mut c = world.commands();
        let Some(mut ec) = c.get_entity(entity) else { return };
        T::update(&mut ec, val.clone());
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
        let mut c = world.commands();
        let Some(mut ec) = c.get_entity(entity) else { return };
        T::update(&mut ec, new_value);
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
        let mut c = world.commands();
        let Some(mut ec) = c.get_entity(entity) else { return };
        T::update(&mut ec, new_value);
        world.flush();
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
///
/// Use [`Interactive`] to make an entity interactable.
pub trait ResponsiveAttribute: Loadable + TypePath {}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
///
/// Use [`Interactive`] to make an entity interactable.
pub trait AnimatableAttribute: Loadable + TypePath {}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for theme values.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Themed<T: ThemedAttribute>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this to become active.
    ///
    /// Only used if this struct is applied to an entity with a loaded theme.
    #[reflect(default)]
    pub state: Option<Vec<PseudoState>>,
    /// The value that will be applied to the entity with `T`.
    pub value: T::Value,
}

impl<T: ThemedAttribute> ApplyLoadable for Themed<T>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
{
    fn apply(self, ec: &mut EntityCommands)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
            CustomStaticStyleAttribute::new(extract_static_value::<T>(self.value)),
        ));

        let id = ec.id();
        ec.syscall((id, false, self.state, attribute), add_attribute_to_theme);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for responsive values.
///
/// Note that the `InteractiveVals::idle` field must always be set, which means it is effectively the 'default'
/// value for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Responsive<T: ResponsiveAttribute + ThemedAttribute>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
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
    /// Only used when this loadable is applied to a non-themed entity.
    ///
    /// `false` by default.
    #[reflect(default)]
    pub inherit_control: bool,
}

impl<T: ResponsiveAttribute + ThemedAttribute> ApplyLoadable for Responsive<T>
where
    <T as ThemedAttribute>::Value: GetTypeRegistration,
{
    fn apply(self, ec: &mut EntityCommands)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Interactive(InteractiveStyleAttribute::Custom(
            CustomInteractiveStyleAttribute::new(extract_responsive_value::<T>(self.values)),
        ));

        let id = ec.id();
        ec.syscall(
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
    <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration,
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
    /// Only used when this loadable is applied to a non-themed entity.
    ///
    /// `false` by default.
    #[reflect(default)]
    pub inherit_control: bool,
}

impl<T: AnimatableAttribute + ThemedAttribute> ApplyLoadable for Animated<T>
where
    <T as ThemedAttribute>::Value: Lerp + GetTypeRegistration,
{
    fn apply(self, ec: &mut EntityCommands)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::Custom(CustomAnimatedStyleAttribute::new(
                extract_animation_value::<T>(self.values),
            )),
            controller: DynamicStyleController::new(self.settings, AnimationState::default()),
        };

        let id = ec.id();
        ec.syscall(
            (id, self.inherit_control, self.state, attribute),
            add_attribute_to_theme,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ThemeLoadingEntityCommandsExt
{
    /// Sets `C` in the theme load context so manually-inserted themable attributes will be applied properly.
    fn set_theme<C: DefaultTheme>(&mut self) -> &mut Self;
    /// Sets `C` and `Ctx` in the theme load context so manually-inserted themable attributes will be applied
    /// properly.
    fn set_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self) -> &mut Self;
    /// Sets up the current entity to receive loadable theme data.
    ///
    /// This is useful if you want to add subthemes to a theme that doesn't need to use [`Self::load_them`] because
    /// it doesn't have any attributes on the root entity.
    fn prepare_theme<C: DefaultTheme>(&mut self) -> &mut Self;
    /// Loads [`Theme<C>`] into the current entity from the loadable reference.
    ///
    /// The [`Themed<T>`], [`Responsive<T>`], and [`Animated<T>`] loadable wrappers found at `loadable_ref` will
    /// insert attributes to the theme when they are loaded onto this entity.
    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> &mut Self;
    /// Loads context-bound subtheme attributes to the nearest ancestor entity that has `C` or `LoadedThemes` with
    /// an entry for `C`.
    ///
    /// The [`Themed<T>`], [`Responsive<T>`], and [`Animated<T>`] loadable wrappers found at `loadable_ref` will
    /// insert attributes to the theme for context `Ctx::type_name()` when they are loaded onto this entity.
    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> &mut Self;
}

impl ThemeLoadingEntityCommandsExt for EntityCommands<'_>
{
    fn set_theme<C: DefaultTheme>(&mut self) -> &mut Self
    {
        let marker = TypeId::of::<C>();
        let marker_name = type_name::<C>();
        self.insert(ThemeLoadContext {
            marker,
            marker_name,
            context: None,
            theme_adder_fn: theme_adder_fn::<C>,
        });
        self
    }

    fn set_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self) -> &mut Self
    {
        let marker = TypeId::of::<C>();
        let marker_name = type_name::<C>();
        self.insert(ThemeLoadContext {
            marker,
            marker_name,
            context: Some(Ctx::NAME),
            theme_adder_fn: theme_adder_fn::<C>,
        });
        self
    }

    fn prepare_theme<C: DefaultTheme>(&mut self) -> &mut Self
    {
        let entity = self.id();
        self.commands().add(AddLoadedTheme::<C>::new(entity));
        self
    }

    fn load_theme<C: DefaultTheme + Component>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        self.prepare_theme::<C>();
        self.load_with_context_setter(loadable_ref, |ec| {
            ec.set_theme::<C>();
        });
        self
    }

    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        self.load_with_context_setter(loadable_ref, |ec| {
            ec.set_subtheme::<C, Ctx>();
        });
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

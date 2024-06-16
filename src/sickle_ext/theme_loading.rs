use std::any::TypeId;

use bevy::ecs::component::Components;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
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

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_loadable<T: Default + ApplyLoadable>(ec: &mut EntityCommands)
{
    T::default().apply(ec);
}

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

fn set_context_for_load_theme<C: DefaultTheme + Component>(ec: &mut EntityCommands)
{
    let marker = TypeId::of::<C>();
    ec.insert(ThemeLoadContext { marker, context: None, theme_adder_fn: theme_adder_fn::<C> });
}

//-------------------------------------------------------------------------------------------------------------------

fn set_context_for_load_theme_with_context<C: DefaultTheme + Component, Ctx: TypeName>(ec: &mut EntityCommands)
{
    let marker = TypeId::of::<C>();
    ec.insert(ThemeLoadContext {
        marker,
        context: Some(Ctx::type_name()),
        theme_adder_fn: theme_adder_fn::<C>,
    });
}

//-------------------------------------------------------------------------------------------------------------------

struct PrepTargetFn(fn(&mut EntityCommands));

//-------------------------------------------------------------------------------------------------------------------

fn add_attribute_to_theme(
    In((entity, state, attribute, prep_target)): In<(
        Entity,
        Option<Vec<PseudoState>>,
        DynamicStyleAttribute,
        PrepTargetFn,
    )>,
    contexts: Query<&ThemeLoadContext>,
    parents: Query<&Parent>,
    components: &Components,
    mut params: ParamSet<(&World, Query<&mut LoadedThemes>, Commands)>,
)
{
    // Get load context for entity.
    //todo: if no load context, insert to DynamicStyles?
    let Ok(load_context) = contexts.get(entity) else {
        tracing::error!("failed adding attribute to theme for {entity:?}, no themes are loaded onto the entity (use \
            `entity.load_theme<MyTheme>(loadable_ref);` or a similar method)");
        return;
    };

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
                prep_target.0(&mut params.p2().entity(entity));
                return;
            }

            // Check if the entity has the theme component.
            if has_theme_component {
                // Insert to the existing loaded themes.
                let loaded_theme = load_context.add_theme(loaded_themes.into_inner());
                loaded_theme.set_attribute(state, load_context.context, attribute);
                prep_target.0(&mut params.p2().entity(entity));
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
            prep_target.0(&mut ec);
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
        //todo: avoid syscall by getting Commands directly from World (bevy v0.14)
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
        //todo: avoid syscall by getting Commands directly from World (bevy v0.14)
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
        //todo: avoid syscall by getting Commands directly from World (bevy v0.14)
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
        ec.syscall(
            (id, self.state, attribute, PrepTargetFn(|_: &mut EntityCommands| {})),
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
}

impl<T: ResponsiveAttribute + ThemedAttribute> ApplyLoadable for Responsive<T>
{
    fn apply(self, ec: &mut EntityCommands)
    {
        // Prepare an updated DynamicStyleAttribute.
        let attribute = DynamicStyleAttribute::Interactive(InteractiveStyleAttribute::Custom(
            CustomInteractiveStyleAttribute::new(extract_responsive_value::<T>(self.values)),
        ));

        let id = ec.id();
        ec.syscall(
            (id, self.state, attribute, PrepTargetFn(add_loadable::<T::Interactive>)),
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
}

impl<T: AnimatableAttribute + ThemedAttribute> ApplyLoadable for Animated<T>
where
    <T as ThemedAttribute>::Value: Lerp,
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
            (id, self.state, attribute, PrepTargetFn(add_loadable::<T::Interactive>)),
            add_attribute_to_theme,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ThemeLoadingEntityCommandsExt
{
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
    fn prepare_theme<C: DefaultTheme>(&mut self) -> &mut Self
    {
        let entity = self.id();
        self.commands().add(AddLoadedTheme::<C>::new(entity));
        self
    }

    fn load_theme<C: DefaultTheme + Component>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        self.prepare_theme::<C>();
        self.load_with_context_setter(loadable_ref, set_context_for_load_theme::<C>);
        self
    }

    fn load_subtheme<C: DefaultTheme, Ctx: TypeName>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        self.load_with_context_setter(loadable_ref, set_context_for_load_theme_with_context::<C, Ctx>);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

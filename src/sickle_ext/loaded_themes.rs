use std::any::{type_name, TypeId};
use std::marker::PhantomData;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use sickle_ui::theme::dynamic_style_attribute::DynamicStyleAttribute;
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::{DefaultTheme, DynamicStyleBuilder, PseudoTheme, Theme, ThemeUpdate};
use sickle_ui::ui_style::builder::StyleBuilder;
use sickle_ui::ui_style::LogicalEq;
use smallvec::SmallVec;

//-------------------------------------------------------------------------------------------------------------------

fn get_loaded_theme<C: Component>(
    style_builder: &mut StyleBuilder,
    source: Option<Entity>,
    state: &Option<Vec<PseudoState>>,
    entity: Entity,
    _context: &C,
    world: &World,
)
{
    // Get the pseudo theme being built.
    let Some(source) = source else {
        tracing::error!("build style for {entity:?} failed, there is unexpectedly no source entity {source:?} for theme {}",
            type_name::<C>());
        return;
    };
    let Some(loaded_themes) = world.get::<LoadedThemes>(source) else {
        tracing::error!("build style for {entity:?} failed, source {source:?} is missing LoadedThemes for theme {}",
            type_name::<C>());
        return;
    };
    let Some(loaded_theme) = loaded_themes.get(TypeId::of::<C>()) else {
        tracing::error!("build style for {entity:?} failed, source {source:?} is missing LoadedTheme for theme {}",
            type_name::<C>());
        return;
    };
    let Some(pseudo_theme) = loaded_theme.get(state) else {
        tracing::error!("build style for {entity:?} skipped, source {source:?} doesn't have pseudo theme for \
            theme {} for pseudo states {:?}", type_name::<C>(), state);
        return;
    };

    // Add all attributes from this pseudo theme to the style builder.
    for CachedContextualAttribute { placement, context, attribute } in pseudo_theme.style.iter() {
        // Set the placement.
        if let Some(placement) = placement {
            style_builder.switch_placement(placement);
        } else {
            style_builder.reset_placement();
        }

        // Set the target.
        if let Some(target) = context {
            style_builder.switch_target(target);
        } else {
            style_builder.reset_target();
        }

        // Insert attribute.
        style_builder.add(attribute.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CachedContextualAttribute
{
    placement: Option<&'static str>,
    context: Option<&'static str>,
    attribute: DynamicStyleAttribute,
}

impl LogicalEq for CachedContextualAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self.placement == other.placement
            && self.context == other.context
            && self.attribute.logical_eq(&other.attribute)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct EditablePseudoTheme
{
    state: Option<Vec<PseudoState>>,
    style: SmallVec<[CachedContextualAttribute; 3]>,
}

impl EditablePseudoTheme
{
    fn new(state: Option<Vec<PseudoState>>, attribute: CachedContextualAttribute) -> Self
    {
        let mut style = SmallVec::new();
        style.push(attribute);
        Self { state, style }
    }

    fn matches(&self, state: &Option<Vec<PseudoState>>) -> bool
    {
        self.state == *state
    }

    fn set_attribute(&mut self, attribute: CachedContextualAttribute)
    {
        // Merge attribute with existing list.
        if let Some(index) = self
            .style
            .iter()
            .position(|attr| attr.logical_eq(&attribute))
        {
            self.style[index] = attribute;
        } else {
            self.style.push(attribute);
        }
    }

    fn pseudo_theme<C: Component>(&self) -> PseudoTheme<C>
    {
        PseudoTheme::new(
            self.state.clone(),
            DynamicStyleBuilder::InfoWorldStyleBuilder(get_loaded_theme::<C>),
        )
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn refresh_loaded_theme<C: DefaultTheme>(
    pseudo_themes: &SmallVec<[EditablePseudoTheme; 1]>,
    ec: &mut EntityCommands,
)
{
    let themes: Vec<PseudoTheme<C>> = pseudo_themes
        .iter()
        .map(EditablePseudoTheme::pseudo_theme::<C>)
        .collect();
    ec.try_insert(Theme::<C>::new(themes));
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct LoadedTheme
{
    /// Indicates if the theme has changed since it was last refreshed.
    is_dirty: bool,

    /// Type id of the component that marks this theme.
    theme_marker: TypeId,

    /// Callback used to refresh a loaded theme on an entity.
    refresh: fn(&SmallVec<[EditablePseudoTheme; 1]>, &mut EntityCommands),

    /// Themes that are loaded from file.
    pseudo_themes: SmallVec<[EditablePseudoTheme; 1]>,
}

impl LoadedTheme
{
    fn new<C: DefaultTheme>() -> Self
    {
        Self {
            is_dirty: true,
            theme_marker: TypeId::of::<C>(),
            refresh: refresh_loaded_theme::<C>,
            pseudo_themes: SmallVec::default(),
        }
    }

    fn matches(&self, marker: TypeId) -> bool
    {
        self.theme_marker == marker
    }

    pub(crate) fn set_attribute(
        &mut self,
        mut state: Option<Vec<PseudoState>>,
        placement: Option<&'static str>,
        context: Option<&'static str>,
        attribute: DynamicStyleAttribute,
    )
    {
        self.is_dirty = true;

        if let Some(states) = state.as_deref_mut() {
            states.sort_unstable();
        }

        let attribute = CachedContextualAttribute { placement, context, attribute };
        match self.pseudo_themes.iter_mut().find(|t| t.matches(&state)) {
            Some(pseudo_theme) => pseudo_theme.set_attribute(attribute),
            None => self
                .pseudo_themes
                .push(EditablePseudoTheme::new(state, attribute)),
        }
    }

    fn get(&self, state: &Option<Vec<PseudoState>>) -> Option<&EditablePseudoTheme>
    {
        self.pseudo_themes.iter().find(|t| t.matches(state))
    }

    fn refresh(&mut self, ec: &mut EntityCommands)
    {
        if !self.is_dirty {
            return;
        }
        (self.refresh)(&self.pseudo_themes, ec);
        self.is_dirty = false;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores themes loaded to an entity.
#[derive(Component, Debug)]
pub struct LoadedThemes
{
    themes: SmallVec<[LoadedTheme; 1]>,
}

impl LoadedThemes
{
    /// Makes a new loaded themes.
    pub fn new() -> Self
    {
        Self { themes: SmallVec::default() }
    }

    /// Makes a new loaded themes for a specific theme.
    pub fn new_with<C: DefaultTheme + Component>() -> Self
    {
        Self { themes: SmallVec::from_elem(LoadedTheme::new::<C>(), 1) }
    }

    /// Adds a theme if it's missing.
    pub(crate) fn add<'a, C: DefaultTheme>(&'a mut self) -> &'a mut LoadedTheme
    {
        let marker = TypeId::of::<C>();
        let index = match self.themes.iter().position(|t| t.matches(marker)) {
            Some(index) => index,
            None => {
                self.themes.push(LoadedTheme::new::<C>());
                self.themes.len() - 1
            }
        };
        &mut self.themes[index]
    }

    /// Gets an internal theme.
    pub(crate) fn get_mut(&mut self, marker: TypeId) -> Option<&mut LoadedTheme>
    {
        self.themes.iter_mut().find(|t| t.matches(marker))
    }

    /// Gets an internal theme.
    fn get(&self, marker: TypeId) -> Option<&LoadedTheme>
    {
        self.themes.iter().find(|t| t.matches(marker))
    }

    /// Refreshes all dirty themes, which means converting them to [`Theme<C>`] and inserting to the entity.
    fn refresh(&mut self, ec: &mut EntityCommands)
    {
        for theme in self.themes.iter_mut() {
            theme.refresh(ec);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Command for calling `LoadedThemes::add` on an entity.
///
/// Inserts a [`LoadedThemes`] component if the entity doesn't have one.
pub struct AddLoadedTheme<C: DefaultTheme>
{
    entity: Entity,
    _p: PhantomData<C>,
}

impl<C: DefaultTheme> AddLoadedTheme<C>
{
    pub fn new(entity: Entity) -> Self
    {
        Self { entity, _p: PhantomData }
    }
}

impl<C: DefaultTheme> Command for AddLoadedTheme<C>
{
    fn apply(self, world: &mut World)
    {
        let Some(mut entity) = world.get_entity_mut(self.entity) else { return };
        let Some(mut themes) = entity.get_mut::<LoadedThemes>() else {
            entity.insert(LoadedThemes::new_with::<C>());
            return;
        };
        themes.add::<C>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Refreshes loaded themes that have changed.
fn refresh_loaded_themes(mut c: Commands, mut q: Query<(Entity, &mut LoadedThemes), Changed<LoadedThemes>>)
{
    for (entity, mut themes) in q.iter_mut() {
        themes.refresh(&mut c.entity(entity));
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadedThemesPlugin;

impl Plugin for LoadedThemesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(PostUpdate, refresh_loaded_themes.before(ThemeUpdate));
    }
}

//-------------------------------------------------------------------------------------------------------------------

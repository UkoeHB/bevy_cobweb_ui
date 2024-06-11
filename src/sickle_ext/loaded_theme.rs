use std::any::TypeId;
use std::marker::PhantomData;

use bevy::ecs::system::{Command, EntityCommands};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::theme::dynamic_style::{ContextStyleAttribute, DynamicStyle};
use sickle_ui::theme::pseudo_state::PseudoState;
use sickle_ui::theme::{DefaultTheme, DynamicStyleBuilder, PseudoTheme, Theme};
use smallvec::SmallVec;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct EditablePseudoTheme
{
    state: Option<Vec<PseudoState>>,
    style: DynamicStyle,
}

impl EditablePseudoTheme
{
    fn new(state: Option<Vec<PseudoState>>, attribute: ContextStyleAttribute) -> Self
    {
        Self { state, style: DynamicStyle::copy_from(vec![attribute]) }
    }
}

impl Into<PseudoTheme> for EditablePseudoTheme
{
    fn into(self) -> PseudoTheme
    {
        PseudoTheme::new(self.state, DynamicStyleBuilder::Static(self.style))
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn refresh_loaded_theme<C: DefaultTheme>(
    mut pseudo_themes: SmallVec<[EditablePseudoTheme; 1]>,
    ec: &mut EntityCommands,
)
{
    let themes: Vec<PseudoTheme> = pseudo_themes.drain(..).map(|t| t.into()).collect();
    ec.insert(Theme::<C>::new(themes));
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct LoadedTheme
{
    /// Type id of the component that marks this theme.
    theme_marker: TypeId,

    /// Callback used to refresh a loaded theme on an entity.
    refresh: fn(SmallVec<[EditablePseudoTheme; 1]>, &mut EntityCommands),

    /// Themes that are loaded from file.
    pseudo_themes: SmallVec<[EditablePseudoTheme; 1]>,
}

impl LoadedTheme
{
    fn new<C: DefaultTheme>() -> Self
    {
        Self {
            theme_marker: TypeId::of::<C>(),
            refresh: refresh_loaded_theme::<C>,
            pseudo_themes: SmallVec::default(),
        }
    }

    fn matches(&self, marker: TypeId) -> bool
    {
        self.theme_marker == marker
    }

    fn update(&mut self, mut state: Option<Vec<PseudoState>>, attribute: ContextStyleAttribute)
    {
        if let Some(states) = state.as_deref_mut() {
            states.sort_unstable();
        }

        match self.pseudo_themes.iter_mut().find(|t| t.state == state) {
            Some(pseudo_theme) => {
                let mut temp = DynamicStyle::new(Vec::default());
                std::mem::swap(&mut pseudo_theme.style, &mut temp);
                pseudo_theme.style = temp.merge(DynamicStyle::copy_from(vec![attribute]));
            }
            None => self
                .pseudo_themes
                .push(EditablePseudoTheme::new(state, attribute)),
        }
    }

    fn refresh(&self, ec: &mut EntityCommands)
    {
        (self.refresh)(self.pseudo_themes.clone(), ec);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn set_context_for_load_theme<C: Component>(ec: &mut EntityCommands)
{
    let entity = ec.id();
    let marker = TypeId::of::<C>();
    ec.commands().add(SetActiveLoadedTheme { entity, marker });
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores themes loaded to an entity.
///
/// Multiple themes can be loaded, but only one can be 'active' at a time. [`Self::update`] will add style
/// attributes to the active theme.
#[derive(Component)]
pub struct LoadedThemes
{
    /// Records which saved theme is currently active.
    ///
    /// Theme updates will be applied to the active theme.
    active_theme: usize,
    /// Saved themes.
    themes: SmallVec<[LoadedTheme; 1]>,
}

impl LoadedThemes
{
    /// Makes a new loaded themes for a specific theme.
    pub fn new<C: DefaultTheme>() -> Self
    {
        Self {
            active_theme: 0,
            themes: SmallVec::from_elem(LoadedTheme::new::<C>(), 1),
        }
    }

    /// Adds a theme if it's missing and updates the active theme index.
    pub fn add<C: DefaultTheme>(&mut self)
    {
        let marker = TypeId::of::<C>();
        match self.themes.iter().position(|t| t.matches(marker)) {
            Some(index) => {
                self.active_theme = index;
            }
            None => {
                self.themes.push(LoadedTheme::new::<C>());
                self.active_theme = self.themes.len() - 1;
            }
        }
    }

    /// Sets the active theme.
    pub fn set_active(&mut self, marker: TypeId)
    {
        let Some(index) = self.themes.iter().position(|t| t.matches(marker)) else {
            tracing::warn!("failed setting active loaded theme, unknown theme marker {:?}", marker);
            return;
        };
        self.active_theme = index;
    }

    /// Updates the active theme with a specific style attribute.
    pub fn update(&mut self, state: Option<Vec<PseudoState>>, attribute: ContextStyleAttribute)
    {
        self.themes[self.active_theme].update(state, attribute);
    }

    /// Refreshes the active theme, which means converting it to a [`Theme<C>`] and inserting it to the entity.
    fn refresh(&self, ec: &mut EntityCommands)
    {
        self.themes[self.active_theme].refresh(ec);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Command for calling [`LoadedThemes::add`] on an entity.
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
            entity.insert(LoadedThemes::new::<C>());
            return;
        };
        themes.add::<C>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Command for calling [`LoadedThemes::set_active`] on an entity.
pub struct SetActiveLoadedTheme
{
    pub entity: Entity,
    pub marker: TypeId,
}

impl SetActiveLoadedTheme
{
    pub fn new<C: Component>(entity: Entity) -> Self
    {
        Self { entity, marker: TypeId::of::<C>() }
    }
}

impl Command for SetActiveLoadedTheme
{
    fn apply(self, world: &mut World)
    {
        let Some(mut entity) = world.get_entity_mut(self.entity) else { return };
        let Some(mut themes) = entity.get_mut::<LoadedThemes>() else { return };
        themes.set_active(self.marker);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Command for applying an entity's current active loaded theme as a [`Theme<C>`] component on the entity.
pub struct RefreshLoadedTheme
{
    pub entity: Entity,
}

impl Command for RefreshLoadedTheme
{
    fn apply(self, world: &mut World)
    {
        world.syscall(
            self.entity,
            |In(entity): In<Entity>, mut c: Commands, q: Query<&LoadedThemes>| {
                let Ok(themes) = q.get(entity) else { return };
                themes.refresh(&mut c.entity(entity));
            },
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait LoadedThemeEntityCommandsExt
{
    /// Loads [`Theme<C>`] into the current entity from the loadable reference.
    ///
    /// After this is called, the theme will be 'active' on the entity, which means it can be updated with
    /// [`LoadedThemes::update`]. The [`Themed<T>`], [`Responsive<T>`], and [`Animated<T>`] loadable wrappers will
    /// call update automatically when applied to an entity with [`ApplyLoadable::apply`].
    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> &mut Self;
}

impl LoadedThemeEntityCommandsExt for EntityCommands<'_>
{
    fn load_theme<C: DefaultTheme>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        let entity = self.id();
        self.commands().add(AddLoadedTheme::<C>::new(entity));
        self.load_with_context_setter(loadable_ref, set_context_for_load_theme::<C>);
        self.commands().add(RefreshLoadedTheme { entity });
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

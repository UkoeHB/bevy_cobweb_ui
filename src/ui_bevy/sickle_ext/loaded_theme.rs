
//-------------------------------------------------------------------------------------------------------------------

struct EditablePseudoTheme
{
    state: Option<Vec<PseudoState>>,
    style: DynamicStyle,
}

impl EditablePseudoTheme
{
    fn new(state: Option<Vec<PseudoState>>, attribute: DynamicStyleAttribute) -> Self
    {
        Self{ state, style: DynamicStyle::new(vec![attribute]) }
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

fn refresh_loaded_theme<C: Component>(entity: Entity, pseudo_themes: SmallVec<[EditablePseudoTheme; 1]>, world: &mut World)
{
    let Some(entity) = world.get_entity(entity) else { return };
    let themes = pseudo_themes.drain(..).map(EditablePseudoTheme::into).collect();
    entity.insert(Theme::<C>::new(themes));
}

//-------------------------------------------------------------------------------------------------------------------

struct LoadedTheme
{
    /// Type id of the component that marks this theme.
    theme_marker: TypeId,

    /// Callback used to refresh a loaded theme on an entity.
    refresh: fn(Entity, SmallVec<[EditablePseudoTheme; 1]>, &mut World),

    /// Themes that are loaded from file.
    pseudo_themes: SmallVec<[EditablePseudoTheme; 1]>,
}

impl LoadedTheme
{
    fn new<C: Component>() -> Self
    {
        Self{ theme_marker: TypeId::of::<C>(), refresh: refresh_loaded_theme::<C>, pseudo_themes: Vec::default() }
    }

    fn matches(&self, marker: TypeId) -> bool
    {
        self.theme_marker == marker
    }

    fn update(
        &mut self,
        state: Option<Vec<PseudoState>>,
        attribute: DynamicStyleAttribute,
    )
    {
        match pseudo_themes.iter().find() {
            Some(pseudo_theme) => pseudo_theme.style.merge(vec![attribute]),
            None => pseudo_themes.push(EditablePseudoTheme::new(state, attribute)),
        }
    }

    fn refresh(&self, entity: Entity, world: &mut World)
    {
        (self.refresh)(entity, self.pseudo_themes.clone(), world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn set_context_for_load_theme<C: Component>(ec: &mut EntityCommands)
{
    let entity = ec.id();
    let marker = TypeId::of::<C>();
    ec.commands().add(SetActiveLoadedTheme{ entity, marker });
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
pub(crate) struct LoadedThemes
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
    pub(crate) fn new<C: Component>() -> Self
    {
        Self{
            active_theme: 0,
            themes: SmallVec::new(LoadedTheme::new::<C>()),
        }
    }

    /// Adds a theme if it's missing and updates the active theme index.
    pub(crate) fn add<C: Component>(&mut self) -> Self
    {
        let marker = TypeId::of::<C>();
        match self.themes.iter().position(|t| t.matches(marker))
        {
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
    pub(crate) fn set_active(&mut self, marker: TypeId)
    {
        let Some(index) = self.themes.iter().position(|t| t.matches(marker)) else {
            tracing::warn!("failed setting active loaded theme, unknown theme marker {:?}", marker);
        };
        self.active_theme = index;
    }

    /// Updates the active theme.
    pub(crate) fn update(
        &mut self,
        state: Option<Vec<PseudoState>>,
        attribute: DynamicStyleAttribute,
    )
    {
        self.themes[self.active_index].update(state, attribute);
    }

    fn refresh(&self, entity: Entity, world: &mut World)
    {
        self.themes[self.active_index].refresh(entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct AddLoadedTheme<C: Component>
{
    entity: Entity,
    _p: PhantomData<C>,
}

impl<C: Component> AddLoadedTheme<C>
{
    pub(crate) fn new(entity: Entity) -> Self
    {
        Self{ entity, _p: PhantomData::default() }
    }
}

impl Command for AddLoadedTheme
{
    fn apply(self, world: &mut World)
    {
        let Some(entity) = world.get_entity(self.entity) else { return };
        let Some(themes) = entity.get::<LoadedThemes>() else {
            entity.insert(LoadedThemes::new::<C>());
            return
        };
        themes.add::<C>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SetActiveLoadedTheme
{
    pub(crate) entity: Entity,
    pub(crate) marker: TypeId,
}

impl Command for SetActiveLoadedTheme
{
    fn apply(self, world: &mut World)
    {
        let Some(entity) = world.get_entity(self.entity) else { return };
        let Some(themes) = entity.get::<LoadedThemes>() else { return };
        themes.set_active(self.marker);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct RefreshLoadedTheme
{
    pub(crate) entity: Entity,
}

impl Command for RefreshLoadedTheme
{
    fn apply(self, world: &mut World)
    {
        let Some(entity) = world.get_entity(self.entity) else { return };
        let Some(themes) = entity.get::<LoadedThemes>() else { return };
        themes.refresh(self.entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait LoadedThemeEntityCommandsExt
{
    fn load_theme<C: Component>(&mut self, loadable_ref: LoadableRef) -> &mut Self;
}

impl LoadedThemeEntityCommandsExt for EntityCommand<'_>
{
    fn load_theme<C: Component>(&mut self, loadable_ref: LoadableRef) -> &mut Self
    {
        let id = self.id();
        self.commands.add(AddLoadedTheme::<C>::new(id));
        self.load_with_context_setter(loadable_ref, set_context_for_load_theme::<C>);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

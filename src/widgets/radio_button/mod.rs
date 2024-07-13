use std::any::type_name;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::prelude::*;
use smallvec::SmallVec;

use crate::load_embedded_widget;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Coordinates toggling of radio buttons.
///
/// See [`RadioButtonBuilder::build`].
#[derive(Component, Default, Debug)]
pub struct RadioButtonManager
{
    selected: Option<Entity>,
}

impl RadioButtonManager
{
    /// Inserts a new manager onto the builder entity.
    ///
    /// Returns the entity where the manager is stored.
    pub fn insert(node: &mut UiBuilder<Entity>) -> Entity
    {
        node.insert(Self::default());
        node.id()
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(TypeName)]
pub struct RadioButtonIndicator;
#[derive(TypeName)]
pub struct RadioButtonIndicatorDot;
#[derive(TypeName)]
pub struct RadioButtonContent;

/// Marker component for the radio button theme.
#[derive(Component, DefaultTheme, Clone, Debug)]
pub struct RadioButton
{
    indicator_entity: Entity,
    indicator_dot_entity: Entity,
    content_entity: Entity,
    content_inner_entities: SmallVec<[(&'static str, Entity); 3]>,
}

impl UiContext for RadioButton
{
    fn get(&self, target: &str) -> Result<Entity, String>
    {
        match target {
            RadioButtonIndicator::NAME => Ok(self.indicator_entity),
            RadioButtonIndicatorDot::NAME => Ok(self.indicator_dot_entity),
            RadioButtonContent::NAME => Ok(self.content_entity),
            _ => {
                let Some((_, entity)) = self
                    .content_inner_entities
                    .iter()
                    .find(|(name, _)| *name == target)
                else {
                    return Err(format!("unknown UI context {target} for {}", type_name::<Self>()));
                };
                Ok(*entity)
            }
        }
    }
    fn contexts(&self) -> Vec<&'static str>
    {
        Vec::from_iter(
            [RadioButtonIndicator::NAME, RadioButtonIndicatorDot::NAME, RadioButtonContent::NAME]
                .iter()
                .map(|name| *name)
                .chain(self.content_inner_entities.iter().map(|(name, _)| *name)),
        )
    }
}

//-------------------------------------------------------------------------------------------------------------------

enum RadioButtonType
{
    Default
    {
        text: Option<String>,
    },
    DefaultInBox
    {
        text: Option<String>,
    },
    Custom(LoadableRef),
    CustomWithText
    {
        loadable: LoadableRef,
        text: Option<String>,
    },
}

impl RadioButtonType
{
    fn get_scene(&self) -> LoadableRef
    {
        match self {
            Self::Default { .. } => LoadableRef::new("builtin.widgets.radio_button", "radio_button_default"),
            Self::DefaultInBox { .. } => {
                LoadableRef::new("builtin.widgets.radio_button", "radio_button_default_in_vertical_box")
            }
            Self::Custom(loadable) => loadable.clone(),
            Self::CustomWithText { loadable, .. } => loadable.clone(),
        }
    }

    fn take_text(self) -> Option<String>
    {
        match self {
            Self::Default { text } | Self::DefaultInBox { text } | Self::CustomWithText { text, .. } => text,
            Self::Custom(..) => None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Builds a [`RadioButton`] widget into an entity.
pub struct RadioButtonBuilder
{
    button_type: RadioButtonType,
    localized: bool,
}

impl RadioButtonBuilder
{
    pub fn default() -> Self
    {
        Self {
            button_type: RadioButtonType::Default { text: None },
            localized: false,
        }
    }

    pub fn default_in_box() -> Self
    {
        Self {
            button_type: RadioButtonType::DefaultInBox { text: None },
            localized: false,
        }
    }

    pub fn custom(scene: LoadableRef) -> Self
    {
        Self {
            button_type: RadioButtonType::Custom(scene),
            localized: false,
        }
    }

    pub fn custom_with_text(scene: LoadableRef, text: impl Into<String>) -> Self
    {
        Self {
            button_type: RadioButtonType::CustomWithText { loadable: scene, text: Some(text.into()) },
            localized: false,
        }
    }

    pub fn new(text: impl Into<String>) -> Self
    {
        Self {
            button_type: RadioButtonType::Default { text: Some(text.into()) },
            localized: false,
        }
    }

    pub fn new_in_box(text: impl Into<String>) -> Self
    {
        Self {
            button_type: RadioButtonType::DefaultInBox { text: Some(text.into()) },
            localized: false,
        }
    }

    /// Cause the text to be localized.
    ///
    /// Mainly useful for default-themed radio buttons, since custom buttons can include [`LocalizedText`]
    /// components directly.
    pub fn localized(mut self) -> Self
    {
        self.localized = true;
        self
    }

    /// Builds the button as a child of the builder entity.
    ///
    /// The `manager_entity` should have a [`RadioButtonManager`] component.
    ///
    /// If you want to add children to the content entity with [`Animated`] or [`Responsive`], then
    /// use [`Self::build_with_themed_content`] instead.
    pub fn build<'a>(self, manager_entity: Entity, node: &'a mut UiBuilder<Entity>) -> UiBuilder<'a, Entity>
    {
        self.build_with_themed_content(manager_entity, node, |_| SmallVec::default())
    }

    /// Builds the button as a child of the builder entity, with custom themed content.
    ///
    /// The `manager_entity` should have a [`RadioButtonManager`] component.
    ///
    /// Load your content sub-entities with `.load_with_subtheme::<RadioButton, YourSubtheme>()`.
    /// Otherwise your sub-entities won't respond properly to interactions on the base button.
    ///
    /// The `content_builder` should return [`UiContext`] entries for themed sub-entities.
    pub fn build_with_themed_content<'a>(
        self,
        manager_entity: Entity,
        node: &'a mut UiBuilder<Entity>,
        content_builder: impl FnOnce(&mut UiBuilder<Entity>) -> SmallVec<[(&'static str, Entity); 3]>,
    ) -> UiBuilder<'a, Entity>
    {
        let scene = self.button_type.get_scene();

        let mut base_entity = Entity::PLACEHOLDER;
        let mut indicator_entity = Entity::PLACEHOLDER;
        let mut indicator_dot_entity = Entity::PLACEHOLDER;
        let mut content_entity = Entity::PLACEHOLDER;
        let mut content_inner_entities = SmallVec::default();

        node.load_with_theme::<RadioButton>(scene.e("base"), &mut base_entity, |base, path| {
            let base_id = base.id();
            base
                // Select this button.
                // TODO: this callback could be moved to an EntityWorldReactor, with the manager entity as entity
                // data.
                .on_pressed(move |mut c: Commands, states: Query<&PseudoStates>| {
                    if let Ok(states) = states.get(base_id) {
                        if states.has(&PseudoState::Selected) {
                            return;
                        }
                    }

                    c.react().entity_event(base_id, Select);
                })
                // Save the newly-selected button and deselect the previously selected.
                .on_select(move |mut c: Commands, mut managers: Query<&mut RadioButtonManager>| {
                    let Ok(mut manager) = managers.get_mut(manager_entity) else { return };
                    if let Some(prev) = manager.selected {
                        c.react().entity_event(prev, Deselect);
                    }
                    manager.selected = Some(base_id);
                });

            base.load_with_subtheme::<RadioButton, RadioButtonIndicator>(
                path.e("indicator"),
                &mut indicator_entity,
                |outline, path| {
                    outline.load_with_subtheme::<RadioButton, RadioButtonIndicatorDot>(
                        path.e("indicator_dot"),
                        &mut indicator_dot_entity,
                        |_, _| {},
                    );
                },
            );

            base.load_with_subtheme::<RadioButton, RadioButtonContent>(
                path.e("content"),
                &mut content_entity,
                |content, _| {
                    // Add text if necessary.
                    if let Some(text) = self.button_type.take_text() {
                        // Note: The text needs to be updated on load otherwise it may be overwritten.
                        content.update_on((), |id| {
                            move |mut e: TextEditor| {
                                e.write(id, |t| write!(t, "{}", text.as_str()));
                            }
                        });
                    }

                    // Localize if necessary.
                    if self.localized {
                        content.insert(LocalizedText::default());
                    }

                    // Build contents.
                    content_inner_entities = (content_builder)(content);
                },
            );

            base.insert(RadioButton {
                indicator_entity,
                indicator_dot_entity,
                content_entity,
                content_inner_entities,
            });
        });

        // Return UiBuilder for root of button where interactions will be detected.
        node.commands().ui_builder(base_entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebRadioButtonPlugin;

impl Plugin for CobwebRadioButtonPlugin
{
    fn build(&self, app: &mut App)
    {
        load_embedded_widget!(app, "bevy_cobweb_ui", "src/widgets/radio_button", "radio_button.caf.json");
        app.add_plugins(ComponentThemePlugin::<RadioButton>::new());
    }
}

//-------------------------------------------------------------------------------------------------------------------

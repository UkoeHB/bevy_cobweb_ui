use std::any::type_name;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::prelude::*;
use smallvec::SmallVec;

use crate::load_embedded_widget;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
enum Indicator
{
    #[default]
    None,
    Prime,
    Reversed,
}

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
    content_entity: Entity,
    inner_entities: SmallVec<[(&'static str, Entity); 5]>,
}

impl UiContext for RadioButton
{
    fn get(&self, target: &str) -> Result<Entity, String>
    {
        match target {
            RadioButtonContent::NAME => Ok(self.content_entity),
            _ => {
                let Some((_, entity)) = self.inner_entities.iter().find(|(name, _)| *name == target) else {
                    return Err(format!("unknown UI context {target} for {}", type_name::<Self>()));
                };
                Ok(*entity)
            }
        }
    }
    fn contexts(&self) -> Vec<&'static str>
    {
        Vec::from_iter(
            [RadioButtonContent::NAME]
                .iter()
                .map(|name| *name)
                .chain(self.inner_entities.iter().map(|(name, _)| *name)),
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
    indicator: Indicator,
    localized: bool,
}

impl RadioButtonBuilder
{
    pub fn default() -> Self
    {
        Self {
            button_type: RadioButtonType::Default { text: None },
            indicator: Indicator::Prime, // Included by default
            localized: false,
        }
    }

    pub fn default_in_box() -> Self
    {
        Self {
            button_type: RadioButtonType::DefaultInBox { text: None },
            indicator: Indicator::Prime, // Included by default
            localized: false,
        }
    }

    /// Builds from a custom scene.
    ///
    /// Does NOT include an indicator. Use [`Self::with_indicator`].
    pub fn custom(scene: LoadableRef) -> Self
    {
        Self {
            button_type: RadioButtonType::Custom(scene),
            indicator: Indicator::None,
            localized: false,
        }
    }

    /// Builds from a custom scene with text.
    ///
    /// Does NOT include an indicator. Use [`Self::with_indicator`].
    pub fn custom_with_text(scene: LoadableRef, text: impl Into<String>) -> Self
    {
        Self {
            button_type: RadioButtonType::CustomWithText { loadable: scene, text: Some(text.into()) },
            indicator: Indicator::None,
            localized: false,
        }
    }

    pub fn new(text: impl Into<String>) -> Self
    {
        Self {
            button_type: RadioButtonType::Default { text: Some(text.into()) },
            indicator: Indicator::Prime, // Included by default
            localized: false,
        }
    }

    pub fn new_in_box(text: impl Into<String>) -> Self
    {
        Self {
            button_type: RadioButtonType::DefaultInBox { text: Some(text.into()) },
            indicator: Indicator::Prime, // Included by default
            localized: false,
        }
    }

    /// Include an indicator dot in the button to the left/top of the content.
    ///
    /// Use [`Self::with_indicator_rev`] if you want the button to the right/bottom of the content.
    pub fn with_indicator(mut self) -> Self
    {
        self.indicator = Indicator::Prime;
        self
    }

    /// Include an indicator dot in the button to the right/bottom of the content.
    ///
    /// Use [`Self::with_indicator`] if you want the button to the left/top of the content.
    pub fn with_indicator_rev(mut self) -> Self
    {
        self.indicator = Indicator::Reversed;
        self
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
        content_builder: impl FnOnce(&mut UiBuilder<Entity>) -> SmallVec<[(&'static str, Entity); 10]>,
    ) -> UiBuilder<'a, Entity>
    {
        let scene = self.button_type.get_scene();

        let mut base_entity = Entity::PLACEHOLDER;
        let mut content_entity = Entity::PLACEHOLDER;
        let mut inner_entities = SmallVec::default();

        node.load_with_theme::<RadioButton>(scene.e("base"), &mut base_entity, |base, path| {
            let base_id = base.id();

            // Setup behavior.
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

            // Add a dot if requested. This dot will be before the content (to the right/bottom).
            if self.indicator == Indicator::Prime {
                Self::add_indicator(base, &path, &mut inner_entities);
            }

            // Add the content.
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
                    let content_entities = (content_builder)(content);
                    inner_entities.extend(content_entities);
                },
            );

            // Add a dot if requested. This dot will be after the content (to the right/bottom).
            if self.indicator == Indicator::Reversed {
                Self::add_indicator(base, &path, &mut inner_entities);
            }

            base.insert(RadioButton { content_entity, inner_entities });
        });

        // Return UiBuilder for root of button where interactions will be detected.
        node.commands().ui_builder(base_entity)
    }

    fn add_indicator(
        node: &mut UiBuilder<Entity>,
        path: &LoadableRef,
        inner_entities: &mut SmallVec<[(&'static str, Entity); 5]>,
    )
    {
        let mut indicator_entity = Entity::PLACEHOLDER;
        let mut indicator_dot_entity = Entity::PLACEHOLDER;

        node.load_with_subtheme::<RadioButton, RadioButtonIndicator>(
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

        inner_entities.push((RadioButtonIndicator::NAME, indicator_entity));
        inner_entities.push((RadioButtonIndicatorDot::NAME, indicator_dot_entity));
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

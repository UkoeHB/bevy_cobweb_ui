use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//use crate::load_embedded_scene_file;
use crate::prelude::*;
use crate::sickle::*;

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

    /// Deselects the previous entity and saves the next selected.
    ///
    /// Does not *select* the next entity, which is assumed to already be selected.
    pub fn swap_selected(&mut self, c: &mut Commands, next: Entity)
    {
        if let Some(prev) = self.selected {
            c.react().entity_event(prev, Deselect);
        }
        self.selected = Some(next);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(TypeName)]
pub struct RadioButton;
#[derive(TypeName)]
pub struct RadioButtonIndicator;
#[derive(TypeName)]
pub struct RadioButtonIndicatorDot;
#[derive(TypeName)]
pub struct RadioButtonContent;

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
    Custom(SceneRef),
    CustomWithText
    {
        loadable: SceneRef,
        text: Option<String>,
    },
}

impl RadioButtonType
{
    fn get_scene(&self) -> SceneRef
    {
        match self {
            Self::Default { .. } => SceneRef::new("builtin.widgets.radio_button", "radio_button_default"),
            Self::DefaultInBox { .. } => {
                SceneRef::new("builtin.widgets.radio_button", "radio_button_default_in_vertical_box")
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
    pub fn custom(scene: SceneRef) -> Self
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
    pub fn custom_with_text(scene: SceneRef, text: impl Into<String>) -> Self
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
        self.build_with_themed_content(manager_entity, node, |_| {})
    }

    /// Builds the button as a child of the builder entity, with custom themed content.
    ///
    /// The `manager_entity` should have a [`RadioButtonManager`] component.
    ///
    /// Load your content sub-entities with `.load_with_subtheme::<RadioButton, YourSubtheme>()`.
    /// Otherwise your sub-entities won't respond properly to interactions on the base button.
    pub fn build_with_themed_content<'a>(
        self,
        manager_entity: Entity,
        node: &'a mut UiBuilder<Entity>,
        content_builder: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<'a, Entity>
    {
        let scene = self.button_type.get_scene();

        let mut base_entity = Entity::PLACEHOLDER;

        node.load(scene + "base", |base, path| {
            base_entity = base.id();

            // Setup behavior.
            base
                // Select this button.
                // TODO: this callback could be moved to an EntityWorldReactor, with the manager entity as entity
                // data.
                .on_pressed(move |mut c: Commands, states: PseudoStateParam| {
                    states.try_select(base_entity, &mut c);
                })
                // Save the newly-selected button and deselect the previously selected.
                .on_select(move |mut c: Commands, mut managers: Query<&mut RadioButtonManager>| {
                    let Ok(mut manager) = managers.get_mut(manager_entity) else { return };
                    manager.swap_selected(&mut c, base_entity);
                });

            // Add a dot if requested. This dot will be before the content (to the left/top).
            if self.indicator == Indicator::Prime {
                Self::add_indicator(base, &path);
            }

            // Add the content.
            base.load(&path + "content", |content, _| {
                // Localize if necessary.
                if self.localized {
                    content.insert(LocalizedText::default());
                }

                // Add text if necessary.
                if let Some(text) = self.button_type.take_text() {
                    // Note: The text needs to be updated on load otherwise it may be overwritten.
                    content.update_on((), |id| {
                        move |mut e: TextEditor| {
                            e.write(id, |t| write!(t, "{}", text.as_str()));
                        }
                    });
                }

                // Build contents.
                (content_builder)(content);
            });

            // Add a dot if requested. This dot will be after the content (to the right/bottom).
            if self.indicator == Indicator::Reversed {
                Self::add_indicator(base, &path);
            }
        });

        // Return UiBuilder for root of button where interactions will be detected.
        node.commands().ui_builder(base_entity)
    }

    fn add_indicator(node: &mut UiBuilder<Entity>, path: &SceneRef)
    {
        node.load(path + "indicator", |outline, path| {
            outline.load(path + "indicator_dot", |_, _| {});
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebRadioButtonPlugin;

impl Plugin for CobwebRadioButtonPlugin
{
    fn build(&self, _app: &mut App)
    {
        // TODO: re-enable once COB scene macros are implemented
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/widgets/radio_button",
        // "radio_button.cob.json");
    }
}

//-------------------------------------------------------------------------------------------------------------------

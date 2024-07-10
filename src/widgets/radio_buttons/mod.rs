use std::any::type_name;

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use sickle_ui::prelude::*;

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
pub struct RadioButtonOutline;
#[derive(TypeName)]
pub struct RadioButtonIndicator;
#[derive(TypeName)]
pub struct RadioButtonText;

/// Marker component for the radio button theme.
#[derive(Component, DefaultTheme, Copy, Clone, Debug)]
pub struct RadioButton
{
    outline_entity: Entity,
    indicator_entity: Entity,
    text_entity: Entity,
}

impl RadioButton
{
    pub fn load_base_theme(builder: &mut UiBuilder<Entity>)
    {
        let theme = LoadableRef::new(RadioButtonBuilder::default_file(), "theme");
        builder.load_theme::<RadioButton>(theme.e("core"));
        builder.load_subtheme::<RadioButton, RadioButtonOutline>(theme.e("outline"));
        builder.load_subtheme::<RadioButton, RadioButtonIndicator>(theme.e("indicator"));
        builder.load_subtheme::<RadioButton, RadioButtonText>(theme.e("text"));
    }
}

impl UiContext for RadioButton
{
    fn get(&self, target: &str) -> Result<Entity, String>
    {
        match target {
            RadioButtonOutline::NAME => Ok(self.outline_entity),
            RadioButtonIndicator::NAME => Ok(self.indicator_entity),
            RadioButtonText::NAME => Ok(self.text_entity),
            _ => Err(format!("unknown UI context {target} for {}", type_name::<Self>())),
        }
    }
    fn contexts(&self) -> Vec<&'static str>
    {
        vec![RadioButtonOutline::NAME, RadioButtonIndicator::NAME, RadioButtonText::NAME]
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Builds a [`RadioButton`] widget into an entity.
pub struct RadioButtonBuilder
{
    //todo: optional text (e.g. what if you want an image intead?)
    text: String,
}

impl RadioButtonBuilder
{
    pub fn new(text: impl Into<String>) -> Self
    {
        Self { text: text.into() }
    }

    pub fn default_file() -> &'static str
    {
        "widgets.radio_buttons"
    }

    /// Builds the button as a child of the builder entity.
    ///
    /// The `manager_entity` should have a [`RadioButtonManager`] component.
    pub fn build<'a>(self, manager_entity: Entity, node: &'a mut UiBuilder<Entity>) -> UiBuilder<'a, Entity>
    {
        let structure = LoadableRef::new(&Self::default_file(), "structure");

        let mut core_entity = Entity::PLACEHOLDER;
        let mut outline_entity = Entity::PLACEHOLDER;
        let mut indicator_entity = Entity::PLACEHOLDER;
        let mut text_entity = Entity::PLACEHOLDER;

        node.load_with_theme::<RadioButton>(structure.e("core"), &mut core_entity, |core, path| {
            let core_id = core.id();
            core
                // Select this button.
                // TODO: this callback could be moved to an EntityWorldReactor, with the manager entity as entity
                // data.
                .on_pressed(move |mut c: Commands, states: Query<&PseudoStates>| {
                    if let Ok(states) = states.get(core_id) {
                        if states.has(&PseudoState::Selected) {
                            return;
                        }
                    }

                    c.react().entity_event(core_id, Select);
                })
                // Save the newly-selected button and deselect the previously selected.
                .on_select(move |mut c: Commands, mut managers: Query<&mut RadioButtonManager>| {
                    let Ok(mut manager) = managers.get_mut(manager_entity) else { return };
                    if let Some(prev) = manager.selected {
                        c.react().entity_event(prev, Deselect);
                    }
                    manager.selected = Some(core_id);
                });

            core.load_with_subtheme::<RadioButton, RadioButtonOutline>(
                path.e("outline"),
                &mut outline_entity,
                |outline, path| {
                    outline.load_with_subtheme::<RadioButton, RadioButtonIndicator>(
                        path.e("indicator"),
                        &mut indicator_entity,
                        |_, _| {},
                    );
                },
            );

            core.load_with_subtheme::<RadioButton, RadioButtonText>(
                path.e("text"),
                &mut text_entity,
                |text, _| {
                    // Note: The text needs to be updated on load otherwise it may be overwritten.
                    let text_val = self.text;
                    text.update_on((), |id| {
                        move |mut e: TextEditor| {
                            e.write(id, |t| write!(t, "{}", text_val.as_str()));
                        }
                    });
                },
            );

            core.insert(RadioButton { outline_entity, indicator_entity, text_entity });
        });

        // Return UiBuilder for root of button where interactions will be detected.
        node.commands().ui_builder(core_entity)
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebRadioButtonsPlugin;

impl Plugin for CobwebRadioButtonsPlugin
{
    fn build(&self, app: &mut App)
    {
        embedded_asset!(app, "src/widgets/radio_buttons", "radio_buttons.caf.json");
        app.add_plugins(ComponentThemePlugin::<RadioButton>::new());
    }
}

//-------------------------------------------------------------------------------------------------------------------

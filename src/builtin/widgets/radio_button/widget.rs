use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//use crate::load_embedded_scene_file;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Coordinates toggling of radio buttons.
#[derive(Component, Default)]
struct RadioButtonManager
{
    selected: Option<Entity>,
}

impl RadioButtonManager
{
    /// Deselects the previous entity and saves the next selected.
    ///
    /// Does not *select* the next entity, which is assumed to already be selected.
    fn swap_selected(&mut self, c: &mut Commands, next: Entity)
    {
        if let Some(prev) = self.selected {
            c.react().entity_event(prev, Deselect);
        }
        self.selected = Some(next);
    }

    /// Clears the requested entity if stored here.
    fn try_clear(&mut self, entity: Entity) -> bool
    {
        let Some(prev) = &self.selected else { return false };
        if *prev != entity {
            return false;
        }
        self.selected = None;
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Used for cleanup of radio button handlers when the `RadioButton` instruction is revoked.
#[derive(Component)]
struct RadioButtonHandlers
{
    press_token: RevokeToken,
    select_token: RevokeToken,
}

impl RadioButtonHandlers
{
    fn revoke(self, rc: &mut ReactCommands)
    {
        rc.revoke(self.press_token);
        rc.revoke(self.select_token);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable that sets up a radio button group.
///
/// Inserts an internal `RadioButtonManager` component to the entity.
///
/// Individual buttons should use [`RadioButton`].
#[derive(Reflect, Default, PartialEq, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct RadioGroup;

impl Instruction for RadioGroup
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.insert_if_new(RadioButtonManager::default());

        // Note: we could try to 'steal' a selected entity from the nearest manager in case it needs to move
        // between groups. We currently don't do that for simplicity.
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.remove::<RadioButtonManager>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable that sets up a radio button on an entity.
///
/// Adds an `on_pressed` handler for selecting the button. Adds an `on_select` handler for updating the nearest
/// `RadioButtonManager`.
///
/// See [`RadioGroup`].
#[derive(Reflect, Default, PartialEq, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct RadioButton;

impl Instruction for RadioButton
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(e) = world.get_entity(entity) else { return };

        // Check if there are already radio button handlers on this entity.
        if e.contains::<RadioButtonHandlers>() {
            return;
        }

        // Add handlers.
        let press_token = world.react(|rc| {
            rc.on_revokable(
                entity_event::<Pressed>(entity),
                move |mut c: Commands, states: PseudoStateParam| {
                    states.try_select(entity, &mut c);
                },
            )
        });

        let select_token = world.react(|rc| rc.on_revokable(
            entity_event::<Select>(entity),
            move |mut c: Commands, mut managers: Query<&mut RadioButtonManager>, parents: Query<&Parent>| {
                // Search for nearest manager parent to update the selected button.
                // - We assume this is fairly cheap and low frequency, allowing us to avoid caching the RadioButtonManager
                // entity, which would make things more complicated.
                let mut search_entity = entity;
                loop {
                    if let Ok(mut manager) = managers.get_mut(search_entity) {
                        manager.swap_selected(&mut c, entity);
                        break;
                    }
                    let Ok(parent) = parents.get(search_entity) else {
                        tracing::warn!("failed selecting radio button {entity:?}; no RadioButtonManager found in ancestors");
                        break;
                    };
                    search_entity = **parent;
                }
            }
        ));

        world
            .entity_mut(entity)
            .insert(RadioButtonHandlers { press_token, select_token });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Remove entity from nearest RadioButtonManager.
        let mut search_entity = entity;
        loop {
            if let Some(mut manager) = world.get_mut::<RadioButtonManager>(search_entity) {
                if manager.try_clear(entity) {
                    world.react(|rc| rc.entity_event(entity, Deselect));
                }
                break;
            }
            let Some(parent) = world.get::<Parent>(search_entity) else { break };
            search_entity = **parent;
        }

        // Cleanup.
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        let Some(handlers) = emut.take::<RadioButtonHandlers>() else { return };
        world.react(|rc| handlers.revoke(rc));
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebRadioButtonPlugin;

impl Plugin for CobwebRadioButtonPlugin
{
    fn build(&self, app: &mut App)
    {
        // TODO: re-enable once COB scene macros are implemented
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/widgets/radio_button",
        // "radio_button.cob");
        app.register_instruction_type::<RadioGroup>()
            .register_instruction_type::<RadioButton>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

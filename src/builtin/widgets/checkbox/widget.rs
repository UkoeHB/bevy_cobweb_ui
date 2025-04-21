use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//use crate::load_embedded_scene_file;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct CheckboxCallbacks
{
    on_press: RevokeToken,
}

impl CheckboxCallbacks
{
    fn revoke(self, rc: &mut ReactCommands)
    {
        rc.revoke(self.on_press);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable that sets up a checkbox.
///
/// Inserts self as a component and applies the [`Interactive`] instruction.
///
/// Pressing the entity will cause a [`ToggleCheck`] entity event to be sent.
#[derive(Reflect, Component, Default, PartialEq, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Checkbox;

impl Instruction for Checkbox
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.insert(self);

        if !emut.contains::<CheckboxCallbacks>() {
            let mut on_press = None;
            emut.world_scope(|world| {
                let token = world.react(|rc| {
                    rc.on_revokable(entity_event::<PointerPressed>(entity), move |mut c: Commands| {
                        c.react().entity_event(entity, ToggleCheck);
                    })
                });
                on_press = Some(token);
            });
            emut.insert(CheckboxCallbacks { on_press: on_press.unwrap() });
        }

        // Make the checkbox interactive.
        Interactive.apply(entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.remove::<Self>();
        if let Some(callbacks) = emut.take::<CheckboxCallbacks>() {
            world.react(move |rc| callbacks.revoke(rc));
        }
        Interactive::revert(entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebCheckboxPlugin;

impl Plugin for CobwebCheckboxPlugin
{
    fn build(&self, app: &mut App)
    {
        // TODO: re-enable once COB scene macros are implemented
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/widgets/checkbox",
        // "checkbox.cob");
        app.register_instruction_type::<Checkbox>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

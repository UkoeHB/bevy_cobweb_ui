use bevy::prelude::*;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for setting up the root entity of a multi-entity widget.
///
/// Applies a [`ControlMember`] instruction and inserts an internal `ControlMap` component to the entity.
///
/// Children of the root node can be accessed through their [`ControlMembers`](ControlMember) using
/// [`ControlBuilderExt::edit_child`].
///
/// It is recommended to apply this instruction before `Static`/`Responsive`/`Animated` instructions for optimal
/// performance.
#[derive(Reflect, Default, Clone, Debug, Eq, PartialEq)]
pub struct ControlRoot;

impl Instruction for ControlRoot
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Add control map if missing.
        if let Some(mut control_map) = emut.get_mut::<ControlMap>() {
            // Repair if map is currently anonymous.
            if control_map.is_anonymous() {
                // Make map non-anonymous.
                control_map.set_anonymous(false);

                // Repair
                control_map.remove_all_labels().count();
            }

            // We are not actually dying, just refreshing the control root, so this can be removed.
            emut.remove::<ControlMapDying>();
        } else {
            emut.insert(ControlMap::default());

            // Cold path when applying a root to an existing scene.
            #[cfg(feature = "hot_reload")]
            if emut.contains::<Children>() {
                // Look for the nearest ancestor with non-anonymous ControlMap and
                // force it to re-apply its labels, in case this new root needs to steal some.
                // - Note: we don't check for ControlMember here since any pre-existing ControlMember was likely
                //   removed when the entity was switched to ControlRoot. When a ControlMember is removed, the
                //   entity will be removed from the associated ControlMap, which should ensure no stale attributes
                //   related to this entity will linger in other maps.
                if let Some((_, control_map)) =
                    get_ancestor_mut_filtered::<ControlMap>(world, entity, |cm| !cm.is_anonymous())
                {
                    let labels: Vec<_> = control_map.remove_all_labels().collect();

                    // Labels
                    // - Re-applying these forces the entities to re-register in the correct control maps.
                    for (label, label_entity) in labels {
                        ControlMember { id: label }.apply(label_entity, world);
                    }
                }

                // Iterate children (stopping at control maps) to identify children with ControlMember. Refresh
                // those nodes in case they have attributes that are 'dangling'.
                let mut dangling = vec![];
                iter_descendants_filtered(
                    world,
                    entity,
                    |world, entity| world.get::<ControlMap>(entity).is_none(),
                    |world, entity| {
                        if world.get::<ControlMember>(entity).is_some() {
                            dangling.push(entity);
                        }
                    },
                );

                let mut scene_buffer = world.resource_mut::<SceneBuffer>();
                for controlled_entity in dangling {
                    scene_buffer.request_reload(controlled_entity);
                }
            }
        }

        // Update control label.
        ControlMember::from(SmolStr::default()).apply(entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Set map to dying. If the control root is re-applied then the dying state will be cleared.
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        if emut.contains::<ControlMap>() {
            emut.insert(ControlMapDying);
        }

        // Clean up the label.
        ControlMember::revert(entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable that adds self as a component to indicate the entity is part of a multi-entity control
/// group.
///
/// Use this if you want values on the entity to respond to interactions on other parts of the group.
///
/// Values in a multi-entity widget can be controlled with the [`Static`], [`Responsive`], and [`Animated`]
/// loadables.
///
/// It is recommended to apply this instruction before `Static`/`Responsive`/`Animated` instructions for optimal
/// performance.
#[derive(Component, Reflect, Default, Clone, Debug, Deref, DerefMut, Eq, PartialEq)]
pub struct ControlMember
{
    /// Optional ID for the control group member. Use this if you want to use [`Responsive::respond_to`] or
    /// [`Animated::respond_to`].
    #[reflect(default)]
    pub id: SmolStr,
}

impl ControlMember
{
    /// Makes a label from a static string.
    pub fn new(label: &'static str) -> Self
    {
        Self { id: SmolStr::new_static(label) }
    }
}

impl<T: Into<SmolStr>> From<T> for ControlMember
{
    fn from(string: T) -> Self
    {
        Self { id: string.into() }
    }
}

impl Instruction for ControlMember
{
    fn apply(mut self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Anonymous label
        if self.id.len() == 0 {
            // TODO: smol_str v0.3 lets you do this without intermediate allocation
            self.id = SmolStr::from(format!("_anon-{}v{}", entity.index(), entity.generation()));
        }

        // Insert or update control label.
        let label_str = self.id.clone();
        if let Some(mut existing_member) = emut.get_mut::<ControlMember>() {
            if existing_member.id != self.id {
                tracing::warn!("updating control label on {entity:?} from {existing_member:?} to {self:?}");
                existing_member.id = self.id;
            }
        } else {
            emut.insert(self);
        }

        // Add entry to nearest control map.
        if let Some(mut control_map) = emut.get_mut::<ControlMap>() {
            if control_map.is_anonymous() {
                // If the map is anonymous then this entity is *not* a control root and we need to remove the map.
                emut.remove::<ControlMap>();
            } else {
                control_map.insert(label_str, entity);
                return;
            }
        }

        if let Some((_, control_map)) =
            get_ancestor_mut_filtered::<ControlMap>(world, entity, |cm| !cm.is_anonymous())
        {
            control_map.insert(label_str, entity);
            return;
        }

        tracing::error!(
            "error while inserting ControlMember({label_str}) to {entity:?}, no ancestor with ControlMap \
            (see ControlRoot)",
        );
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Clean up dynamic style.
        // TODO: in dynamic style system, remove DynamicStyleStopwatch if DynamicStyle is empty or non-existent
        emut.remove::<(DynamicStyle, DynamicStyleStopwatch)>();

        // Remove entry from nearest control map.
        // - All still-existing attributes will be re-inserted when the entity reloads.
        if let Some(mut control_map) = emut.get_mut::<ControlMap>() {
            if control_map.is_anonymous() {
                emut.remove::<ControlMap>();
            } else {
                control_map.remove(entity);
            }
        } else {
            if let Some((_, control_map)) =
                get_ancestor_mut_filtered::<ControlMap>(world, entity, |cm| !cm.is_anonymous())
            {
                control_map.remove(entity);
            }
        }

        // Remove label component.
        let mut emut = world.entity_mut(entity);
        emut.remove::<ControlMember>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<ControlRoot>()
            .register_instruction_type::<ControlMember>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

use bevy::prelude::*;
#[cfg(feature = "hot_reload")]
use bevy_cobweb::prelude::*;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "hot_reload")]
fn collect_dangling_controlled(child: Entity, world: &World, dangling: &mut Vec<Entity>)
{
    // Terminate at control maps.
    if world.get::<ControlMap>(child).is_some() {
        return;
    }

    // Collect dangling label.
    if world.get::<ControlLabel>(child).is_some() {
        dangling.push(child);
    }

    // Iterate into children.
    if let Some(children) = world.get::<Children>(child) {
        for child in children.iter() {
            collect_dangling_controlled(*child, world, dangling);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for setting up the root entity of a multi-entity widget.
///
/// Applies a [`ControlLabel`] instruction and inserts an internal `ControlMap` component to the entity.
///
/// Children of the root node can be accessed through their [`ControlLabels`](ControlLabel) using
/// [`ControlBuilderExt::edit_child`].
///
/// It is recommended to apply this instruction before `Static`/`Responsive`/`Animated` instructions for optimal
/// performance.
#[derive(Reflect, Default, Clone, Debug, Deref, DerefMut, Eq, PartialEq)]
pub struct ControlRoot(pub SmolStr);

impl ControlRoot
{
    /// Makes a control root from a static string.
    pub fn new(label: &'static str) -> Self
    {
        Self(SmolStr::new_static(label))
    }
}

impl<T: Into<SmolStr>> From<T> for ControlRoot
{
    fn from(string: T) -> Self
    {
        Self(string.into())
    }
}

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
                let attrs = control_map.remove_all_attrs();

                // Reset invalid targets now that we have a proper label.
                let new_target = Some(self.0.clone());
                for (origin, source, _target, state, attribute) in attrs {
                    control_map.set_attribute(origin, state, source, new_target.clone(), attribute);
                }
            }

            // We are not actually dying, just refreshing the control root, so this can be removed.
            emut.remove::<ControlMapDying>();
        } else {
            emut.insert(ControlMap::default());

            // Cold path when applying a root to an existing scene.
            #[cfg(feature = "hot_reload")]
            if emut.contains::<Children>() {
                // Look for the nearest ancestor with non-anonymous ControlMap and
                // force it to re-apply its attributes and labels, in case this new root needs to steal some.
                // - Note: we don't check for ControlLabel here since any pre-existing ControlLabel was likely
                //   removed when the entity was switched to ControlRoot. When a ControlLabel is removed, the
                //   entity will be removed from the associated ControlMap, which should ensure no stale attributes
                //   related to this entity will linger in other maps.
                let mut current = entity;
                while let Some(parent) = world.get::<Parent>(current) {
                    current = parent.get();
                    let Some(mut control_map) = world.get_mut::<ControlMap>(current) else { continue };
                    if control_map.is_anonymous() {
                        continue;
                    }
                    let labels: Vec<_> = control_map.remove_all_labels().collect();
                    let attrs = control_map.remove_all_attrs();

                    // Labels
                    // - Re-applying these forces the entities to re-register in the correct control maps.
                    for (label, label_entity) in labels {
                        ControlLabel(label).apply(label_entity, world);
                    }

                    // Attrs
                    // Note: target not needed, it is always set to self.
                    for (origin, source, _target, state, attribute) in attrs {
                        world.syscall((origin, source, state, attribute, "unknown"), super::add_attribute);
                    }
                    break;
                }

                // Iterate children (stopping at control maps) to identify children with ControlLabel. Refresh
                // those nodes in case they have attributes that are 'dangling'.
                let mut dangling = vec![];
                for child in world.get::<Children>(entity).unwrap().iter() {
                    collect_dangling_controlled(*child, world, &mut dangling);
                }

                let mut scene_buffer = world.resource_mut::<SceneBuffer>();
                for controlled_entity in dangling {
                    scene_buffer.request_reload(controlled_entity);
                }
            }
        }

        // Update control label.
        ControlLabel(self.0).apply(entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Set map to dying. If the control root is re-applied then the dying state will be cleared.
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        if emut.contains::<ControlMap>() {
            emut.insert(ControlMapDying);
        }

        // Clean up the label.
        ControlLabel::revert(entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable that adds self as a component to indicate the entity is part of a widget.
///
/// Use this if you want values on the entity to respond to interactions on other parts of the widget.
///
/// Values in a multi-entity widget can be controlled with the [`Static`], [`Responsive`], and [`Animated`]
/// loadables.
///
/// It is recommended to apply this instruction before `Static`/`Responsive`/`Animated` instructions for optimal
/// performance.
#[derive(Component, Reflect, Default, Clone, Debug, Deref, DerefMut, Eq, PartialEq)]
pub struct ControlLabel(pub SmolStr);

impl ControlLabel
{
    /// Makes a label from a static string.
    pub fn new(label: &'static str) -> Self
    {
        Self(SmolStr::new_static(label))
    }
}

impl<T: Into<SmolStr>> From<T> for ControlLabel
{
    fn from(string: T) -> Self
    {
        Self(string.into())
    }
}

impl Instruction for ControlLabel
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Insert or update control label.
        let label_str = self.0.clone();
        if let Some(mut existing_label) = emut.get_mut::<ControlLabel>() {
            if *existing_label != self {
                tracing::warn!("updating control label on {entity:?} from {existing_label:?} to {self:?}");
                *existing_label = self;
            }
        } else {
            emut.insert(self);
        }

        // Add entry to nearest control map.
        if let Some(mut control_map) = emut.get_mut::<ControlMap>() {
            if control_map.is_anonymous() {
                // If the map is anonymous then this entity is *not* a control root and we need to drain the
                // map's attributes and re-apply them now that we have a label on this entity.
                let attrs = control_map.remove_all_attrs();
                emut.remove::<ControlMap>();

                // Attrs
                // Note: target not needed, it is always set to self.
                for (origin, source, _target, state, attribute) in attrs {
                    world.syscall((origin, source, state, attribute, "unknown"), super::add_attribute);
                }
            } else {
                control_map.insert(label_str, entity);
                return;
            }
        }

        let mut current = entity;
        while let Some(parent) = world.get::<Parent>(current) {
            current = parent.get();
            let Some(mut control_map) = world.get_mut::<ControlMap>(current) else { continue };
            if control_map.is_anonymous() {
                continue;
            }
            control_map.insert(label_str, entity);
            return;
        }

        tracing::error!(
            "error while inserting ControlLabel({}) to {entity:?}, no ancestor with ControlMap \
            (see ControlRoot)",
            label_str
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
            let mut current = entity;
            while let Some(parent) = world.get::<Parent>(current) {
                current = parent.get();
                let Some(mut control_map) = world.get_mut::<ControlMap>(current) else { continue };
                control_map.remove(entity);
                break;
            }
        }

        // Remove label component.
        let mut emut = world.entity_mut(entity);
        emut.remove::<ControlLabel>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<ControlRoot>()
            .register_instruction_type::<ControlLabel>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

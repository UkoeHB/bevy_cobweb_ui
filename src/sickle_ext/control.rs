use bevy::prelude::*;
#[cfg(feature = "hot_reload")]
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::prelude::DynamicStyle;
use crate::sickle::theme::dynamic_style::DynamicStyleStopwatch;

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

/// Loadable for setting up the root entity of a multi-entity widget.
///
/// Inserts a [`ControlLabel`] and a `ControlMap` (internal component) to the entity.
///
/// Children of the root node can be accessed through their [`ControlLabels`](ControlLabel) using
/// [`ControlBuilderExt::edit_child`].
#[derive(Reflect, Default, Clone, Debug, Deref, DerefMut, Eq, PartialEq, Serialize, Deserialize)]
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
        let Some(mut emut) = world.get_entity_mut(entity) else { return };

        // Add control map if missing.
        if !emut.contains::<ControlMap>() {
            emut.insert(ControlMap::default());

            // Cold path when applying a root to an existing scene.
            #[cfg(feature = "hot_reload")]
            if emut.contains::<Children>() {
                // Look for the nearest ancestor with ControlMap and
                // force it to re-apply its attributes and labels, in case this new root needs to steal some.
                // - Note: we don't check for ControlLabel here since any pre-existing ControlLabel was likely
                //   removed
                // when it was switched to ControlRoot. When a ControlLabel is removed, the entity will be removed
                // from the associated ControlMap, which should ensure no stale attributes related
                // to this entity will linger in other maps.
                let mut current = entity;
                while let Some(parent) = world.get::<Parent>(current) {
                    current = parent.get();
                    let Some(mut control_map) = world.get_mut::<ControlMap>(current) else { continue };
                    let labels: Vec<_> = control_map.remove_all_labels().collect();
                    let attrs = control_map.remove_all_attrs();

                    // Labels
                    for (label, label_entity) in labels {
                        ControlLabel(label).apply(label_entity, world);
                    }

                    // Attrs
                    for (origin, source, target, state, attribute) in attrs {
                        world.syscall((origin, source, target, state, attribute), super::add_attribute);
                    }
                    break;
                }

                // Iterate children (stopping at control maps) to identify children with ControlLabel. Refresh
                // those nodes in case they have attributes that are 'dangling'.
                let mut dangling = vec![];
                for child in world.get::<Children>(entity).unwrap().iter() {
                    collect_dangling_controlled(*child, world, &mut dangling);
                }

                let mut caf_cache = world.resource_mut::<CobwebAssetCache>();
                for controlled_entity in dangling {
                    caf_cache.request_reload(controlled_entity);
                }
            }
        } else {
            // We are not actually dying, just refreshing the control root, so this can be removed.
            emut.remove::<ControlMapDying>();
        }

        // Update control label.
        ControlLabel(self.0).apply(entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        // Set map to dying. If the control root is re-applied then the dying state will be cleared.
        let Some(mut emut) = world.get_entity_mut(entity) else { return };
        if emut.contains::<ControlMap>() {
            emut.insert(ControlMapDying);
        }

        // Clean up the label.
        ControlLabel::revert(entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that indicates an entity is part of a widget.
///
/// Use this if you want values on the entity to respond to interactions on other parts of the widget, or if
/// you want different values to be applied depending on the widget's
/// [`PseudoStates`](crate::sickle::prelude::PseudoState).
///
/// Values in a multi-entity widget can be controlled with the [`Themed`], [`Responsive`], and [`Animated`]
/// loadables.
#[derive(Component, Reflect, Default, Clone, Debug, Deref, DerefMut, Eq, PartialEq, Serialize, Deserialize)]
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
        let Some(mut emut) = world.get_entity_mut(entity) else { return };

        // Add entry to nearest control map.
        if let Some(mut control_map) = emut.get_mut::<ControlMap>() {
            control_map.insert(self.0.as_str(), entity);
        } else {
            let mut current = entity;
            let mut found = false;
            while let Some(parent) = world.get::<Parent>(current) {
                current = parent.get();
                let Some(mut control_map) = world.get_mut::<ControlMap>(current) else { continue };
                control_map.insert(self.0.as_str(), entity);
                found = true;
                break;
            }

            if !found {
                tracing::error!(
                    "error while inserting ControlLabel({}) to {entity:?}, no ancestor with ControlMap \
                    (see ControlRoot)",
                    self.0
                );
            }
        }

        // Insert or update control label.
        let mut emut = world.entity_mut(entity);
        if let Some(mut existing_label) = emut.get_mut::<ControlLabel>() {
            if *existing_label != self {
                tracing::warn!("updating control label on {entity:?} from {existing_label:?} to {self:?}");
                *existing_label = self;
            }
        } else {
            emut.insert(self);
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Some(mut emut) = world.get_entity_mut(entity) else { return };

        // Clean up dynamic style.
        // TODO: in dynamic style system, remove DynamicStyleStopwatch if DynamicStyle is empty or non-existent
        emut.remove::<(DynamicStyle, DynamicStyleStopwatch)>();

        // Remove entry from nearest control map.
        // - All still-existing attributes will be re-inserted when the entity reloads.
        if let Some(mut control_map) = emut.get_mut::<ControlMap>() {
            control_map.remove(entity);
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

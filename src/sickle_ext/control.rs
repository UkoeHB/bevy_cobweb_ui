use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//use smol_str::SmolStr;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Loadable for setting up the root entity of a multi-entity widget.
///
/// Inserts a [`ControlLabel`] and a `ControlMap` (internal component) to the entity.
///
/// Children of the root node can be accessed through their [`ControlLabels`](ControlLabel) using
/// [`ControlBuilderExt::edit_child`].
//todo: use SmolStr in bevy v0.15; see https://github.com/bevyengine/bevy/issues/14969
#[derive(Reflect, Default, Clone, Debug, Deref, DerefMut, Eq, PartialEq, Serialize, Deserialize)]
pub struct ControlRoot(pub String);

impl ControlRoot
{
    /// Makes a control root from a static string.
    pub fn new(label: &'static str) -> Self
    {
        //Self(SmolStr::new_static(label))
        Self(String::from(label))
    }
}

impl<T: Into<String>> From<T> for ControlRoot
{
    fn from(string: T) -> Self
    {
        Self(string.into())
    }
}

impl ApplyLoadable for ControlRoot
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut ec) = world.get_entity_mut(entity) else { return };

        // Add control map if missing.
        if !ec.contains::<ControlMap>() {
            ec.insert(ControlMap::default());
        }

        // Update control label.
        ControlLabel(self.0).apply(entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that indicates an entity is part of a widget.
///
/// Use this if you want values on the entity to respond to interactions on other parts of the widget, or if
/// you want different values to be applied depending on the widget's [`PseudoStates`](PseudoState).
///
/// Values in a multi-entity widget can be controlled with the [`Themed`], [`Responsive`], and [`Animated`]
/// loadables.
//todo: use SmolStr in bevy v0.15; see https://github.com/bevyengine/bevy/issues/14969
#[derive(Component, Reflect, Default, Clone, Debug, Deref, DerefMut, Eq, PartialEq, Serialize, Deserialize)]
pub struct ControlLabel(pub String);

impl ControlLabel
{
    /// Makes a label from a static string.
    pub fn new(label: &'static str) -> Self
    {
        //Self(SmolStr::new_static(label))
        Self(String::from(label))
    }
}

impl<T: Into<String>> From<T> for ControlLabel
{
    fn from(string: T) -> Self
    {
        Self(string.into())
    }
}

impl ApplyLoadable for ControlLabel
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut ec) = world.get_entity_mut(entity) else { return };

        // Add entry to nearest control map.
        if let Some(mut control_map) = ec.get_mut::<ControlMap>() {
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
                tracing::error!("failed inserting ControlLabel({}) to {entity:?}, no ancestor with ControlMap \
                    (see ControlRoot)", self.0);
                return;
            }
        }

        // Insert or update control label.
        let mut ec = world.entity_mut(entity);
        if let Some(mut existing_label) = ec.get_mut::<ControlLabel>() {
            if *existing_label != self {
                tracing::warn!("updating control label on {entity:?} from {existing_label:?} to {self:?}");
                *existing_label = self;
            }
        } else {
            ec.insert(self);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_derived::<ControlRoot>()
            .register_derived::<ControlLabel>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

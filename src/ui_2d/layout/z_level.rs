use crate::*;

use bevy::prelude::*;
use bevy::ecs::entity::Entities;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

/// Component that controls the z-order of nodes relative to their siblings on the same parent.
///
/// Sibling nodes are sorted by `ZLevel` so higher levels are positioned above lower levels.
/// Within a level, sibling nodes are ordered based on their index in the parent's [`Children`] list so that newer
/// nodes default to sorting above older nodes.
///
/// If one node is sorted above another, then the higher node's children will be sorted above all children of the lower,
/// regardless of `ZLevel`.
//todo: consider adding options for how nodes at the same z-level will be sorted (e.g. child-order, (+/-)(x/y)-order, etc.)
//todo: consider adding option to insert artificial gap into the hierarchy for 3D effects
#[derive(Component, Reflect, Debug, Default, Copy, Clone, Deref, DerefMut, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ZLevel(pub i32);

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ZLevelPlugin;

impl Plugin for ZLevelPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<ZLevel>()
            .register_loadable::<ZLevel>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

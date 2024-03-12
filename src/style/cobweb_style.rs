//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Trait representing [`UiInstructions`](UiInstruction) that can be loaded with [`StyleSheet`].
///
/// Styles must be inserted as [`ReactComponents`](bevy_cobweb::prelude::ReactComponent) to node entities, and their
/// [`UiInstruction`] implementation should include reaction logic for handling mutations to the style component
/// caused by stylesheet loading.
///
/// Note that it is typically safe to manually mutate `CobwebStyle` components on node entities, because stylesheet
/// loading is only used for initialization in production settings.
/// If you *do* reload a stylesheet (e.g. during development), then existing dynamic styles that were changed will be
/// overwritten.
pub trait CobwebStyle: ReactComponent + Reflect + FromReflect + PartialEq + Clone + Default
    + Serialize + for<'de> Deserialize<'de>
{
    /// Applies a style to a node.
    ///
    /// Implementing this enables styles to be used as UI instructions. The [`UiInstruction`] implmentation for styles
    /// invokes this method. The UI instruction then inserts `Self` as a `ReactComponent` on the node. You should not
    /// insert it manually.
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity);
}

impl<T: CobwebStyle> UiInstruction for T
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        Self::apply_style(&self, rc, node);
        rc.insert(node, self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

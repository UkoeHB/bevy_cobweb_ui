use crate::*;

use bevy::prelude::*;
use bevy::transform::TransformSystem::TransformPropagate;

//-------------------------------------------------------------------------------------------------------------------

/// [`SystemSet`] containing layout systems that run in [`PostUpdate`] immediately before [`TransformPropagate`], which
/// updates [`GlobalTransforms`](GlobalTransform).
///
/// UI-updating systems should run before this system set to ensure layout handling systems
/// see changes before they update transforms.
#[derive(SystemSet, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub struct LayoutSet;

/// [`SystemSet`] containing systems that prepare the UI tree for layout computation.
///
/// Use this if you need systems that mark nodes dirty based on certain conditions (e.g. by default this set includes
/// a system to check for [`Children`] and [`Parent`] changes on UI nodes).
///
/// Runs in [`LayoutSet`].
#[derive(SystemSet, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub struct LayoutSetPrep;

/// [`SystemSet`] containing layout update systems.
///
/// Runs in [`LayoutSet`].
#[derive(SystemSet, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub(crate) struct LayoutSetCompute;

/// [`SystemSet`] containing z-order update systems.
///
/// Runs in [`LayoutSet`].
#[derive(SystemSet, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub(crate) struct LayoutSetSort;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LayoutPlugin;

impl Plugin for LayoutPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .add_plugins(Val2dPlugin)
            .add_plugins(Dims2dPlugin)
            .add_plugins(Position2dPlugin)
            .add_plugins(SortingPlugin)
            .add_plugins(ZLevelPlugin)
            .add_plugins(SizeRefSourcePlugin)
            .add_plugins(TrackDirtyPlugin)
            .add_plugins(LayoutAlgorithmPlugin)
            .configure_sets(PostUpdate, LayoutSet.before(TransformPropagate))
            .configure_sets(PostUpdate,
                (
                    LayoutSetPrep,
                    LayoutSetCompute,
                    LayoutSetSort,
                )
                    .chain()
                    .in_set(LayoutSet)
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

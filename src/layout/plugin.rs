//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::transform::TransformSystem::TransformPropagate;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// [`SystemSet`] containing layout systems that run immediately before [`TransformPropagate`], which updates
/// [`GlobalTransforms`](GlobalTransform).
///
/// UI-updating systems should run before this system set to ensure layout handling systems
/// see changes before they update transforms.
#[derive(SystemSet, Debug, Copy, Clone, Hash, Eq, PartialEq, Default)]
pub struct LayoutSet;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LayoutPlugin;

impl Plugin for LayoutPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(DimsPlugin)
            .add_plugins(PositionPlugin)
            .add_plugins(SortingPlugin)
            .configure_sets(PostUpdate, LayoutSet.before(TransformPropagate));
    }
}

//-------------------------------------------------------------------------------------------------------------------

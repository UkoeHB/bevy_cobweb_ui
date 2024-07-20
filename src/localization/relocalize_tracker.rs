use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn set_tracker(mut tracker: ResMut<RelocalizeTracker>)
{
    tracker.waiting = true;
}

//-------------------------------------------------------------------------------------------------------------------

fn try_trigger_tracker(mut c: Commands, mut tracker: ResMut<RelocalizeTracker>, progress: Res<LoadProgress>)
{
    if !tracker.waiting {
        return;
    }

    // NOTE: We assume this won't be false after `LanguagesNegotiated` until all assets
    // that start loading due to `LanguagesNegotiated` have finished loading.
    if progress.is_loading() {
        return;
    }

    tracker.waiting = false;
    c.react().broadcast(RelocalizeApp);
}

//-------------------------------------------------------------------------------------------------------------------

/// Tracker too coordinate sending `RelocalizeApp` events after `LanguagesNegotiated` is detected.
///
/// The tracker waits until the app is done loading any assets, then emits `RelocalizeApp`.
///
/// NOTE: We assume
#[derive(Resource, Default)]
struct RelocalizeTracker
{
    /// Indicates if the tracker is waiting for loading to complete after languages were renegotiated.
    waiting: bool,
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when the app is ready to relocalize all text, fonts, and other assets.
///
/// This is used to synchronize relocalizing miscellaneous assets that are loaded and tracked separately. Without
/// synchronization, users may experience a lot of jank as assets for new languages are loaded asynchronously.
pub struct RelocalizeApp;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct RelocalizeTrackerPlugin;

impl Plugin for RelocalizeTrackerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<RelocalizeTracker>()
            .react(|rc| rc.on_persistent(broadcast::<LanguagesNegotiated>(), set_tracker))
            // Note: when transitioning LoadState::Loading -> LoadState::Done, the trigger will fire
            // *before* the state transition is applied even though at this point it will already be scheduled.
            .add_systems(PreUpdate, try_trigger_tracker.after(LoadProgressSet::Check));
    }
}

//-------------------------------------------------------------------------------------------------------------------

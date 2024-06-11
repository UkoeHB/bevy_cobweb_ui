use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn check_load_progress(
    mut c: Commands,
    loadablesheet: ReactRes<LoadableSheet>,
    mut next: ResMut<NextState<LoadProgress>>,
)
{
    let (pending, total) = loadablesheet.loading_progress();
    //todo: incorporate secondary asset tracking

    if pending > 0 {
        return;
    }

    tracing::info!("Startup loading done: {total} files");
    c.react().broadcast(StartupLoadingDone);
    next.set(LoadProgress::Done);
}

//-------------------------------------------------------------------------------------------------------------------

/// Bevy states for tracking the initial load of `bevy_cobweb_ui` assets.
///
/// It is recommended to show a loading screen in [`LoadProgress::Loading`], and to initialize UI in
/// schedule `OnEnter(LoadProgress::Done)`.
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum LoadProgress
{
    /// Loadable files and transitive assets are loading.
    #[default]
    Loading,
    /// Loading is done.
    Done,
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when state [`LoadProgress::Done`] is entered.
pub struct StartupLoadingDone;

//-------------------------------------------------------------------------------------------------------------------

/// System set in [`PreUpdate`] where load progress is checked.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct CheckLoadProgressSet;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadProgressPlugin;

impl Plugin for LoadProgressPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<LoadProgress>()
            .add_systems(PreUpdate, check_load_progress.run_if(in_state(LoadProgress::Loading)));
    }
}

//-------------------------------------------------------------------------------------------------------------------

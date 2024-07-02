use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn clear_asset_progress(mut tracker: ResMut<AssetLoadProgressTracker>)
{
    tracker.clear();
}

//-------------------------------------------------------------------------------------------------------------------

fn collect_asset_progress<T: AssetLoadProgress>(mut tracker: ResMut<AssetLoadProgressTracker>, res: Res<T>)
{
    tracker.insert(res.pending_assets(), res.total_assets());
}

//-------------------------------------------------------------------------------------------------------------------

fn check_load_progress(
    mut c: Commands,
    caf_cache: ReactRes<CobwebAssetCache>,
    asset_tracker: ResMut<AssetLoadProgressTracker>,
    mut next: ResMut<NextState<LoadState>>,
)
{
    let (pending_files, total_files) = caf_cache.loading_progress();
    let (pending_assets, total_assets) = asset_tracker.loading_progress();

    let pending = pending_files + pending_assets;
    if pending > 0 {
        return;
    }

    tracing::info!("Startup loading done: {total_files} file(s), {total_assets} asset(s)");
    c.react().broadcast(StartupLoadingDone);
    next.set(LoadState::Done);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct AssetLoadProgressTracker
{
    pending: usize,
    total: usize,
}

impl AssetLoadProgressTracker
{
    fn insert(&mut self, pending: usize, total: usize)
    {
        self.pending += pending;
        self.total += total;
    }

    fn clear(&mut self)
    {
        self.pending = 0;
        self.total = 0;
    }

    fn loading_progress(&self) -> (usize, usize)
    {
        (self.pending, self.total)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for resources that track asset loading in state [`LoadState::Loading`].
///
/// If a resource is registered with
/// [`App::register_asset_tracker`](AssetLoadProgressTrackerAppExt::register_asset_tracker)
/// then state [`LoadState::Done`] will be postponed until the resource returns 0 from [`Self::pending_assets`].
pub trait AssetLoadProgress: Resource
{
    /// Gets the number of assets currently loading.
    fn pending_assets(&self) -> usize;
    /// Gets the total number of assets loaded and loading.
    fn total_assets(&self) -> usize;
}

//-------------------------------------------------------------------------------------------------------------------

/// App extension trait for registering types that implement [`AssetLoadProgress`].
pub trait AssetLoadProgressTrackerAppExt
{
    /// Registers a resource that reports asset load progress in state [`LoadState::Loading`].
    ///
    /// It is recommended to load all assets in state [`LoadState::Loading`] so they are available
    /// post-initialization.
    fn register_asset_tracker<T: AssetLoadProgress>(&mut self) -> &mut Self;
}

impl AssetLoadProgressTrackerAppExt for App
{
    fn register_asset_tracker<T: AssetLoadProgress>(&mut self) -> &mut Self
    {
        self.add_systems(
            PreUpdate,
            collect_asset_progress::<T>.in_set(LoadProgressSet::AssetProgress),
        )
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Bevy states for tracking the initial load of `bevy_cobweb_ui` assets.
///
/// It is recommended to show a loading screen in [`LoadState::Loading`], and to initialize UI in
/// schedule `OnEnter(LoadState::Done)`.
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
pub enum LoadState
{
    /// Loadable files and transitive assets are loading.
    #[default]
    Loading,
    /// Loading is done.
    Done,
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when state [`LoadState::Done`] is entered.
pub struct StartupLoadingDone;

//-------------------------------------------------------------------------------------------------------------------

/// System set in [`PreUpdate`] where load progress is checked.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum LoadProgressSet
{
    /// Set where load progress checking is prepared.
    Prepare,
    /// Set where asset load progress is collected.
    AssetProgress,
    /// Set where total load progress is checked.
    Check,
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LoadProgressPlugin;

impl Plugin for LoadProgressPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<LoadState>()
            .init_resource::<AssetLoadProgressTracker>()
            .configure_sets(
                PreUpdate,
                (
                    LoadProgressSet::Prepare,
                    LoadProgressSet::AssetProgress,
                    LoadProgressSet::Check,
                )
                    .chain()
                    .run_if(in_state(LoadState::Loading)),
            )
            .add_systems(
                PreUpdate,
                (
                    clear_asset_progress.in_set(LoadProgressSet::Prepare),
                    check_load_progress.in_set(LoadProgressSet::Check),
                ),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

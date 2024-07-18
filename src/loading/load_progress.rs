use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn clear_asset_progress(mut progress: ResMut<LoadProgress>)
{
    progress.clear();
}

//-------------------------------------------------------------------------------------------------------------------

fn get_asset_progress<T: AssetLoadProgress + Resource>(world: &mut World) -> (usize, usize)
{
    let res = world.resource::<T>();
    (res.pending_assets(), res.total_assets())
}

//-------------------------------------------------------------------------------------------------------------------

fn get_asset_progress_reactive<T: AssetLoadProgress + ReactResource>(world: &mut World) -> (usize, usize)
{
    let res = world.react_resource::<T>();
    (res.pending_assets(), res.total_assets())
}

//-------------------------------------------------------------------------------------------------------------------

fn collect_asset_progress(world: &mut World)
{
    world.resource_scope(|world, mut progress: Mut<LoadProgress>| {
        let tracked = std::mem::take(&mut progress.tracked);
        for tracked_fn in tracked.iter() {
            let (pending, total) = (tracked_fn)(world);
            progress.insert(pending, total);
        }
        progress.tracked = tracked;
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn check_load_progress(progress: Res<LoadProgress>, mut next: ResMut<NextState<LoadState>>)
{
    let (pending, total) = progress.loading_progress();

    if pending > 0 {
        return;
    }

    tracing::info!("Loading done: {total} asset(s)");
    next.set(LoadState::Done);
}

//-------------------------------------------------------------------------------------------------------------------

/// Tracks the global loading progress of asset trackers.
///
/// Cleared in [`LoadProgressSet::Prepare`], updated in [`LoadProgressSet::Collect`], evaluated in
/// [`LoadProgressSet::Check`].
#[derive(Resource, Default)]
pub struct LoadProgress
{
    pending: usize,
    total: usize,

    tracked: Vec<fn(&mut World) -> (usize, usize)>,
}

impl LoadProgress
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

    /// Returns (num pending assets, num total assets).
    ///
    /// The total number of assets should be considered an approximation, since for efficiency asset managers may
    /// double-count some assets.
    pub fn loading_progress(&self) -> (usize, usize)
    {
        (self.pending, self.total)
    }

    /// Returns `true` if there are pending assets.
    pub fn is_loading(&self) -> bool
    {
        self.pending > 0
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for resources that track asset loading in state [`LoadState::Loading`].
///
/// If a resource is registered with
/// [`App::register_asset_tracker`](AssetLoadProgressAppExt::register_asset_tracker)
/// then state [`LoadState::Done`] will be postponed until the resource returns 0 from [`Self::pending_assets`].
pub trait AssetLoadProgress
{
    /// Gets the number of assets currently loading.
    fn pending_assets(&self) -> usize;
    /// Gets the total number of assets loaded and loading.
    fn total_assets(&self) -> usize;
}

//-------------------------------------------------------------------------------------------------------------------

/// App extension trait for registering types that implement [`AssetLoadProgress`].
pub trait AssetLoadProgressAppExt
{
    /// Registers a resource that reports asset load progress in [`LoadProgressSet::Collect`].
    fn register_asset_tracker<T: AssetLoadProgress + Resource>(&mut self) -> &mut Self;

    /// Registers a reactive resource that reports asset load progress in [`LoadProgressSet::Collect`].
    fn register_reactive_asset_tracker<T: AssetLoadProgress + ReactResource>(&mut self) -> &mut Self;
}

impl AssetLoadProgressAppExt for App
{
    fn register_asset_tracker<T: AssetLoadProgress + Resource>(&mut self) -> &mut Self
    {
        self.world_mut()
            .resource_mut::<LoadProgress>()
            .tracked
            .push(get_asset_progress::<T>);
        self
    }

    fn register_reactive_asset_tracker<T: AssetLoadProgress + ReactResource>(&mut self) -> &mut Self
    {
        self.world_mut()
            .resource_mut::<LoadProgress>()
            .tracked
            .push(get_asset_progress_reactive::<T>);
        self
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

/// System sets in [`PreUpdate`] where load progress is checked.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum LoadProgressSet
{
    /// Set where load progress checking is prepared.
    Prepare,
    /// Set where asset load progress is collected.
    Collect,
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
            .init_resource::<LoadProgress>()
            .configure_sets(
                PreUpdate,
                (
                    LoadProgressSet::Prepare,
                    LoadProgressSet::Collect,
                    //todo: need a more sophisticated loading state abstraction to capture 'initial loading' vs
                    // 'additional loads' (and then there is the Game Areas idea...)
                    LoadProgressSet::Check.run_if(in_state(LoadState::Loading)),
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                (
                    clear_asset_progress.in_set(LoadProgressSet::Prepare),
                    collect_asset_progress.in_set(LoadProgressSet::Collect),
                    check_load_progress.in_set(LoadProgressSet::Check),
                ),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

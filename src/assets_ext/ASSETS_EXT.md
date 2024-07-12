Contains various asset managers.

## Custom asset loaders

All custom asset loaders that feed data to asset managers in this module should implement [`AssetLoadProgress`](bevy_cobweb_ui::prelude::AssetLoadProgress) and register themselves in the app as tracked assets with [`register_asset_tracker`](bevy_cobweb_ui::prelude::AssetLoadProgressTrackerAppExt::register_asset_tracker). Otherwise [`LoadState`](bevy_cobweb_ui::prelude::LoadState) may move to `LoadState::Done` before your assets are loaded, which can cause synchronization issues (various systems run in `OnExt(LoadState::Loading)` and `OnEnter(LoadState::Done)`).

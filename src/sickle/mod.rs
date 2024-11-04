mod assets;
pub mod input_extension;

use assets::BuiltInAssetsPlugin;
use bevy::prelude::*;
use flux_interaction::FluxInteractionPlugin;
pub use sickle_macros::*;
pub use sickle_math::*;
pub use sickle_ui_scaffold::*;
use theme::ThemePlugin;

pub mod prelude {
    pub use sickle_macros::*;
    pub use sickle_math::*;
    pub use sickle_ui_scaffold::prelude::*;
}

/// Core plugin.
///
/// Must be added after [`DefaultPlugins`].
pub struct SickleUiPlugin;

impl Plugin for SickleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BuiltInAssetsPlugin,
            FluxInteractionPlugin,
            ThemePlugin,
        ));
    }
}

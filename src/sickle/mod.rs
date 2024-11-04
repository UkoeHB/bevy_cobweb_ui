mod assets;
#[cfg(feature = "dev_panels")]
pub mod dev_panels;
pub mod input_extension;
pub mod widgets;

use assets::BuiltInAssetsPlugin;
use bevy::prelude::*;
use drag_interaction::DragInteractionPlugin;
use drop_interaction::DropInteractionPlugin;
use flux_interaction::FluxInteractionPlugin;
use scroll_interaction::ScrollInteractionPlugin;
pub use sickle_macros::*;
pub use sickle_math::*;
pub use sickle_ui_scaffold::*;
use theme::ThemePlugin;
use widgets::WidgetsPlugin;

pub mod prelude
{
    pub use sickle_macros::*;
    pub use sickle_math::*;
    pub use sickle_ui_scaffold::prelude::*;

    pub use super::widgets::prelude::*;
}

/// Core plugin.
///
/// Must be added after [`DefaultPlugins`].
pub struct SickleUiPlugin;

impl Plugin for SickleUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins((
            BuiltInAssetsPlugin,
            DragInteractionPlugin,
            DropInteractionPlugin,
            FluxInteractionPlugin,
            ScrollInteractionPlugin,
            WidgetsPlugin,
            ThemePlugin,
        ));
    }
}

pub mod drag_interaction;
pub mod drop_interaction;
pub mod flux_interaction;
pub mod scroll_interaction;
pub mod theme;
pub mod ui_builder;
pub mod ui_commands;
pub mod ui_style;
pub mod ui_utils;

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

pub mod prelude
{
    pub use super::drag_interaction::{DragState, Draggable, DraggableUpdate};
    pub use super::drop_interaction::{DropPhase, DropZone, Droppable, DroppableUpdate};
    pub use super::flux_interaction::{
        FluxInteraction, FluxInteractionStopwatch, FluxInteractionStopwatchLock, FluxInteractionUpdate,
        TrackedInteraction,
    };
    pub use super::scroll_interaction::{ScrollAxis, Scrollable, ScrollableUpdate};
    pub use super::theme::prelude::*;
    pub use super::ui_builder::{UiBuilder, UiBuilderExt, UiContextRoot, UiRoot};
    pub use super::ui_commands::{ManagePseudoStateExt, UpdateStatesExt, UpdateTextExt};
    pub use super::ui_style::prelude::*;
    pub use super::ui_utils::UiUtils;
    pub use super::CardinalDirection;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Reflect, Serialize, Deserialize)]
pub enum CardinalDirection
{
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

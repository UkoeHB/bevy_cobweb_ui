pub mod inputs;
pub mod layout;
pub mod menus;

use bevy::prelude::*;

use self::inputs::checkbox::CheckboxPlugin;
use self::inputs::dropdown::DropdownPlugin;
use self::inputs::radio_group::RadioGroupPlugin;
use self::inputs::slider::SliderPlugin;
use self::layout::docking_zone::DockingZonePlugin;
use self::layout::floating_panel::{FloatingPanelPlugin, FloatingPanelUpdate};
use self::layout::foldable::FoldablePlugin;
use self::layout::resize_handles::ResizeHandlePlugin;
use self::layout::scroll_view::ScrollViewPlugin;
use self::layout::sized_zone::SizedZonePlugin;
use self::layout::tab_container::TabContainerPlugin;
use self::menus::context_menu::ContextMenuPlugin;
use self::menus::menu::MenuPlugin;
use self::menus::menu_bar::MenuBarPlugin;
use self::menus::menu_item::MenuItemPlugin;
use self::menus::menu_separators::MenuSeparatorPlugin;
use self::menus::shortcut::ShortcutPlugin;
use self::menus::submenu::SubmenuPlugin;
use self::menus::toggle_menu_item::ToggleMenuItemPlugin;

pub mod prelude
{
    // Used with scroll views, floating panels, etc. often
    pub use sickle_ui_scaffold::scroll_interaction::ScrollAxis;

    pub use super::inputs::checkbox::{Checkbox, UiCheckboxExt};
    pub use super::inputs::dropdown::{Dropdown, UiDropdownExt};
    pub use super::inputs::radio_group::{RadioGroup, UiRadioGroupExt};
    pub use super::inputs::slider::{Slider, SliderConfig, UiSliderExt};
    pub use super::layout::column::UiColumnExt;
    pub use super::layout::container::UiContainerExt;
    pub use super::layout::docking_zone::UiDockingZoneExt;
    pub use super::layout::floating_panel::{
        FloatingPanelConfig, FloatingPanelLayout, FloatingPanelUpdate, UiFloatingPanelExt,
    };
    pub use super::layout::foldable::{Foldable, FoldableUpdate, UiFoldableExt};
    pub use super::layout::icon::UiIconExt;
    pub use super::layout::label::{LabelConfig, UiLabelExt};
    pub use super::layout::panel::UiPanelExt;
    pub use super::layout::resize_handles::{ResizeHandle, ResizeHandles, UiResizeHandlesExt};
    pub use super::layout::row::UiRowExt;
    pub use super::layout::scroll_view::{ScrollViewLayoutUpdate, UiScrollViewExt};
    pub use super::layout::sized_zone::{SizedZoneConfig, SizedZonePreUpdate, UiSizedZoneExt};
    pub use super::layout::tab_container::{TabContainerUpdate, UiTabContainerExt, UiTabContainerSubExt};
    pub use super::menus::context_menu::{
        ContextMenuGenerator, ContextMenuUpdate, ReflectContextMenuGenerator, UiContextMenuExt,
    };
    pub use super::menus::extra_menu::{ExtraMenu, UiExtraMenuExt};
    pub use super::menus::menu::{MenuConfig, MenuUpdate, UiMenuExt, UiMenuSubExt};
    pub use super::menus::menu_bar::UiMenuBarExt;
    pub use super::menus::menu_item::{MenuItem, MenuItemConfig, MenuItemUpdate, UiMenuItemExt};
    pub use super::menus::menu_separators::{UiMenuItemSeparatorExt, UiMenuSeparatorExt};
    pub use super::menus::shortcut::{Shortcut, ShortcutPreUpdate};
    pub use super::menus::submenu::{SubmenuConfig, SubmenuUpdate, UiSubmenuExt, UiSubmenuSubExt};
    pub use super::menus::toggle_menu_item::{ToggleMenuItemConfig, ToggleMenuItemUpdate, UiToggleMenuItemExt};
    pub use super::WidgetLibraryUpdate;
}

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.configure_sets(Update, WidgetLibraryUpdate.after(FloatingPanelUpdate))
            .add_plugins((
                CheckboxPlugin,
                ContextMenuPlugin,
                SizedZonePlugin,
                DockingZonePlugin,
                DropdownPlugin,
                FloatingPanelPlugin,
                FoldablePlugin,
                MenuPlugin,
            ))
            .add_plugins((
                MenuBarPlugin,
                MenuItemPlugin,
                MenuSeparatorPlugin,
                RadioGroupPlugin,
                ResizeHandlePlugin,
                ShortcutPlugin,
                SliderPlugin,
                ScrollViewPlugin,
                SubmenuPlugin,
                TabContainerPlugin,
                ToggleMenuItemPlugin,
            ));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct WidgetLibraryUpdate;

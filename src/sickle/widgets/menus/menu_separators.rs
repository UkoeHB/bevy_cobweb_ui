use bevy::prelude::*;

use sickle_macros::UiContext;
use sickle_ui_scaffold::prelude::*;

use super::{
    context_menu::{ContextMenu, UiContextMenuExt},
    menu::{Menu, UiMenuSubExt},
    menu_bar::MenuBar,
    submenu::{Submenu, UiSubmenuSubExt},
};

pub struct MenuSeparatorPlugin;

impl Plugin for MenuSeparatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ComponentThemePlugin::<MenuSeparator>::default(),
            ComponentThemePlugin::<MenuItemSeparator>::default(),
        ));
    }
}

#[derive(Component, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MenuSeparator;

impl DefaultTheme for MenuSeparator {
    fn default_theme() -> Option<Theme<MenuSeparator>> {
        MenuSeparator::theme().into()
    }
}

impl MenuSeparator {
    pub fn theme() -> Theme<MenuSeparator> {
        let base_theme = PseudoTheme::deferred(None, MenuSeparator::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .align_self(AlignSelf::Center)
            .width(Val::Px(theme_spacing.gaps.tiny))
            .height(Val::Px(theme_spacing.areas.extra_small))
            .margin(UiRect::horizontal(Val::Px(theme_spacing.gaps.small)))
            .background_color(colors.accent(Accent::OutlineVariant));
    }

    fn separator() -> impl Bundle {
        (Name::new("Separator"), NodeBundle::default())
    }
}

pub trait UiMenuSeparatorExt {
    fn separator(&mut self) -> UiBuilder<Entity>;
}

impl UiMenuSeparatorExt for UiBuilder<'_, (Entity, MenuBar)> {
    fn separator(&mut self) -> UiBuilder<Entity> {
        let container_id = self.id();
        let id = self
            .commands()
            .ui_builder(container_id)
            .spawn((MenuSeparator::separator(), MenuSeparator))
            .id();

        self.commands().ui_builder(id)
    }
}

#[derive(Component, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MenuItemSeparator;

impl DefaultTheme for MenuItemSeparator {
    fn default_theme() -> Option<Theme<MenuItemSeparator>> {
        MenuItemSeparator::theme().into()
    }
}

impl MenuItemSeparator {
    pub fn theme() -> Theme<MenuItemSeparator> {
        let base_theme = PseudoTheme::deferred(None, MenuItemSeparator::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .min_width(Val::Px(theme_spacing.areas.extra_large))
            .height(Val::Px(theme_spacing.gaps.tiny))
            .margin(UiRect::vertical(Val::Px(theme_spacing.gaps.small)))
            .background_color(colors.accent(Accent::OutlineVariant));
    }

    fn separator() -> impl Bundle {
        (Name::new("Separator"), NodeBundle::default())
    }
}

pub trait UiMenuItemSeparatorExt {
    fn separator(&mut self) -> UiBuilder<Entity>;
}

impl UiMenuItemSeparatorExt for UiBuilder<'_, Menu> {
    fn separator(&mut self) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .spawn((MenuItemSeparator::separator(), MenuItemSeparator))
            .id();

        self.commands().ui_builder(id)
    }
}

impl UiMenuItemSeparatorExt for UiBuilder<'_, Submenu> {
    fn separator(&mut self) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .spawn((MenuItemSeparator::separator(), MenuItemSeparator))
            .id();

        self.commands().ui_builder(id)
    }
}

impl UiMenuItemSeparatorExt for UiBuilder<'_, ContextMenu> {
    fn separator(&mut self) -> UiBuilder<Entity> {
        let container_id = self.container();
        let id = self
            .commands()
            .ui_builder(container_id)
            .spawn((MenuItemSeparator::separator(), MenuItemSeparator))
            .id();

        self.commands().ui_builder(id)
    }
}

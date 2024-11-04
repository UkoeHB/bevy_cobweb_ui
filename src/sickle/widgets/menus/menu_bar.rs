use bevy::prelude::*;

use sickle_macros::UiContext;
use sickle_ui_scaffold::prelude::*;

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MenuBar>::default());
    }
}

#[derive(Component, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MenuBar;

impl DefaultTheme for MenuBar {
    fn default_theme() -> Option<Theme<MenuBar>> {
        MenuBar::theme().into()
    }
}

impl MenuBar {
    pub fn theme() -> Theme<MenuBar> {
        let base_theme = PseudoTheme::deferred(None, MenuBar::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .width(Val::Percent(100.))
            .height(Val::Px(theme_spacing.areas.medium))
            .border(UiRect::bottom(Val::Px(theme_spacing.borders.extra_small)))
            .background_color(colors.container(Container::SurfaceMid))
            .border_color(colors.accent(Accent::Shadow))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }

    fn frame() -> impl Bundle {
        (
            Name::new("Menu Bar"),
            NodeBundle {
                style: Style {
                    overflow: Overflow::visible(),
                    ..default()
                },
                ..default()
            },
            LockedStyleAttributes::lock(LockableStyleAttribute::Overflow),
        )
    }
}

pub trait UiMenuBarExt {
    fn menu_bar(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<(Entity, MenuBar)>),
    ) -> UiBuilder<Entity>;
}

impl UiMenuBarExt for UiBuilder<'_, Entity> {
    fn menu_bar(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<(Entity, MenuBar)>),
    ) -> UiBuilder<Entity> {
        let id = self.spawn((MenuBar::frame(), MenuBar)).id();

        let mut builder = self.commands().ui_builder((id, MenuBar));
        spawn_children(&mut builder);

        self.commands().ui_builder(id)
    }
}

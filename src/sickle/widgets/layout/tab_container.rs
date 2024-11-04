use bevy::{ecs::world::Command, prelude::*, ui::RelativeCursorPosition};

use sickle_macros::UiContext;
use sickle_ui_scaffold::prelude::*;

use crate::widgets::menus::{
    context_menu::{
        ContextMenu, ContextMenuGenerator, ContextMenuUpdate, GenerateContextMenu,
        ReflectContextMenuGenerator,
    },
    menu_item::{MenuItem, MenuItemConfig, MenuItemUpdate, UiMenuItemExt},
};

use super::{
    container::UiContainerExt,
    floating_panel::{
        FloatingPanel, FloatingPanelConfig, FloatingPanelLayout, FloatingPanelUpdate,
        UiFloatingPanelExt, UpdateFloatingPanelPanelId,
    },
    label::{LabelConfig, UiLabelExt},
    panel::{Panel, UiPanelExt},
    scroll_view::UiScrollViewExt,
    sized_zone::{SizedZonePreUpdate, SizedZoneResizeHandleContainer},
};

pub struct TabContainerPlugin;

impl Plugin for TabContainerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            TabContainerUpdate
                .after(DraggableUpdate)
                .before(FloatingPanelUpdate),
        )
        .add_plugins((
            ComponentThemePlugin::<TabContainer>::default(),
            ComponentThemePlugin::<TabPlaceholder>::default(),
            ComponentThemePlugin::<Tab>::default(),
        ))
        .register_type::<Tab>()
        .add_systems(PreUpdate, popout_panel_from_tab.before(SizedZonePreUpdate))
        .add_systems(
            Update,
            (
                close_tab_on_context_menu_press,
                popout_tab_on_context_menu_press,
            )
                .after(MenuItemUpdate)
                .before(ContextMenuUpdate)
                .before(TabContainerUpdate),
        )
        .add_systems(
            Update,
            (
                update_tab_container_on_tab_press,
                update_tab_container_on_change,
                update_sized_zone_resize_handles_on_tab_drag,
                handle_tab_dragging,
            )
                .chain()
                .in_set(TabContainerUpdate),
        )
        .add_systems(PostUpdate, dock_panel_in_tab_container.before(ThemeUpdate));
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct TabContainerUpdate;

fn dock_panel_in_tab_container(
    mut q_docking_panels: Query<
        (Entity, &mut TabContainer, &DockFloatingPanel),
        Added<DockFloatingPanel>,
    >,
    q_floating_panel: Query<&FloatingPanel>,
    q_panel: Query<&Panel>,
    mut commands: Commands,
) {
    for (container_id, mut tab_container, dock_ref) in &mut q_docking_panels {
        commands.entity(container_id).remove::<DockFloatingPanel>();

        let Ok(floating_panel) = q_floating_panel.get(dock_ref.floating_panel) else {
            warn!(
                "Failed to dock floating panel {}: Not a FloatingPanel",
                dock_ref.floating_panel
            );
            continue;
        };

        let panel_id = floating_panel.content_panel_id();

        let Ok(panel) = q_panel.get(panel_id) else {
            warn!(
                "Failed to dock floating panel {}: Missing Panel {}",
                dock_ref.floating_panel, panel_id
            );
            continue;
        };

        let bar_id = tab_container.bar;
        let viewport_id = tab_container.viewport;

        let mut tab = Tab {
            container: container_id,
            bar: bar_id,
            panel: panel_id,
            ..default()
        };

        commands
            .ui_builder(bar_id)
            .container(
                Tab::frame(format!("Tab [{}]", panel.title())),
                |container| {
                    tab.label_container = container
                        .container(NodeBundle::default(), |container| {
                            tab.label = container
                                .label(LabelConfig {
                                    label: panel.title(),
                                    ..default()
                                })
                                .id();
                        })
                        .id();
                },
            )
            .insert(tab);

        commands.entity(viewport_id).add_child(panel_id);
        commands.entity(dock_ref.floating_panel).despawn_recursive();

        tab_container.tab_count += 1;
        tab_container.active = tab_container.tab_count - 1;
    }
}

fn popout_panel_from_tab(
    q_popout: Query<
        (Entity, &Tab, &PopoutPanelFromTabContainer),
        Added<PopoutPanelFromTabContainer>,
    >,
    q_panel: Query<&Panel>,
    q_parent: Query<&Parent>,
    q_ui_context_root: Query<&UiContextRoot>,
    mut q_tab_container: Query<&mut TabContainer>,
    mut commands: Commands,
) {
    for (entity, tab, popout_ref) in &q_popout {
        commands
            .entity(entity)
            .remove::<PopoutPanelFromTabContainer>();

        let tab_contaier_id = tab.container;

        let Ok(mut tab_container) = q_tab_container.get_mut(tab_contaier_id) else {
            warn!(
                "Failed to remove Tab {}: {} is not a TabContainer!",
                entity, tab_contaier_id,
            );
            continue;
        };
        tab_container.tab_count = match tab_container.tab_count > 1 {
            true => tab_container.tab_count - 1,
            false => 0,
        };

        if tab_container.active >= tab_container.tab_count {
            tab_container.active = match tab_container.tab_count > 0 {
                true => tab_container.tab_count - 1,
                false => 0,
            };
        }

        let panel_id = tab.panel;
        let Ok(panel) = q_panel.get(panel_id) else {
            warn!("Cannot pop out panel {}: Not a Panel", panel_id);
            continue;
        };
        let title = panel.title();

        let root_node = q_parent
            .iter_ancestors(tab_contaier_id)
            .find(|parent| q_ui_context_root.get(*parent).is_ok())
            .or(q_parent.iter_ancestors(tab_contaier_id).last())
            .unwrap_or(tab_contaier_id);

        commands.entity(entity).despawn_recursive();
        let floating_panel_id = commands
            .ui_builder(root_node)
            .floating_panel(
                FloatingPanelConfig {
                    title: title.into(),
                    ..default()
                },
                FloatingPanelLayout {
                    size: popout_ref.size,
                    position: popout_ref.position.into(),
                    droppable: true,
                    ..default()
                },
                |_| {},
            )
            .id();

        commands.entity(panel_id).set_parent(root_node);
        commands.style(panel_id).hide();
        commands
            .entity(floating_panel_id)
            .insert(UpdateFloatingPanelPanelId { panel_id });
    }
}

fn close_tab_on_context_menu_press(
    q_menu_items: Query<(Entity, &CloseTabContextMenu, &MenuItem), Changed<MenuItem>>,
    q_tab: Query<&Tab>,
    mut q_tab_container: Query<&mut TabContainer>,
    mut commands: Commands,
) {
    for (entity, context_menu, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            let Ok(tab_data) = q_tab.get(context_menu.tab) else {
                warn!(
                    "Context menu {} refers to missing tab {}",
                    entity, context_menu.tab
                );
                continue;
            };

            let tab_contaier_id = tab_data.container;
            let Ok(mut tab_container) = q_tab_container.get_mut(tab_contaier_id) else {
                warn!(
                    "Failed to remove Tab {}: {} is not a TabContainer!",
                    entity, tab_contaier_id,
                );
                continue;
            };
            tab_container.tab_count = match tab_container.tab_count > 1 {
                true => tab_container.tab_count - 1,
                false => 0,
            };

            commands.entity(context_menu.tab).despawn_recursive();
            commands.entity(tab_data.panel).despawn_recursive();
        }
    }
}

fn popout_tab_on_context_menu_press(
    q_menu_items: Query<(Entity, &PopoutTabContextMenu, &MenuItem), Changed<MenuItem>>,
    q_tab: Query<(&Tab, &GlobalTransform)>,
    q_node: Query<&Node>,
    mut commands: Commands,
) {
    for (entity, tab_ref, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            let Ok((tab, transform)) = q_tab.get(tab_ref.tab) else {
                warn!(
                    "Context menu tab reference {} refers to missing tab {}",
                    entity, tab_ref.tab
                );
                continue;
            };

            let Ok(container) = q_node.get(tab.container) else {
                warn!(
                    "Context menu tab reference {} refers to a tab without a container {}",
                    entity, tab_ref.tab
                );
                continue;
            };

            let size = container.size() * 0.8;
            let position = transform.translation().truncate();
            commands
                .entity(tab_ref.tab)
                .insert(PopoutPanelFromTabContainer { size, position });
        }
    }
}

fn update_tab_container_on_tab_press(
    q_tabs: Query<(Entity, &Tab, &Interaction), Changed<Interaction>>,
    q_tab: Query<Entity, With<Tab>>,
    q_children: Query<&Children>,
    mut q_tab_container: Query<&mut TabContainer>,
) {
    for (tab_entity, tab, interaction) in &q_tabs {
        if *interaction == Interaction::Pressed {
            let Ok(mut tab_container) = q_tab_container.get_mut(tab.container) else {
                continue;
            };

            let Ok(tabs) = q_children.get(tab_container.bar) else {
                continue;
            };

            for (i, id) in tabs.iter().enumerate() {
                if let Ok(_) = q_tab.get(*id) {
                    if *id == tab_entity {
                        tab_container.active = i;
                    }
                }
            }
        }
    }
}

fn update_tab_container_on_change(
    q_tab_containers: Query<&TabContainer, Changed<TabContainer>>,
    q_tab: Query<Entity, With<Tab>>,
    q_children: Query<&Children>,
    mut commands: Commands,
) {
    for tab_container in &q_tab_containers {
        let Ok(tabs) = q_children.get(tab_container.bar) else {
            continue;
        };

        for (i, id) in tabs.iter().enumerate() {
            if let Ok(tab_entity) = q_tab.get(*id) {
                if i == tab_container.active {
                    commands
                        .entity(tab_entity)
                        .add_pseudo_state(PseudoState::Selected);
                } else {
                    commands
                        .entity(tab_entity)
                        .remove_pseudo_state(PseudoState::Selected);
                }
            }
        }
    }
}

// TODO: Replace this when focus management is implemented
fn update_sized_zone_resize_handles_on_tab_drag(
    q_accepted_types: Query<&Draggable, (With<Tab>, Changed<Draggable>)>,
    q_handle_containers: Query<Entity, With<SizedZoneResizeHandleContainer>>,
    mut commands: Commands,
) {
    if q_accepted_types
        .iter()
        .all(|draggable| draggable.state == DragState::Inactive)
    {
        return;
    }

    let dragging = q_accepted_types.iter().any(|draggable| {
        draggable.state == DragState::DragStart || draggable.state == DragState::Dragging
    });

    for container in &q_handle_containers {
        commands.style(container).render(!dragging);
    }
}

fn handle_tab_dragging(
    q_tabs: Query<(Entity, &Draggable, &Node, &Transform), (With<Tab>, Changed<Draggable>)>,
    q_tab_container: Query<&TabContainer>,
    q_tab_bar: Query<&Node, With<TabBar>>,
    q_children: Query<&Children>,
    q_transform: Query<(&GlobalTransform, &Interaction)>,
    mut q_tab: Query<&mut Tab>,
    mut commands: Commands,
) {
    for (entity, draggable, node, transform) in &q_tabs {
        let tab = q_tab.get(entity).unwrap();

        let Ok(container) = q_tab_container.get(tab.container) else {
            warn!("Tried to drag orphan Tab {}", entity);
            continue;
        };

        let Ok(bar_node) = q_tab_bar.get(container.bar) else {
            error!("Tab container {} doesn't have a tab bar", tab.container);
            continue;
        };

        let Ok(children) = q_children.get(container.bar) else {
            error!("Tab container has no tabs {}", tab.container);
            continue;
        };

        if children
            .iter()
            .filter(|child| q_tab.get(**child).is_ok())
            .count()
            < 2
        {
            continue;
        }

        let bar_half_width = bar_node.size().x / 2.;
        match draggable.state {
            DragState::DragStart => {
                commands
                    .style_unchecked(container.bar)
                    .overflow(Overflow::visible());

                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).disable_flux_interaction();
                    }
                });

                let Some(tab_index) = children
                    .iter()
                    .filter(|child| q_tab.get(**child).is_ok())
                    .position(|child| *child == entity)
                else {
                    error!("Tab {} isn't a child of its tab container bar", entity);
                    continue;
                };

                let left =
                    transform.translation.truncate().x - (node.size().x / 2.) + bar_half_width;
                let placeholder = commands
                    .ui_builder(container.bar)
                    .tab_placeholder(node.size().x)
                    .id();

                commands
                    .entity(container.bar)
                    .insert_children(tab_index, &[placeholder]);

                commands
                    .ui_builder(entity)
                    .style_unchecked()
                    .position_type(PositionType::Absolute)
                    .left(Val::Px(left))
                    .z_index(ZIndex::Local(100));

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = placeholder.into();
                tab.original_index = tab_index.into();
            }
            DragState::Dragging => {
                let Some(diff) = draggable.diff else {
                    continue;
                };
                let Some(position) = draggable.position else {
                    continue;
                };

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {} missing placeholder", entity);
                    continue;
                };

                let new_x = transform.translation.truncate().x + diff.x + bar_half_width;
                let left = new_x - (node.size().x / 2.);
                let mut new_index: Option<usize> = None;
                let mut placeholder_index = children.len();
                for (i, child) in children.iter().enumerate() {
                    if *child == entity {
                        continue;
                    }
                    if *child == placeholder {
                        placeholder_index = i;
                        continue;
                    }
                    let Ok(_) = q_tab.get(entity) else {
                        continue;
                    };
                    let Ok((transform, interaction)) = q_transform.get(*child) else {
                        continue;
                    };

                    if *interaction == Interaction::Hovered {
                        if position.x < transform.translation().truncate().x {
                            if i < placeholder_index {
                                new_index = i.into();
                            } else {
                                // placeholder is between 0 and children.len or less
                                new_index = (i - 1).into();
                            }
                        } else {
                            if i + 1 < placeholder_index {
                                new_index = (i + 1).into();
                            } else {
                                // placeholder is between 0 and children.len or less
                                new_index = i.into();
                            }
                        }

                        break;
                    }
                }

                if let Some(new_index) = new_index {
                    commands
                        .entity(container.bar)
                        .insert_children(new_index, &[placeholder]);
                }

                commands
                    .ui_builder(entity)
                    .style_unchecked()
                    .left(Val::Px(left));
            }
            DragState::DragEnd => {
                commands
                    .style_unchecked(container.bar)
                    .overflow(Overflow::clip());

                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).enable_flux_interaction();
                    }
                });

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {} missing placeholder", entity);
                    continue;
                };

                let Some(placeholder_index) =
                    children.iter().position(|child| *child == placeholder)
                else {
                    error!(
                        "Tab placeholder {} isn't a child of its tab container bar",
                        entity
                    );
                    continue;
                };

                commands
                    .style_unchecked(entity)
                    .position_type(PositionType::Relative)
                    .left(Val::Auto)
                    .z_index(ZIndex::Local(0));

                commands
                    .entity(container.bar)
                    .insert_children(placeholder_index, &[entity]);

                commands.entity(placeholder).despawn_recursive();

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = None;
                tab.original_index = None;
            }
            DragState::DragCanceled => {
                commands
                    .style_unchecked(container.bar)
                    .overflow(Overflow::clip());

                children.iter().for_each(|child| {
                    if *child != entity && q_tab.get(*child).is_ok() {
                        commands.style(*child).enable_flux_interaction();
                    }
                });

                let Some(placeholder) = tab.placeholder else {
                    warn!("Tab {} missing placeholder", entity);
                    continue;
                };

                let original_index = tab.original_index.unwrap_or(0);

                commands
                    .style_unchecked(entity)
                    .position_type(PositionType::Relative)
                    .left(Val::Auto)
                    .z_index(ZIndex::Local(0));

                commands.entity(placeholder).despawn_recursive();

                commands
                    .entity(container.bar)
                    .insert_children(original_index, &[entity]);

                let mut tab = q_tab.get_mut(entity).unwrap();
                tab.placeholder = None;
                tab.original_index = None;
            }
            _ => continue,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CloseTabContextMenu {
    tab: Entity,
}

impl Default for CloseTabContextMenu {
    fn default() -> Self {
        Self {
            tab: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct PopoutTabContextMenu {
    tab: Entity,
}

impl Default for PopoutTabContextMenu {
    fn default() -> Self {
        Self {
            tab: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, ContextMenuGenerator)]
pub struct Tab {
    container: Entity,
    bar: Entity,
    panel: Entity,
    label_container: Entity,
    label: Entity,
    placeholder: Option<Entity>,
    original_index: Option<usize>,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
            bar: Entity::PLACEHOLDER,
            panel: Entity::PLACEHOLDER,
            label_container: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
            placeholder: None,
            original_index: None,
        }
    }
}

impl ContextMenuGenerator for Tab {
    fn build_context_menu(&self, context: Entity, container: &mut UiBuilder<ContextMenu>) {
        let icons = ThemeData::default().icons;

        container
            .menu_item(MenuItemConfig {
                name: "Close Tab".into(),
                leading_icon: icons.close,
                ..default()
            })
            .insert(CloseTabContextMenu { tab: context });
        container
            .menu_item(MenuItemConfig {
                name: "Popout Tab".into(),
                trailing_icon: icons.open_in_new,
                ..default()
            })
            .insert(PopoutTabContextMenu { tab: context });
    }

    fn placement_index(&self) -> usize {
        0
    }
}

impl UiContext for Tab {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            Tab::LABEL_CONTAINER => Ok(self.label_container),
            Tab::LABEL => Ok(self.label),
            Tab::PANEL => Ok(self.panel),
            _ => Err(format!(
                "{} doesn't exist for Tab. Possible contexts: {:?}",
                target,
                Vec::from_iter(self.contexts())
            )),
        }
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_ {
        [Tab::LABEL_CONTAINER, Tab::LABEL, Tab::PANEL].into_iter()
    }
}

impl DefaultTheme for Tab {
    fn default_theme() -> Option<Theme<Tab>> {
        Tab::theme().into()
    }
}

impl Tab {
    pub const LABEL_CONTAINER: &'static str = "LabelContainer";
    pub const LABEL: &'static str = "Label";
    pub const PANEL: &'static str = "Panel";

    pub fn theme() -> Theme<Tab> {
        let base_theme = PseudoTheme::deferred(None, Tab::primary_style);
        let selected_theme =
            PseudoTheme::deferred(vec![PseudoState::Selected], Tab::selected_style);
        Theme::new(vec![base_theme, selected_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .padding(UiRect::bottom(Val::Px(theme_spacing.gaps.small)))
            .border(UiRect::right(Val::Px(theme_spacing.borders.small)))
            .border_color(colors.accent(Accent::OutlineVariant))
            .border_radius(BorderRadius::top(Val::Px(theme_spacing.corners.small)))
            .bottom(Val::Px(0.))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::SurfaceMid),
                hover: colors.container(Container::SurfaceHighest).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(Tab::LABEL_CONTAINER)
            .size(Val::Percent(100.))
            .padding(UiRect::px(
                theme_spacing.gaps.medium,
                theme_spacing.gaps.medium,
                theme_spacing.gaps.small,
                0.,
            ))
            .border(UiRect::all(Val::Px(0.)))
            .border_color(Color::NONE);

        style_builder
            .switch_target(Tab::LABEL)
            .sized_font(font)
            .font_color(colors.on(OnColor::Surface));

        style_builder
            .switch_target(Tab::PANEL)
            .position_type(PositionType::Absolute)
            .visibility(Visibility::Hidden);
    }

    fn selected_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .background_color(colors.surface(Surface::Surface))
            .animated()
            .bottom(AnimatedVals {
                idle: Val::Px(-theme_spacing.gaps.tiny),
                enter_from: Val::Px(0.).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .animated()
            .padding(AnimatedVals {
                idle: UiRect::bottom(Val::Px(theme_spacing.gaps.small + theme_spacing.gaps.tiny)),
                enter_from: UiRect::bottom(Val::Px(theme_spacing.gaps.small)).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(Tab::LABEL_CONTAINER)
            .border(UiRect::top(Val::Px(theme_spacing.borders.extra_small)))
            .animated()
            .border_color(AnimatedVals {
                idle: colors.accent(Accent::Outline),
                enter_from: Color::NONE.into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(Tab::PANEL)
            .visibility(Visibility::Inherited);
    }

    fn frame(name: String) -> impl Bundle {
        (
            Name::new(name),
            NodeBundle::default(),
            Interaction::default(),
            TrackedInteraction::default(),
            Draggable::default(),
            RelativeCursorPosition::default(),
            GenerateContextMenu::default(),
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::PositionType,
                LockableStyleAttribute::Left,
                LockableStyleAttribute::ZIndex,
            ]),
        )
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct PopoutPanelFromTabContainer {
    size: Vec2,
    position: Vec2,
}

struct IncrementTabCount {
    container: Entity,
}

impl Command for IncrementTabCount {
    fn apply(self, world: &mut World) {
        let Some(mut container) = world.get_mut::<TabContainer>(self.container) else {
            warn!(
                "Failed to increment tab count: {} is not a TabContainer!",
                self.container,
            );
            return;
        };

        container.tab_count += 1;
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct DockFloatingPanel {
    floating_panel: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TabBar {
    container: Entity,
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

impl TabBar {
    pub fn container_id(&self) -> Entity {
        self.container
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct TabViewport {
    container: Entity,
}

impl Default for TabViewport {
    fn default() -> Self {
        Self {
            container: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Clone, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct TabPlaceholder {
    width: f32,
}

impl DefaultTheme for TabPlaceholder {
    fn default_theme() -> Option<Theme<TabPlaceholder>> {
        TabPlaceholder::theme().into()
    }
}

impl TabPlaceholder {
    pub fn theme() -> Theme<TabPlaceholder> {
        let base_theme = PseudoTheme::deferred_context(None, TabPlaceholder::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(
        style_builder: &mut StyleBuilder,
        context: &TabPlaceholder,
        theme_data: &ThemeData,
    ) {
        let colors = theme_data.colors();

        style_builder
            .background_color(colors.accent(Accent::Outline))
            .animated()
            .width(AnimatedVals {
                idle: Val::Px(context.width * 1.1),
                enter_from: Val::Px(context.width).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);
    }

    fn frame(width: f32) -> impl Bundle {
        (
            Name::new("Tab Placeholder"),
            NodeBundle {
                style: Style {
                    width: Val::Px(width),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
            LockedStyleAttributes::lock(LockableStyleAttribute::PositionType),
        )
    }
}

pub trait UiTabPlaceholderExt {
    fn tab_placeholder(&mut self, width: f32) -> UiBuilder<Entity>;
}

impl UiTabPlaceholderExt for UiBuilder<'_, Entity> {
    fn tab_placeholder(&mut self, width: f32) -> UiBuilder<Entity> {
        self.spawn((TabPlaceholder::frame(width), TabPlaceholder { width }))
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct TabContainer {
    active: usize,
    bar: Entity,
    viewport: Entity,
    tab_count: usize,
}

impl Default for TabContainer {
    fn default() -> Self {
        Self {
            active: 0,
            tab_count: 0,
            bar: Entity::PLACEHOLDER,
            viewport: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for TabContainer {
    fn default_theme() -> Option<Theme<TabContainer>> {
        TabContainer::theme().into()
    }
}

impl UiContext for TabContainer {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            TabContainer::TAB_BAR => Ok(self.bar),
            _ => Err(format!(
                "{} doesn't exist for TabContainer. Possible contexts: {:?}",
                target,
                Vec::from_iter(self.contexts())
            )),
        }
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_ {
        [TabContainer::TAB_BAR].into_iter()
    }
}

impl TabContainer {
    pub const TAB_BAR: &'static str = "TabBar";

    pub fn bar_id(&self) -> Entity {
        self.bar
    }

    pub fn tab_count(&self) -> usize {
        self.tab_count
    }

    pub fn set_active(&mut self, active: usize) {
        self.active = active;
    }

    pub fn theme() -> Theme<TabContainer> {
        let base_theme = PseudoTheme::deferred(None, TabContainer::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .width(Val::Percent(100.))
            .height(Val::Percent(100.))
            .flex_direction(FlexDirection::Column);

        style_builder
            .switch_target(TabContainer::TAB_BAR)
            .width(Val::Percent(100.))
            .height(Val::Px(theme_spacing.areas.medium))
            .padding(UiRect::top(Val::Px(theme_spacing.gaps.tiny)))
            .border(UiRect::bottom(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.surface(Surface::Surface));
    }
}

impl TabContainer {
    fn frame() -> impl Bundle {
        (
            Name::new("Tab Container"),
            NodeBundle::default(),
            Interaction::default(),
        )
    }

    fn bar() -> impl Bundle {
        (
            Name::new("Tab Bar"),
            NodeBundle {
                style: Style {
                    overflow: Overflow::clip(),
                    ..default()
                },
                ..default()
            },
            Interaction::default(),
            LockedStyleAttributes::lock(LockableStyleAttribute::Overflow),
        )
    }
}

pub trait UiTabContainerExt {
    fn tab_container(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<(Entity, TabContainer)>),
    ) -> UiBuilder<Entity>;
}

impl UiTabContainerExt for UiBuilder<'_, Entity> {
    /// A simple tab container.
    fn tab_container(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<(Entity, TabContainer)>),
    ) -> UiBuilder<Entity> {
        let mut tab_container = TabContainer { ..default() };

        let mut container = self.container(TabContainer::frame(), |container| {
            let container_id = container.id();

            tab_container.bar = container
                .spawn((
                    TabContainer::bar(),
                    TabBar {
                        container: container_id,
                    },
                ))
                .id();

            container.scroll_view(None, |scroll_view| {
                tab_container.viewport = scroll_view
                    .insert(TabViewport {
                        container: container_id,
                    })
                    .id();
            });
        });

        let container_id = container.id();
        container.insert(tab_container);

        let mut builder = self.commands().ui_builder((container_id, tab_container));
        spawn_children(&mut builder);

        self.commands().ui_builder(container_id)
    }
}

pub trait UiTabContainerSubExt {
    fn id(&self) -> Entity;

    fn add_tab(
        &mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<(Entity, TabContainer)>;

    fn dock_panel(&mut self, floating_panel: Entity) -> UiBuilder<(Entity, TabContainer)>;
}

impl UiTabContainerSubExt for UiBuilder<'_, (Entity, TabContainer)> {
    fn id(&self) -> Entity {
        self.context().0
    }

    /// Adds a tab to the TabContainer
    ///
    /// ### PseudoState usage
    /// - `PseudoState::Selected` is added to the tab currently selected per TabContainer
    fn add_tab(
        &mut self,
        title: String,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<(Entity, TabContainer)> {
        let context = self.context().clone();
        let container_id = context.0;
        let bar_id = context.1.bar;
        let viewport_id = context.1.viewport;
        let panel = self
            .commands()
            .ui_builder(viewport_id)
            .panel(title.clone(), spawn_children)
            .id();

        let mut tab = Tab {
            container: container_id,
            bar: bar_id,
            panel,
            ..default()
        };

        self.commands()
            .ui_builder(bar_id)
            .container(
                Tab::frame(format!("Tab [{}]", title.clone())),
                |container| {
                    tab.label_container = container
                        .container(NodeBundle::default(), |container| {
                            tab.label = container
                                .label(LabelConfig {
                                    label: title,
                                    ..default()
                                })
                                .id();
                        })
                        .id();
                },
            )
            .insert(tab);

        self.commands().add(IncrementTabCount {
            container: container_id,
        });
        self.commands().ui_builder(context)
    }

    fn dock_panel(&mut self, floating_panel: Entity) -> UiBuilder<(Entity, TabContainer)> {
        let context = self.context().clone();
        let entity = self.id();

        self.commands()
            .entity(entity)
            .insert(DockFloatingPanel { floating_panel });
        self.commands().ui_builder(context)
    }
}

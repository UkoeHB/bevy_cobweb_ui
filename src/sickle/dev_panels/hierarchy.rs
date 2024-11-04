use bevy::prelude::*;

use sickle_ui_scaffold::{prelude::*, theme::icons::Icons};

use crate::widgets::{
    layout::{
        column::UiColumnExt,
        foldable::{Foldable, UiFoldableExt},
        panel::UiPanelExt,
        row::UiRowExt,
        scroll_view::UiScrollViewExt,
        sized_zone::{SizedZoneConfig, UiSizedZoneExt},
    },
    menus::menu_item::{MenuItem, MenuItemConfig, UiMenuItemExt},
};

use super::entity_component_list::{
    EntityComponentList, EntityComponentListPlugin, UiEntityComponentListExt,
};

// TODO: Move to subapp? to separate inspection from UI entities
pub struct HierarchyTreeViewPlugin;

impl Plugin for HierarchyTreeViewPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntityComponentListPlugin>() {
            app.add_plugins(EntityComponentListPlugin);
        }

        app.add_systems(
            PreUpdate,
            (
                refresh_hierarchy_on_press,
                initialize_hierarchy_tree_view,
                update_hierarchy_on_foldable_change,
                update_hierarchy_selection,
                update_hierarchy_node_style,
                update_entity_component_list,
            )
                .chain()
                .in_set(HierarchyPreUpdate),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct HierarchyPreUpdate;

fn initialize_hierarchy_tree_view(
    q_hierarchy_nodes: Query<(Entity, &HierarchyNodeContainer), Added<HierarchyNodeContainer>>,
    q_hierarchy: Query<&HierarchyContainer>,
    q_name: Query<&Name>,
    mut commands: Commands,
) {
    for (entity, node_container) in &q_hierarchy_nodes {
        let Ok(hierarchy) = q_hierarchy.get(node_container.hierarchy) else {
            warn!(
                "Hierarchy node container {} missing main container {}",
                entity, node_container.hierarchy
            );
            continue;
        };

        let mut container = commands.ui_builder(entity);
        spawn_hierarchy_level(
            node_container.hierarchy,
            hierarchy.root,
            &mut container,
            &q_name,
        );
    }
}

fn refresh_hierarchy_on_press(
    q_menu_items: Query<(&MenuItem, &RefreshHierarchyButton), Changed<MenuItem>>,
    q_name: Query<&Name>,
    mut q_hierarchy: Query<&mut HierarchyContainer>,
    mut commands: Commands,
) {
    for (menu_item, refresh_button) in &q_menu_items {
        if menu_item.interacted() {
            let Ok(mut hierarchy) = q_hierarchy.get_mut(refresh_button.hierarchy) else {
                continue;
            };

            hierarchy.selected = None;

            commands
                .entity(refresh_button.container)
                .despawn_descendants();

            let mut builder = commands.ui_builder(refresh_button.container);

            spawn_hierarchy_level(
                refresh_button.hierarchy,
                hierarchy.root,
                &mut builder,
                &q_name,
            );

            break;
        }
    }
}

fn update_hierarchy_selection(
    q_hierarchy_nodes: Query<(&FluxInteraction, &HierarchyNode), Changed<FluxInteraction>>,
    mut q_hierarchy: Query<&mut HierarchyContainer>,
) {
    for (interaction, hierarchy_node) in &q_hierarchy_nodes {
        if interaction.is_released() {
            let Ok(mut hierarchy) = q_hierarchy.get_mut(hierarchy_node.hierarchy) else {
                continue;
            };

            if hierarchy.selected != hierarchy_node.entity.into() {
                hierarchy.selected = hierarchy_node.entity.into();
            }
        }
    }
}

fn update_entity_component_list(
    q_hierarchies: Query<&mut HierarchyContainer, Changed<HierarchyContainer>>,
    mut q_entity_component_list: Query<&mut EntityComponentList>,
) {
    for hierarchy in &q_hierarchies {
        let Ok(mut component_list) = q_entity_component_list.get_mut(hierarchy.component_list)
        else {
            continue;
        };

        if component_list.entity != hierarchy.selected {
            component_list.entity = hierarchy.selected;
        }
    }
}

fn update_hierarchy_on_foldable_change(
    mut q_foldables: Query<(&HierarchyNode, &mut Foldable), Changed<Foldable>>,
    q_children: Query<&Children>,
    q_name: Query<&Name>,
    mut commands: Commands,
) {
    for (hierarchy_node, mut foldable) in &mut q_foldables {
        if foldable.empty {
            continue;
        }

        commands.entity(foldable.container()).despawn_descendants();

        if foldable.open {
            if let Ok(children) = q_children.get(hierarchy_node.entity) {
                let mut builder = commands.ui_builder(foldable.container());
                for child in children.iter() {
                    spawn_hierarchy_level(hierarchy_node.hierarchy, *child, &mut builder, &q_name);
                }
            } else {
                foldable.empty = true;
            }
        } else if q_children.get(hierarchy_node.entity).is_err() {
            foldable.empty = true;
        }
    }
}

// TODO: Refresh the hierarchy automatically
// TODO: Rework hierarchy: use treeview with node callbacks, add search, pop-out,
// anchestor access, theme, separate world for layout (or filter itself) etc. Tag open entities per hierarchy
fn update_hierarchy_node_style(
    q_hierarchies: Query<(Entity, &HierarchyContainer), Changed<HierarchyContainer>>,
    q_hierarchy_nodes: Query<(Entity, &HierarchyNode)>,
    mut commands: Commands,
) {
    for (entity, hierarchy) in &q_hierarchies {
        for (foldable, hierarchy_node) in q_hierarchy_nodes
            .iter()
            .filter(|(_, node)| node.hierarchy == entity)
        {
            if let Some(selected) = hierarchy.selected {
                if hierarchy_node.entity == selected {
                    commands
                        .entity(foldable)
                        .add_pseudo_state(PseudoState::Selected);
                } else {
                    commands
                        .entity(foldable)
                        .remove_pseudo_state(PseudoState::Selected);
                }
            } else {
                commands
                    .entity(foldable)
                    .remove_pseudo_state(PseudoState::Selected);
            }
        }
    }
}

fn spawn_hierarchy_level(
    hierarchy: Entity,
    entity: Entity,
    container: &mut UiBuilder<'_, Entity>,
    q_name: &Query<&Name>,
) {
    let name = match q_name.get(entity) {
        Ok(name) => format!("[{}] {}", entity, name),
        Err(_) => format!("[{}]", entity),
    };

    container
        .foldable(name, false, false, |_| {})
        .insert(HierarchyNode { hierarchy, entity });
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct HierarchyNodeContainer {
    hierarchy: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct RefreshHierarchyButton {
    hierarchy: Entity,
    container: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct HierarchyNode {
    hierarchy: Entity,
    entity: Entity,
}

impl HierarchyNode {
    pub fn target(&self) -> Entity {
        self.entity
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct HierarchyContainer {
    root: Entity,
    selected: Option<Entity>,
    component_list: Entity,
}

impl HierarchyContainer {
    pub fn foldable_theme() -> Theme<Foldable> {
        let base_theme = PseudoTheme::deferred(None, HierarchyContainer::foldable_style);
        let selected_theme = PseudoTheme::deferred(
            vec![PseudoState::Selected],
            HierarchyContainer::selected_style,
        );

        Theme::new(vec![base_theme, selected_theme])
    }

    pub fn refresh_button_theme() -> Theme<MenuItem> {
        let base_theme = PseudoTheme::deferred(None, HierarchyContainer::refresh_button_style);

        Theme::new(vec![base_theme])
    }

    fn refresh_button_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;

        style_builder
            .border(UiRect::bottom(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(theme_data.colors().accent(Accent::Shadow))
            .width(Val::Percent(100.));
    }

    fn foldable_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder.background_color(Color::NONE);

        style_builder
            .switch_target(Foldable::CONTAINER)
            .margin(UiRect::left(Val::Px(theme_spacing.gaps.medium)))
            .border(UiRect::left(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::OutlineVariant));
    }

    fn selected_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let colors = theme_data.colors();

        style_builder.background_color(colors.container(Container::Tertiary));
    }
}

pub trait UiHierarchyExt {
    fn hierarchy_for(&mut self, root_entity: Entity) -> UiBuilder<Entity>;
}

impl UiHierarchyExt for UiBuilder<'_, Entity> {
    fn hierarchy_for(&mut self, root_entity: Entity) -> UiBuilder<Entity> {
        self.column(|column| {
            column.insert(HierarchyContainer::foldable_theme());
            column.style().width(Val::Percent(100.));

            let main_zone = column
                .sized_zone(
                    SizedZoneConfig {
                        size: 70.,
                        min_size: 200.,
                    },
                    |zone| {
                        let hierarchy_id = zone.id();
                        let mut refresh_button = Entity::PLACEHOLDER;
                        zone.panel("Hierarchy content".into(), |panel| {
                            panel
                                .row(|row| {
                                    refresh_button = row
                                        .menu_item(MenuItemConfig {
                                            name: "Refresh".into(),
                                            trailing_icon: Icons::default().refresh,
                                            ..default()
                                        })
                                        .id();
                                })
                                .insert(HierarchyContainer::refresh_button_theme());

                            panel.scroll_view(None, |scroll_view| {
                                let node_container = scroll_view
                                    .column(|_| {})
                                    .insert(HierarchyNodeContainer {
                                        hierarchy: hierarchy_id,
                                    })
                                    .id();

                                scroll_view.commands().entity(refresh_button).insert(
                                    RefreshHierarchyButton {
                                        hierarchy: hierarchy_id,
                                        container: node_container,
                                    },
                                );
                            });
                        });
                    },
                )
                .id();

            let mut component_list = Entity::PLACEHOLDER;
            column.sized_zone(
                SizedZoneConfig {
                    size: 25.,
                    ..default()
                },
                |zone| {
                    component_list = zone.entity_component_list(None).id();
                },
            );

            column.commands().ui_builder(main_zone).insert((
                Name::new(format!("Hierarchy of [{}]", root_entity)),
                HierarchyContainer {
                    root: root_entity,
                    selected: None,
                    component_list,
                },
            ));
        })
    }
}

use bevy::{prelude::*, ui::UiSystem};

use sickle_ui_scaffold::{prelude::*, ui_commands::LogHierarchyExt};

use super::{
    container::UiContainerExt,
    docking_zone::DockingZoneUpdate,
    resize_handles::{ResizeHandle, UiResizeHandlesExt},
};

const MIN_SIZED_ZONE_SIZE: f32 = 50.;

pub struct SizedZonePlugin;

impl Plugin for SizedZonePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            HierarchyToPseudoState::<SizedZone>::new(),
            ComponentThemePlugin::<SizedZone>::default(),
        ))
        .add_systems(
            PreUpdate,
            (
                preset_sized_zone_flex_layout,
                preset_sized_zone_children_size,
            )
                .chain()
                .in_set(SizedZonePreUpdate)
                .run_if(should_update_sized_zone_layout),
        )
        .add_systems(
            Update,
            (update_sized_zone_on_resize, update_sized_zone_style)
                .after(DockingZoneUpdate)
                .chain(),
        )
        .add_systems(
            PostUpdate,
            fit_sized_zones_on_window_resize
                .run_if(should_fit_sized_zones)
                .after(UiSystem::Layout),
        )
        .add_systems(
            PostUpdate,
            update_sized_zone_resize_handles
                .run_if(should_update_sized_zone_layout)
                .after(fit_sized_zones_on_window_resize),
        );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SizedZonePreUpdate;

fn should_update_sized_zone_layout(
    q_added_zones: Query<Entity, Added<SizedZone>>,
    q_parent_children_changed_zones: Query<
        Entity,
        (With<SizedZone>, Or<(Changed<Parent>, Changed<Children>)>),
    >,
    q_removed_zones: RemovedComponents<SizedZone>,
) -> bool {
    q_added_zones.iter().count() > 0
        || q_parent_children_changed_zones.iter().count() > 0
        || q_removed_zones.len() > 0
}

fn preset_sized_zone_flex_layout(
    q_sized_zones: Query<(Entity, &Parent), With<SizedZone>>,
    mut q_sized_zone: Query<&mut SizedZone>,
    q_children: Query<&Children>,
    q_style: Query<&Style>,
) {
    let static_zones: Vec<(Entity, Entity)> = q_sized_zones
        .iter()
        .filter(|(_, parent)| q_sized_zone.get(parent.get()).is_err())
        .map(|(e, p)| (e, p.get()))
        .collect();

    for (sized_zone, parent) in static_zones {
        let Ok(parent_style) = q_style.get(parent) else {
            warn!("No Style found for sized zone parent {}!", parent);
            continue;
        };

        let parent_flex_direction = parent_style.flex_direction;
        preset_drop_zone_flex_direction(
            sized_zone,
            &mut q_sized_zone,
            &q_children,
            parent_flex_direction,
        );
    }
}

fn preset_drop_zone_flex_direction(
    sized_zone: Entity,
    q_sized_zone: &mut Query<&mut SizedZone>,
    q_children: &Query<&Children>,
    parent_flex_direction: FlexDirection,
) {
    let mut zone = q_sized_zone.get_mut(sized_zone).unwrap();

    zone.flex_direction = match parent_flex_direction {
        FlexDirection::Row => FlexDirection::Column,
        FlexDirection::Column => FlexDirection::Row,
        FlexDirection::RowReverse => FlexDirection::Column,
        FlexDirection::ColumnReverse => FlexDirection::Row,
    };

    let zone_direction = zone.flex_direction;
    if let Ok(children) = q_children.get(sized_zone) {
        for child in children {
            if q_sized_zone.get(*child).is_ok() {
                preset_drop_zone_flex_direction(*child, q_sized_zone, q_children, zone_direction);
            }
        }
    }
}

fn preset_sized_zone_children_size(
    q_sized_zones: Query<Entity, With<SizedZone>>,
    mut q_sized_zone: Query<&mut SizedZone>,
    q_parents: Query<&Parent>,
) {
    for mut zone in &mut q_sized_zone {
        zone.children_size = 0.;
    }

    for entity in &q_sized_zones {
        let zone = q_sized_zone.get(entity).unwrap();
        let zone_size = zone.min_size;
        let direction = zone.flex_direction;

        for parent in q_parents.iter_ancestors(entity) {
            let Ok(mut parent_zone) = q_sized_zone.get_mut(parent) else {
                continue;
            };

            if parent_zone.flex_direction == direction {
                parent_zone.children_size += zone_size;
            }
        }
    }

    for mut zone in &mut q_sized_zone {
        zone.children_size = zone.children_size.max(zone.min_size);
    }
}

fn update_sized_zone_resize_handles(
    q_sized_zone_parents: Query<&Parent, With<SizedZone>>,
    q_children: Query<&Children>,
    q_sized_zones: Query<&SizedZone>,
    q_style: Query<&Style>,
    mut q_resize_handle: Query<&mut SizedZoneResizeHandle>,
    mut commands: Commands,
) {
    let zone_count = q_sized_zone_parents.iter().count();
    let mut handle_visibility: Vec<(Entity, CardinalDirection, bool)> =
        Vec::with_capacity(zone_count * 4);
    let mut handle_neighbours: Vec<(Entity, Option<Entity>)> = Vec::with_capacity(zone_count * 4);
    let parents: Vec<Entity> =
        q_sized_zone_parents
            .iter()
            .fold(Vec::with_capacity(zone_count), |mut acc, parent| {
                let entity = parent.get();
                if !acc.contains(&entity) {
                    acc.push(entity);
                }

                acc
            });

    for parent in parents {
        let children: Vec<Entity> = q_children.get(parent).unwrap().iter().map(|e| *e).collect();
        let child_count = children.len();

        if child_count == 1 {
            let Ok(zone) = q_sized_zones.get(children[0]) else {
                return;
            };
            handle_visibility.push((zone.resize_handles, CardinalDirection::North, false));
            handle_visibility.push((zone.resize_handles, CardinalDirection::East, false));
            handle_visibility.push((zone.resize_handles, CardinalDirection::South, false));
            handle_visibility.push((zone.resize_handles, CardinalDirection::West, false));
        } else {
            let mut zone_children: Vec<Entity> = Vec::with_capacity(child_count);
            let mut prev_is_zone = true;

            for i in 0..child_count {
                let Ok(style) = q_style.get(children[i]) else {
                    warn!(
                        "Missing Style detected on Node {} during sized zone handle update.",
                        children[i]
                    );
                    commands.entity(children[i]).log_hierarchy(None);
                    continue;
                };

                let Ok(zone) = q_sized_zones.get(children[i]) else {
                    if style.position_type == PositionType::Relative {
                        prev_is_zone = false;
                    }
                    continue;
                };

                match zone.flex_direction {
                    FlexDirection::Row => {
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::North,
                            !prev_is_zone,
                        ));
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::South,
                            i != child_count - 1,
                        ));
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::East,
                            false,
                        ));
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::West,
                            false,
                        ));
                    }
                    FlexDirection::Column => {
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::West,
                            !prev_is_zone,
                        ));
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::East,
                            i != child_count - 1,
                        ));
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::North,
                            false,
                        ));
                        handle_visibility.push((
                            zone.resize_handles,
                            CardinalDirection::South,
                            false,
                        ));
                    }
                    _ => warn!(
                        "Invalid flex_direction detected on sized zone {}",
                        children[i]
                    ),
                }

                prev_is_zone = true;
                zone_children.push(children[i]);
            }

            for i in 0..zone_children.len() {
                let zone = q_sized_zones.get(zone_children[i]).unwrap();
                let Some((prev_dir, prev_handle, next_dir, next_handle)) =
                    (match zone.flex_direction {
                        FlexDirection::Row => (
                            CardinalDirection::North,
                            zone.top_handle,
                            CardinalDirection::South,
                            zone.bottom_handle,
                        )
                            .into(),
                        FlexDirection::Column => (
                            CardinalDirection::West,
                            zone.left_handle,
                            CardinalDirection::East,
                            zone.right_handle,
                        )
                            .into(),
                        _ => None,
                    })
                else {
                    warn!(
                        "Invalid flex_direction detected on sized zone {}",
                        zone_children[i]
                    );
                    continue;
                };

                if i == 0 {
                    handle_visibility.push((zone.resize_handles, prev_dir, false));
                }

                if i == zone_children.len() - 1 {
                    handle_visibility.push((zone.resize_handles, next_dir, false));
                }

                handle_neighbours.push((
                    prev_handle,
                    match i > 0 {
                        true => zone_children[i - 1].into(),
                        false => None,
                    },
                ));

                handle_neighbours.push((
                    next_handle,
                    match i < zone_children.len() - 1 {
                        true => zone_children[i + 1].into(),
                        false => None,
                    },
                ));
            }
        }
    }

    for (handles, direction, visible) in handle_visibility {
        if visible {
            commands
                .entity(handles)
                .add_pseudo_state(PseudoState::Resizable(direction));
        } else {
            commands
                .entity(handles)
                .remove_pseudo_state(PseudoState::Resizable(direction));
        }
    }

    for (handle, neighbour) in handle_neighbours {
        let mut handle = q_resize_handle.get_mut(handle).unwrap();
        handle.neighbour = neighbour;
    }
}

fn update_sized_zone_on_resize(
    q_draggable: Query<(&Draggable, &ResizeHandle, &SizedZoneResizeHandle), Changed<Draggable>>,
    mut q_sized_zone: Query<(&mut SizedZone, &Parent)>,
    q_node: Query<&Node>,
) {
    for (draggable, handle, handle_ref) in &q_draggable {
        if handle_ref.neighbour.is_none() {
            continue;
        }

        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Some(diff) = draggable.diff else {
            continue;
        };

        let current_zone_id = handle_ref.sized_zone;
        let neighbour_zone_id = handle_ref.neighbour.unwrap();
        let Ok((current_zone, parent)) = q_sized_zone.get(current_zone_id) else {
            continue;
        };
        let Ok((neighbour_zone, other_parent)) = q_sized_zone.get(neighbour_zone_id) else {
            continue;
        };

        if parent != other_parent {
            warn!(
                "Failed to resize sized zone: Neighbouring zones have different parents: {} <-> {}",
                parent.get(),
                other_parent.get()
            );
            continue;
        }

        let size_diff = match current_zone.flex_direction {
            FlexDirection::Row => handle.direction().to_size_diff(diff).y,
            FlexDirection::Column => handle.direction().to_size_diff(diff).x,
            _ => 0.,
        };
        if size_diff == 0. {
            continue;
        }

        let Ok(node) = q_node.get(parent.get()) else {
            warn!(
                "Cannot calculate sized zone pixel size: Entity {} has parent without Node!",
                current_zone_id
            );
            continue;
        };

        let total_size = match current_zone.flex_direction {
            FlexDirection::Row => node.size().y,
            FlexDirection::Column => node.size().x,
            _ => 0.,
        };
        if total_size == 0. {
            continue;
        }

        let current_min_size = current_zone.children_size;
        let current_size = (current_zone.size_percent / 100.) * total_size;
        let mut current_new_size = current_size;
        let neighbour_min_size = neighbour_zone.children_size;
        let neighbour_size = (neighbour_zone.size_percent / 100.) * total_size;
        let mut neighbour_new_size = neighbour_size;

        if size_diff < 0. {
            if current_size + size_diff >= current_min_size {
                current_new_size += size_diff;
                neighbour_new_size -= size_diff;
            } else {
                current_new_size = current_min_size;
                neighbour_new_size += current_size - current_min_size;
            }
        } else if size_diff > 0. {
            if neighbour_size - size_diff >= neighbour_min_size {
                neighbour_new_size -= size_diff;
                current_new_size += size_diff;
            } else {
                neighbour_new_size = neighbour_min_size;
                current_new_size += neighbour_size - neighbour_min_size;
            }
        }

        q_sized_zone
            .get_mut(current_zone_id)
            .unwrap()
            .0
            .size_percent = (current_new_size / total_size) * 100.;

        q_sized_zone
            .get_mut(neighbour_zone_id)
            .unwrap()
            .0
            .size_percent = (neighbour_new_size / total_size) * 100.;
    }
}

fn update_sized_zone_style(mut q_sized_zones: Query<(&SizedZone, &mut Style), Changed<SizedZone>>) {
    for (zone, mut style) in &mut q_sized_zones {
        style.flex_direction = zone.flex_direction;
        match zone.flex_direction {
            FlexDirection::Row => {
                style.width = Val::Percent(100.);
                style.height = Val::Percent(zone.size_percent);
            }
            FlexDirection::Column => {
                style.width = Val::Percent(zone.size_percent);
                style.height = Val::Percent(100.);
            }
            _ => (),
        }
    }
}

fn should_fit_sized_zones(
    q_changed_nodes: Query<Entity, (With<SizedZone>, Changed<Node>)>,
    q_removed_zones: RemovedComponents<SizedZone>,
) -> bool {
    q_changed_nodes.iter().count() > 0 || q_removed_zones.len() > 0
}

fn fit_sized_zones_on_window_resize(
    q_children: Query<&Children>,
    q_node: Query<&Node>,
    q_sized_zone_parents: Query<&Parent, With<SizedZone>>,
    q_non_sized: Query<(&Node, &Style), Without<SizedZone>>,
    mut q_sized_zone: Query<(&mut SizedZone, &Node)>,
) {
    let parents: Vec<Entity> = q_sized_zone_parents.iter().fold(
        Vec::with_capacity(q_sized_zone_parents.iter().count()),
        |mut acc, parent| {
            let entity = parent.get();
            if !acc.contains(&entity) {
                acc.push(entity);
            }

            acc
        },
    );

    for parent in parents {
        let Ok(parent_node) = q_node.get(parent) else {
            warn!("Sized zone parent {} doesn't have a Node!", parent);
            continue;
        };

        if parent_node.size() == Vec2::ZERO {
            continue;
        }

        let mut non_sized_size = Vec2::ZERO;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((node, style)) = q_non_sized.get(*child) {
                if style.position_type == PositionType::Relative {
                    non_sized_size += node.size();
                }
            }
        }

        let mut sum_zone_size = Vec2::ZERO;
        for child in q_children.get(parent).unwrap().iter() {
            if let Ok((_, node)) = q_sized_zone.get(*child) {
                sum_zone_size += node.size();
            };
        }

        for child in q_children.get(parent).unwrap().iter() {
            let Ok((mut sized_zone, zone_node)) = q_sized_zone.get_mut(*child) else {
                continue;
            };

            let total_size = match sized_zone.flex_direction {
                FlexDirection::Row => parent_node.size().y,
                FlexDirection::Column => parent_node.size().x,
                _ => 0.,
            };
            let non_sized_size = match sized_zone.flex_direction {
                FlexDirection::Row => non_sized_size.y,
                FlexDirection::Column => non_sized_size.x,
                _ => 0.,
            };
            let sum_zone_size = match sized_zone.flex_direction {
                FlexDirection::Row => sum_zone_size.y,
                FlexDirection::Column => sum_zone_size.x,
                _ => 0.,
            };

            let sized_size = total_size - non_sized_size;

            if total_size == 0. || sum_zone_size == 0. || sized_size <= 0. {
                continue;
            }

            let multiplier = sized_size / sum_zone_size;
            let own_size = match sized_zone.flex_direction {
                FlexDirection::Row => zone_node.size().y,
                FlexDirection::Column => zone_node.size().x,
                _ => 0.,
            };

            sized_zone.size_percent =
                (own_size.max(sized_zone.children_size) / total_size) * 100. * multiplier;
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SizedZoneResizeHandleContainer;

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct SizedZoneResizeHandle {
    pub sized_zone: Entity,
    pub neighbour: Option<Entity>,
}

impl Default for SizedZoneResizeHandle {
    fn default() -> Self {
        Self {
            sized_zone: Entity::PLACEHOLDER,
            neighbour: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct SizedZoneConfig {
    pub size: f32,
    pub min_size: f32,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct SizedZone {
    size_percent: f32,
    min_size: f32,
    children_size: f32,
    flex_direction: FlexDirection,
    resize_handles: Entity,
    top_handle: Entity,
    right_handle: Entity,
    bottom_handle: Entity,
    left_handle: Entity,
}

impl Default for SizedZone {
    fn default() -> Self {
        Self {
            size_percent: Default::default(),
            min_size: MIN_SIZED_ZONE_SIZE,
            children_size: Default::default(),
            flex_direction: Default::default(),
            resize_handles: Entity::PLACEHOLDER,
            top_handle: Entity::PLACEHOLDER,
            right_handle: Entity::PLACEHOLDER,
            bottom_handle: Entity::PLACEHOLDER,
            left_handle: Entity::PLACEHOLDER,
        }
    }
}

impl UiContext for SizedZone {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            SizedZone::RESIZE_HANDLES => Ok(self.resize_handles),
            _ => Err(format!(
                "{} doesn't exist for SizedZone. Possible contexts: {:?}",
                target,
                Vec::from_iter(self.contexts())
            )),
        }
    }

    fn cleared_contexts(&self) -> impl Iterator<Item = &str> + '_ {
        [].into_iter()
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_ {
        [SizedZone::RESIZE_HANDLES].into_iter()
    }
}

impl DefaultTheme for SizedZone {
    fn default_theme() -> Option<Theme<SizedZone>> {
        SizedZone::theme().into()
    }
}

impl SizedZone {
    pub const RESIZE_HANDLES: &'static str = "Label";
    pub const RESIZE_HANDLES_Z_INDEX: i32 = 200;

    pub fn direction(&self) -> FlexDirection {
        self.flex_direction
    }

    pub fn size(&self) -> f32 {
        self.size_percent
    }

    pub fn set_size(&mut self, size: f32) {
        self.size_percent = size.clamp(0., 100.);
    }

    pub fn min_size(&self) -> f32 {
        self.min_size
    }

    pub fn theme() -> Theme<SizedZone> {
        let base_theme = PseudoTheme::deferred(None, SizedZone::primary_style);

        let theme_row = PseudoTheme::deferred(vec![PseudoState::LayoutRow], SizedZone::style_row);
        let theme_row_first = PseudoTheme::deferred(
            vec![PseudoState::LayoutRow, PseudoState::FirstChild],
            SizedZone::style_row_first,
        );
        let theme_row_last = PseudoTheme::deferred(
            vec![PseudoState::LayoutRow, PseudoState::LastChild],
            SizedZone::style_row_last,
        );
        let theme_row_single = PseudoTheme::deferred(
            vec![
                PseudoState::LayoutRow,
                PseudoState::FirstChild,
                PseudoState::LastChild,
                PseudoState::SingleChild,
            ],
            SizedZone::style_row_single,
        );

        let theme_column =
            PseudoTheme::deferred(vec![PseudoState::LayoutColumn], SizedZone::style_column);
        let theme_column_first = PseudoTheme::deferred(
            vec![PseudoState::LayoutColumn, PseudoState::FirstChild],
            SizedZone::style_column_first,
        );
        let theme_column_last = PseudoTheme::deferred(
            vec![PseudoState::LayoutColumn, PseudoState::LastChild],
            SizedZone::style_column_last,
        );
        let theme_column_single = PseudoTheme::deferred(
            vec![
                PseudoState::LayoutColumn,
                PseudoState::FirstChild,
                PseudoState::LastChild,
                PseudoState::SingleChild,
            ],
            SizedZone::style_column_single,
        );

        Theme::new(vec![
            base_theme,
            theme_row,
            theme_row_first,
            theme_row_last,
            theme_row_single,
            theme_column,
            theme_column_first,
            theme_column_last,
            theme_column_single,
        ])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let colors = theme_data.colors();

        style_builder
            .border_color(colors.accent(Accent::OutlineVariant))
            .background_color(colors.surface(Surface::Surface));

        style_builder
            .switch_target(SizedZone::RESIZE_HANDLES)
            .z_index(ZIndex::Global(SizedZone::RESIZE_HANDLES_Z_INDEX));
    }

    fn style_row(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        style_builder.border(UiRect::vertical(Val::Px(
            theme_data.spacing.borders.extra_small,
        )));
    }
    fn style_row_first(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        style_builder.border(UiRect::bottom(Val::Px(
            theme_data.spacing.borders.extra_small,
        )));
    }
    fn style_row_last(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        style_builder.border(UiRect::top(Val::Px(theme_data.spacing.borders.extra_small)));
    }
    fn style_row_single(style_builder: &mut StyleBuilder, _: &ThemeData) {
        style_builder.border(UiRect::all(Val::Auto));
    }

    fn style_column(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        style_builder.border(UiRect::horizontal(Val::Px(
            theme_data.spacing.borders.extra_small,
        )));
    }
    fn style_column_first(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        style_builder.border(UiRect::right(Val::Px(
            theme_data.spacing.borders.extra_small,
        )));
    }
    fn style_column_last(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        style_builder.border(UiRect::left(Val::Px(
            theme_data.spacing.borders.extra_small,
        )));
    }
    fn style_column_single(style_builder: &mut StyleBuilder, _: &ThemeData) {
        style_builder.border(UiRect::all(Val::Auto));
    }

    fn frame() -> impl Bundle {
        (
            Name::new("Sized Zone"),
            NodeBundle::default(),
            PseudoStates::new(),
            FlexDirectionToPseudoState,
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::Width,
                LockableStyleAttribute::Height,
            ]),
        )
    }
}

pub trait UiSizedZoneExt {
    fn sized_zone(
        &mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiSizedZoneExt for UiBuilder<'_, Entity> {
    /// A sized zone, that can be resized by dragging its edge handle.
    /// Nested sized zones automatically change layout direction to fit the resizing axis.
    ///
    /// ### PseudoState usage
    /// - `PseudoState::LayoutRow` and `PseudoState::LayoutColumn` are added automatically
    /// - `PseudoState::FirstChild`, `PseudoState::LastChild`, `PseudoState::NthChild(i)`,
    /// `PseudoState::SingleChild`, `PseudoState::EvenChild`, and `PseudoState::OddChild`
    /// are added automatically
    /// - `PseudoState::Resizable(_)` is used transiently to configure the zone resize handles.
    fn sized_zone(
        &mut self,
        config: SizedZoneConfig,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        let size = config.size.clamp(0., 100.);
        let min_size = config.min_size.max(MIN_SIZED_ZONE_SIZE);
        let mut sized_zone = SizedZone {
            size_percent: size,
            min_size,
            ..Default::default()
        };

        let mut frame = self.container(SizedZone::frame(), |container| {
            let zone_id = container.id();
            spawn_children(container);

            let handle = SizedZoneResizeHandle {
                sized_zone: zone_id,
                ..default()
            };
            sized_zone.resize_handles = container
                .resize_handles(handle, |handles| {
                    sized_zone.top_handle = handles.context().handle_north;
                    sized_zone.right_handle = handles.context().handle_east;
                    sized_zone.bottom_handle = handles.context().handle_south;
                    sized_zone.left_handle = handles.context().handle_west;
                })
                .insert(SizedZoneResizeHandleContainer)
                .id();
        });

        frame.insert(sized_zone);

        frame
    }
}

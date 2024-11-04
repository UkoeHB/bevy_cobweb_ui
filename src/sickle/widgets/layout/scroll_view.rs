use bevy::{
    input::mouse::MouseScrollUnit,
    prelude::*,
    ui::{FocusPolicy, RelativeCursorPosition},
};

use sickle_ui_scaffold::prelude::*;

use super::container::UiContainerExt;

// TODO: implement click-to-scroll
// TODO: Fix horizontal scroll performance issue
pub struct ScrollViewPlugin;

impl Plugin for ScrollViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<ScrollView>::default())
            .add_systems(
                Update,
                (
                    update_scroll_view_on_tracked_style_state_change,
                    update_scroll_view_on_content_change,
                    update_scroll_view_on_scroll.after(ScrollableUpdate),
                    update_scroll_view_on_drag.after(DraggableUpdate),
                    update_scroll_view_offset.in_set(ScrollViewOffsetUpdate),
                    update_scroll_view_layout.in_set(ScrollViewLayoutUpdate),
                )
                    .chain(),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ScrollViewOffsetUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct ScrollViewLayoutUpdate;

fn update_scroll_view_on_tracked_style_state_change(
    mut q_scroll_views: Query<(&mut ScrollView, &TrackedStyleState), Changed<TrackedStyleState>>,
) {
    for (mut scroll_view, state) in &mut q_scroll_views {
        let should_disable =
            *state == TrackedStyleState::Enter || *state == TrackedStyleState::Transitioning;

        if scroll_view.disabled != should_disable {
            scroll_view.disabled = should_disable;
        }
    }
}

fn update_scroll_view_on_content_change(
    q_content: Query<&ScrollViewContent, Changed<Node>>,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for content in &q_content {
        let Ok(mut container) = q_scroll_view.get_mut(content.scroll_view) else {
            continue;
        };

        // Touch for change
        container.scroll_offset = container.scroll_offset;
    }
}

fn update_scroll_view_on_scroll(
    q_scrollables: Query<
        (AnyOf<(&ScrollViewViewport, &ScrollBarHandle)>, &Scrollable),
        Changed<Scrollable>,
    >,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for ((viewport, handle), scrollable) in &q_scrollables {
        let Some((axis, diff, unit)) = scrollable.last_change() else {
            continue;
        };

        let scroll_container_id = if let Some(viewport) = viewport {
            viewport.scroll_view
        } else if let Some(handle) = handle {
            handle.scroll_view
        } else {
            continue;
        };

        let Ok(mut scroll_view) = q_scroll_view.get_mut(scroll_container_id) else {
            continue;
        };

        if scroll_view.disabled {
            continue;
        }

        let offset = match axis {
            ScrollAxis::Horizontal => Vec2 { x: diff, y: 0. },
            ScrollAxis::Vertical => Vec2 { x: 0., y: diff },
        };
        let diff = match unit {
            MouseScrollUnit::Line => offset * 20.,
            MouseScrollUnit::Pixel => offset,
        };
        scroll_view.scroll_offset = scroll_view.scroll_offset + diff;
    }
}

// TODO: replace dragging logic to make scroll bar to treat the drag start as a threshold
// TODO: add drag bar container click-to-scroll and click-to-drag
fn update_scroll_view_on_drag(
    q_draggable: Query<(Entity, &Draggable, &ScrollBarHandle), Changed<Draggable>>,
    q_node: Query<&Node>,
    mut q_scroll_view: Query<&mut ScrollView>,
) {
    for (entity, draggable, bar_handle) in &q_draggable {
        if draggable.state == DragState::Inactive
            || draggable.state == DragState::MaybeDragged
            || draggable.state == DragState::DragCanceled
        {
            continue;
        }

        let Ok(mut scroll_view) = q_scroll_view.get_mut(bar_handle.scroll_view) else {
            continue;
        };
        if scroll_view.disabled {
            continue;
        }

        let Some(diff) = draggable.diff else {
            continue;
        };
        let Ok(bar_node) = q_node.get(entity) else {
            continue;
        };
        let Ok(content_node) = q_node.get(scroll_view.content_container) else {
            continue;
        };
        let Ok(container_node) = q_node.get(bar_handle.scroll_view) else {
            continue;
        };

        let container_size = match bar_handle.axis {
            ScrollAxis::Horizontal => container_node.unrounded_size().x,
            ScrollAxis::Vertical => container_node.unrounded_size().y,
        };
        let content_size = match bar_handle.axis {
            ScrollAxis::Horizontal => content_node.unrounded_size().x,
            ScrollAxis::Vertical => content_node.unrounded_size().y,
        };
        let overflow = content_size - container_size;
        if overflow <= 0. {
            continue;
        }

        let bar_size = match bar_handle.axis {
            ScrollAxis::Horizontal => bar_node.unrounded_size().x,
            ScrollAxis::Vertical => bar_node.unrounded_size().y,
        };
        let remaining_space = container_size - bar_size;
        let ratio = overflow / remaining_space;
        let diff = match bar_handle.axis {
            ScrollAxis::Horizontal => diff.x,
            ScrollAxis::Vertical => diff.y,
        } * ratio;

        scroll_view.scroll_offset += match bar_handle.axis {
            ScrollAxis::Horizontal => Vec2 { x: diff, y: 0. },
            ScrollAxis::Vertical => Vec2 { x: 0., y: diff },
        };
    }
}

fn update_scroll_view_offset(
    mut q_scroll_views: Query<(&Node, &mut ScrollView), Or<(Changed<ScrollView>, Changed<Node>)>>,
    q_node: Query<&Node>,
) {
    for (container_node, mut scroll_view) in &mut q_scroll_views {
        let container_width = container_node.unrounded_size().x;
        let container_height = container_node.unrounded_size().y;
        if container_width == 0. || container_height == 0. {
            continue;
        }

        // Unsafe unwrap: if a ScrollView's content doesn't have a Node we panic!
        let content_node = q_node.get(scroll_view.content_container).unwrap();
        let content_width = content_node.unrounded_size().x;
        let content_height = content_node.unrounded_size().y;

        let overflow_x = content_width - container_width;
        let scroll_offset_x = if overflow_x > 0. {
            scroll_view.scroll_offset.x.clamp(0., overflow_x)
        } else {
            scroll_view.scroll_offset.x
        };

        let overflow_y = content_height - container_height;
        let scroll_offset_y = if overflow_y > 0. {
            scroll_view.scroll_offset.y.clamp(0., overflow_y)
        } else {
            scroll_view.scroll_offset.y
        };

        scroll_view.scroll_offset = Vec2 {
            x: scroll_offset_x,
            y: scroll_offset_y,
        };

        scroll_view.overflow = Vec2 {
            x: overflow_x,
            y: overflow_y,
        };

        scroll_view.visible_ratio = Vec2 {
            x: (container_width / content_width).clamp(0., 1.),
            y: (container_height / content_height).clamp(0., 1.),
        };
    }
}

fn update_scroll_view_layout(
    q_scroll_view: Query<(Entity, &ScrollView), Or<(Changed<ScrollView>, Changed<Node>)>>,
    q_node: Query<&Node>,
    mut commands: Commands,
) {
    for (entity, scroll_view) in &q_scroll_view {
        if scroll_view.disabled {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::Disabled)
                .remove_pseudo_state(PseudoState::OverflowX)
                .remove_pseudo_state(PseudoState::OverflowY);

            continue;
        } else {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Disabled);
        }

        // Update content scroll
        let scroll_offset_x = scroll_view.scroll_offset.x;
        let scroll_offset_y = scroll_view.scroll_offset.y;
        let overflow_x = scroll_view.overflow.x;
        let overflow_y = scroll_view.overflow.y;

        if overflow_x > 0. {
            commands
                .style_unchecked(scroll_view.content_container)
                .left(Val::Px(-scroll_offset_x));
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::OverflowX);

            // Unsafe unwrap: Scroll view scroll bars must have a Node
            let bar_container_node = q_node.get(scroll_view.horizontal_scroll_bar).unwrap();
            let bar_container_width = bar_container_node.unrounded_size().x;

            let bar_width = (scroll_view.visible_ratio.x * bar_container_width)
                .clamp(5., bar_container_width.max(5.));
            let remaining_space = bar_container_width - bar_width;
            let bar_offset = (scroll_offset_x / overflow_x) * remaining_space;
            commands
                .style_unchecked(scroll_view.horizontal_scroll_bar_handle)
                .width(Val::Px(bar_width))
                .left(Val::Px(bar_offset));
        } else {
            commands
                .style_unchecked(scroll_view.content_container)
                .left(Val::Px(0.));
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::OverflowX);
        }

        if overflow_y > 0. {
            commands
                .style_unchecked(scroll_view.content_container)
                .top(Val::Px(-scroll_offset_y));
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::OverflowY);

            // Unsafe unwrap: Scroll view scroll bars must have a Node
            let bar_container_node = q_node.get(scroll_view.vertical_scroll_bar).unwrap();
            let bar_container_height = bar_container_node.unrounded_size().y;

            let bar_height = (scroll_view.visible_ratio.y * bar_container_height)
                .clamp(5., bar_container_height.max(5.));
            let remaining_space = bar_container_height - bar_height;
            let bar_offset = (scroll_offset_y / overflow_y) * remaining_space;
            commands
                .style_unchecked(scroll_view.vertical_scroll_bar_handle)
                .height(Val::Px(bar_height))
                .top(Val::Px(bar_offset));
        } else {
            commands
                .style_unchecked(scroll_view.content_container)
                .top(Val::Px(0.));
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::OverflowY);
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBarHandle {
    axis: ScrollAxis,
    scroll_view: Entity,
}

impl Default for ScrollBarHandle {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollBar {
    axis: ScrollAxis,
    scroll_view: Entity,
    handle: Entity,
}

impl Default for ScrollBar {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            scroll_view: Entity::PLACEHOLDER,
            handle: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollViewContent {
    scroll_view: Entity,
}

impl Default for ScrollViewContent {
    fn default() -> Self {
        Self {
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

impl ScrollViewContent {
    pub fn scroll_view(&self) -> Entity {
        self.scroll_view
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollViewViewport {
    scroll_view: Entity,
}

impl Default for ScrollViewViewport {
    fn default() -> Self {
        Self {
            scroll_view: Entity::PLACEHOLDER,
        }
    }
}

impl ScrollViewViewport {
    pub fn scroll_view(&self) -> Entity {
        self.scroll_view
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollView {
    viewport: Entity,
    content_container: Entity,
    horizontal_scroll_bar: Entity,
    horizontal_scroll_bar_handle: Entity,
    vertical_scroll_bar: Entity,
    vertical_scroll_bar_handle: Entity,
    /// Can be used to control the scroll offset of the scroll view
    ///
    /// If updated before `ScrollViewOffsetUpdate` the change will be
    /// made in the same frame. It is clamped to the scroll overflow,
    /// so it is safe to set it to any value.
    pub scroll_offset: Vec2,
    overflow: Vec2,
    visible_ratio: Vec2,
    restricted_to: Option<ScrollAxis>,
    pub disabled: bool,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            viewport: Entity::PLACEHOLDER,
            content_container: Entity::PLACEHOLDER,
            horizontal_scroll_bar: Entity::PLACEHOLDER,
            horizontal_scroll_bar_handle: Entity::PLACEHOLDER,
            vertical_scroll_bar: Entity::PLACEHOLDER,
            vertical_scroll_bar_handle: Entity::PLACEHOLDER,
            scroll_offset: Vec2::ZERO,
            overflow: Vec2::ZERO,
            visible_ratio: Vec2::ZERO,
            disabled: false,
            restricted_to: None,
        }
    }
}

impl UiContext for ScrollView {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            ScrollView::VIEWPORT => Ok(self.viewport),
            ScrollView::CONTENT_CONTAINER => Ok(self.content_container),
            ScrollView::HORIZONTAL_SCROLL_BAR => Ok(self.horizontal_scroll_bar),
            ScrollView::HORIZONTAL_SCROLL_HANDLE => Ok(self.horizontal_scroll_bar_handle),
            ScrollView::VERTICAL_SCROLL_BAR => Ok(self.vertical_scroll_bar),
            ScrollView::VERTICAL_SCROLL_HANDLE => Ok(self.vertical_scroll_bar_handle),
            _ => Err(format!(
                "{} doesn't exist for ScrollView. Possible contexts: {:?}",
                target,
                Vec::from_iter(self.contexts())
            )),
        }
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_ {
        [
            ScrollView::VIEWPORT,
            ScrollView::CONTENT_CONTAINER,
            ScrollView::HORIZONTAL_SCROLL_BAR,
            ScrollView::HORIZONTAL_SCROLL_HANDLE,
            ScrollView::VERTICAL_SCROLL_BAR,
            ScrollView::VERTICAL_SCROLL_HANDLE,
        ]
        .into_iter()
    }
}

impl DefaultTheme for ScrollView {
    fn default_theme() -> Option<Theme<ScrollView>> {
        ScrollView::theme().into()
    }
}

impl ScrollView {
    pub const VIEWPORT: &'static str = "Viewport";
    pub const CONTENT_CONTAINER: &'static str = "ContentContainer";
    pub const HORIZONTAL_SCROLL_BAR: &'static str = "HorizontalScrollBar";
    pub const HORIZONTAL_SCROLL_HANDLE: &'static str = "HorizontalScrollHandle";
    pub const VERTICAL_SCROLL_BAR: &'static str = "VerticalScrollBar";
    pub const VERTICAL_SCROLL_HANDLE: &'static str = "VerticalScrollHandle";

    /// The viewport that clips the content
    pub fn viewport_id(&self) -> Entity {
        self.viewport
    }

    /// The container of the content that will be scrolled when it overflows the viewport
    pub fn content_container_id(&self) -> Entity {
        self.content_container
    }

    /// The amount of content hidden by the viewport
    /// Update in ScrollViewOffsetUpdate
    pub fn overflow(&self) -> Vec2 {
        self.overflow
    }

    /// The 0-1 ratio of content visible in the viewport
    pub fn visible_ratio(&self) -> Vec2 {
        self.visible_ratio
    }

    pub fn theme() -> Theme<ScrollView> {
        let base_theme = PseudoTheme::deferred_context(None, ScrollView::primary_style);
        let disabled_theme =
            PseudoTheme::deferred(vec![PseudoState::Disabled], ScrollView::disabled_style);
        let overflow_x_theme =
            PseudoTheme::deferred(vec![PseudoState::OverflowX], ScrollView::overflow_x_style);
        let overflow_y_theme =
            PseudoTheme::deferred(vec![PseudoState::OverflowY], ScrollView::overflow_y_style);
        let overflow_xy_theme = PseudoTheme::deferred(
            vec![PseudoState::OverflowX, PseudoState::OverflowY],
            ScrollView::overflow_xy_style,
        );

        Theme::new(vec![
            base_theme,
            disabled_theme,
            overflow_x_theme,
            overflow_y_theme,
            overflow_xy_theme,
        ])
    }

    fn primary_style(
        style_builder: &mut StyleBuilder,
        _scroll_view: &ScrollView,
        theme_data: &ThemeData,
    ) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .width(Val::Percent(100.))
            .height(Val::Percent(100.));

        style_builder
            .switch_target(ScrollView::HORIZONTAL_SCROLL_BAR)
            .bottom(Val::Px(0.))
            .left(Val::Px(0.))
            .right(Val::Px(0.))
            .height(Val::Px(theme_spacing.scroll_bar_size))
            .border(UiRect::top(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.container(Container::SurfaceMid))
            .visibility(Visibility::Hidden);

        style_builder
            .switch_target(ScrollView::VERTICAL_SCROLL_BAR)
            .right(Val::Px(0.))
            .width(Val::Px(theme_spacing.scroll_bar_size))
            .height(Val::Percent(100.))
            .border(UiRect::left(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.container(Container::SurfaceMid))
            .visibility(Visibility::Hidden);

        style_builder
            .switch_target(ScrollView::CONTENT_CONTAINER)
            .padding(UiRect::all(Val::Auto));

        style_builder
            .switch_context(ScrollView::HORIZONTAL_SCROLL_HANDLE, None)
            .border(UiRect::horizontal(Val::Px(
                theme_spacing.borders.extra_small,
            )))
            .border_color(colors.accent(Accent::Shadow))
            .border_radius(BorderRadius::all(Val::Px(theme_spacing.scroll_bar_size)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.accent(Accent::Tertiary).with_alpha(0.5),
                hover: colors.accent(Accent::Tertiary).with_alpha(0.8).into(),
                press: colors.accent(Accent::Tertiary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_context(ScrollView::VERTICAL_SCROLL_HANDLE, None)
            .border(UiRect::vertical(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Shadow))
            .border_radius(BorderRadius::all(Val::Px(theme_spacing.scroll_bar_size)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.accent(Accent::Tertiary).with_alpha(0.5),
                hover: colors.accent(Accent::Tertiary).with_alpha(0.8).into(),
                press: colors.accent(Accent::Tertiary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);
    }

    fn disabled_style(style_builder: &mut StyleBuilder, _theme_data: &ThemeData) {
        style_builder
            .switch_target(ScrollView::HORIZONTAL_SCROLL_BAR)
            .visibility(Visibility::Hidden);
        style_builder
            .switch_target(ScrollView::VERTICAL_SCROLL_BAR)
            .visibility(Visibility::Hidden);
    }

    fn overflow_x_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;

        style_builder
            .switch_target(ScrollView::HORIZONTAL_SCROLL_BAR)
            .visibility(Visibility::Inherited);

        style_builder
            .switch_target(ScrollView::CONTENT_CONTAINER)
            .padding(UiRect::bottom(Val::Px(theme_spacing.scroll_bar_size)));
    }

    fn overflow_y_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;

        style_builder
            .switch_target(ScrollView::VERTICAL_SCROLL_BAR)
            .visibility(Visibility::Inherited);

        style_builder
            .switch_target(ScrollView::CONTENT_CONTAINER)
            .padding(UiRect::right(Val::Px(theme_spacing.scroll_bar_size)));
    }

    fn overflow_xy_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;

        style_builder
            .switch_target(ScrollView::HORIZONTAL_SCROLL_BAR)
            .right(Val::Px(theme_spacing.scroll_bar_size));

        style_builder
            .switch_target(ScrollView::CONTENT_CONTAINER)
            .padding(UiRect::px(
                0.,
                theme_spacing.scroll_bar_size,
                0.,
                theme_spacing.scroll_bar_size,
            ));
    }

    fn frame() -> impl Bundle {
        (
            Name::new("Scroll View"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            LockedStyleAttributes::from_vec(vec![LockableStyleAttribute::FlexDirection]),
        )
    }

    fn viewport(scroll_view: Entity) -> impl Bundle {
        (
            Name::new("Viewport"),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    height: Val::Percent(100.),
                    width: Val::Percent(100.),
                    overflow: Overflow::clip(),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            Interaction::default(),
            Scrollable::default(),
            ScrollViewViewport { scroll_view },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::PositionType,
                LockableStyleAttribute::Width,
                LockableStyleAttribute::Height,
                LockableStyleAttribute::Overflow,
                LockableStyleAttribute::FocusPolicy,
                LockableStyleAttribute::Border,
                LockableStyleAttribute::Padding,
                LockableStyleAttribute::Margin,
            ]),
        )
    }

    fn content(scroll_view: Entity, restrict_to: Option<ScrollAxis>) -> impl Bundle {
        let width = if let Some(axis) = restrict_to {
            match axis {
                ScrollAxis::Horizontal => Val::Auto,
                ScrollAxis::Vertical => Val::Percent(100.),
            }
        } else {
            Val::Auto
        };

        let height = if let Some(axis) = restrict_to {
            match axis {
                ScrollAxis::Horizontal => Val::Percent(100.),
                ScrollAxis::Vertical => Val::Auto,
            }
        } else {
            Val::Auto
        };

        (
            Name::new("Content"),
            NodeBundle {
                style: Style {
                    width,
                    height,
                    min_width: Val::Percent(100.),
                    min_height: Val::Percent(100.),
                    justify_self: JustifySelf::Start,
                    align_self: AlignSelf::Start,
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.,
                    ..default()
                },
                ..default()
            },
            ScrollViewContent { scroll_view },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::PositionType,
                LockableStyleAttribute::MinWidth,
                LockableStyleAttribute::MinHeight,
                LockableStyleAttribute::JustifySelf,
                LockableStyleAttribute::AlignSelf,
                LockableStyleAttribute::FlexDirection,
                LockableStyleAttribute::FlexShrink,
                LockableStyleAttribute::Top,
                LockableStyleAttribute::Left,
            ]),
        )
    }

    fn scroll_bar(axis: ScrollAxis) -> impl Bundle {
        (
            Name::new(match axis {
                ScrollAxis::Horizontal => "Horizontal Scroll Bar",
                ScrollAxis::Vertical => "Vertical Scroll Bar",
            }),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                z_index: ZIndex::Local(1),
                ..default()
            },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::PositionType,
                LockableStyleAttribute::JustifyContent,
                LockableStyleAttribute::FocusPolicy,
                LockableStyleAttribute::ZIndex,
                LockableStyleAttribute::Padding,
                LockableStyleAttribute::Margin,
            ]),
        )
    }

    fn scroll_bar_handle(scroll_view: Entity, axis: ScrollAxis) -> impl Bundle {
        (
            Name::new("Scroll Bar Handle"),
            ButtonBundle {
                style: Style {
                    width: match axis {
                        ScrollAxis::Horizontal => Val::Auto,
                        ScrollAxis::Vertical => Val::Percent(100.),
                    },
                    height: match axis {
                        ScrollAxis::Horizontal => Val::Percent(100.),
                        ScrollAxis::Vertical => Val::Auto,
                    },
                    ..default()
                },
                ..default()
            },
            TrackedInteraction::default(),
            Draggable::default(),
            RelativeCursorPosition::default(),
            Scrollable::default(),
            ScrollBarHandle { axis, scroll_view },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::Width,
                LockableStyleAttribute::Height,
                LockableStyleAttribute::Top,
                LockableStyleAttribute::Left,
            ]),
        )
    }
}

pub trait UiScrollViewExt {
    fn scroll_view(
        &mut self,
        restrict_to: impl Into<Option<ScrollAxis>>,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity>;
}

impl UiScrollViewExt for UiBuilder<'_, Entity> {
    /// A simple scroll view. When the content overflows, scroll bars appear for the given direction.
    /// Can be restricted to scroll only on one axis.
    ///
    /// ### PseudoState usage
    /// - `PseudoState::Disabled` is used when the view is disabled, preventing scroll bars from showing
    /// - `PseudoState::OverflowX` and `PseudoState::OverflowY` is used to indicate which axis the content
    /// overflows on.
    fn scroll_view(
        &mut self,
        restrict_to: impl Into<Option<ScrollAxis>>,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>),
    ) -> UiBuilder<Entity> {
        let restricted_to = restrict_to.into();
        let mut scroll_view = ScrollView {
            restricted_to,
            ..default()
        };

        let mut frame = self.container(ScrollView::frame(), |frame| {
            let scroll_view_id = frame.id();

            scroll_view.viewport = frame
                .container((ScrollView::viewport(scroll_view_id),), |viewport| {
                    scroll_view.content_container = viewport
                        .container(
                            ScrollView::content(scroll_view_id, restricted_to),
                            spawn_children,
                        )
                        .id();
                })
                .id();

            scroll_view.horizontal_scroll_bar = frame
                .container(
                    ScrollView::scroll_bar(ScrollAxis::Horizontal),
                    |scroll_bar| {
                        scroll_view.horizontal_scroll_bar_handle = scroll_bar
                            .spawn((ScrollView::scroll_bar_handle(
                                scroll_view_id,
                                ScrollAxis::Horizontal,
                            ),))
                            .id();
                    },
                )
                .insert(ScrollBar {
                    axis: ScrollAxis::Horizontal,
                    scroll_view: scroll_view_id,
                    handle: scroll_view.horizontal_scroll_bar_handle,
                })
                .id();

            scroll_view.vertical_scroll_bar = frame
                .container(ScrollView::scroll_bar(ScrollAxis::Vertical), |scroll_bar| {
                    scroll_view.vertical_scroll_bar_handle = scroll_bar
                        .spawn((ScrollView::scroll_bar_handle(
                            scroll_view_id,
                            ScrollAxis::Vertical,
                        ),))
                        .id();
                })
                .insert(ScrollBar {
                    axis: ScrollAxis::Vertical,
                    scroll_view: scroll_view_id,
                    handle: scroll_view.vertical_scroll_bar_handle,
                })
                .id();
        });

        frame.insert(scroll_view);

        frame
    }
}

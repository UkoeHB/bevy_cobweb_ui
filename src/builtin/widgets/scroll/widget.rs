/*
unimplemented
- touch-based scrolling; currently only directly dragging the scrollbar works
    - TouchEvent
        - need to track touch id lifetime
            - record initial scroll position when touch starts
        - scroll = distance traveled
        - how to block touch events when elements are pressed in view? and likewise, how to cancel presses on elements when
        scrolling?
- mobile kinematic scrolling w/ buffers at top/bottom
    - https://stackoverflow.com/a/7224899
- macos-style 'jump one page on scrollbar press'
    - needs animation framework overhaul or bespoke solution
        - bespoke solution likely best: need to also support pagination via mouse scroll events and gamepad/controller inputs
    - The 'animate to next page' setting does the following (if you press on the bar and not on the slider handle)
        1. On press, animate one page in the direction of the cursor.
        2. Delay
        3. Rapidly animate pages toward the cursor at a fixed velocity. If the cursor moves above or below the handle,
        then the movement may be reversed.
        4. When the handle reaches the cursor, or when the cursor is released/canceled, the page movement stops - but the
        final page animation runs to completion (so you always end on a page boundary). Page boundaries are calculated based
        on the view position when you first press the bar (so `original position + n * view size`).
- automatic wheel-scroll-line-size calculation using font sizes in the scroll view
    - current solution is hard-coded line size
- gamepad/game controller support
    - need to research expected behavior
- robust framework for deciding when to receive scroll events vs when not to

unsolved problems
- we set the scrollbar handle's size before layout, which means if the scroll area or view area sizes change, it won't
be reflected in handle size until the next tick (an off-by-1 glitch)
    - Solving this requires more flexibility from bevy's layout system. ComputedNode is not outwardly mutable, and
    ContentSize doesn't give enough information about other nodes. There is no way post-layout to adjust the handle size.
- if content size changes, we want the scroll view to 'stay in place' pointing at the same spot on the content
    - How to figure out if content size increased above or below the view?
*/

use bevy::prelude::*;

use crate::prelude::*;
use crate::builtin::widgets::slider::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_interactions_hack(world: &mut World)
{
    world.syscall((), bevy::picking::focus::update_interactions);
}

//-------------------------------------------------------------------------------------------------------------------

fn cleanup_dead_bases(mut c: Commands, dying: Query<Entity, With<ScrollBaseDying>>)
{
    // If any scroll base is dead, then remove it and reapply its contents.
    // - We know if a base is 'dying' here then it's actually dead, because the only time a 'dying' flag can
    // be unset is immediately after it is set (i.e. ScrollBase instruction reverted -> ScrollBase instruction
    // re-applied).
    for entity in dying.iter() {
        c.entity(entity)
            .queue(RemoveDeadScrollBase)
            .remove::<ScrollBaseDying>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// This is separate from updating scroll position because we add/remove states based on whether a given axis
/// has scrollable content.
// TODO: move this to after layout if bevy's ComputedNode struct's fields become public, so we can edit the handle
// size directly and escape 1-tick-delay glitchiness
// TODO: consider setting slider value to zero when content shrinks smaller than the area
fn refresh_scroll_handles(
    mut c: Commands,
    ps: PseudoStateParam,
    mut iter_children: IterChildren,
    parents: Query<&Parent>,
    children: Query<&Children>,
    ui_surface: Res<UiSurface>,
    bases: Query<(Entity, &ComputedScrollBase, &Node, &ViewVisibility)>,
    areas: Query<(Entity, &ScrollArea, &ComputedNode)>,
    mut bar_handles: Query<&mut Node, (With<SliderHandle>, With<ScrollHandle>)>,
)
{
    for (area_entity, scroll_area, area_node) in areas.iter() {
        // Get area size.
        let area_size = area_node.size();

        // Get area content size.
        // - Skipping here helps avoid initialization glitchiness where handle sizes are wrong because
        // we don't know content size yet. Any handle that has no minimum size will be zero-sized, and appear to just
        // 'pop into place' once content size is available. Handles with a minimum size will still have glitchiness.
        let Some(content_size) = ui_surface
            .get_layout(area_entity).map(|l| Vec2::new(l.content_size.width, l.content_size.height))
        else {
            continue
        };

        // Look up base.
        // - Note: base and area can be the same entity.
        let mut current = area_entity;
        let res = loop {
            if let Ok(res) = bases.get_mut(current) {
                break Some(res);
            }

            let Some(parent) = parents.get(current) else { break None };
            current = *parent;
        };
        let Some((base_entity, computed_base, base_node, _base_visibility)) = res else { continue };

        // Skip if base is not visible.
        // - ViewVisibility is updated after TransformPropagate, so we can't use it until this system moves there.
        // if base_node.display == Display::None || !base_visibility.get() {
        //     continue;
        // }
        if base_node.display == Display::None {
            continue;
        }

        // Update handle sizes.
        if let Some(horizontal) = computed_base.horizontal {
            // Look up scrollbar's handle.
            if let Some(bar_children) = children.get(horizontal) {
                if let Some(mut handle_node) = iter_children.search_descendants(bar_children, &children, |entity|
                    bar_handles.get_mut(entity)
                ) {
                    let new_width = if content_size.x > 0.0 {
                        area_size.x / content_size.x
                    else {
                        1.0
                    };
                    let new_width = (new_width * 100.0).clamp(0.0, 100.0);

                    // We try add/remove these every tick to make sure they are correct, especially on init.
                    if new_width == 100.0 {
                        ps.try_remove(base_entity, &mut c, HORIZONTAL_SCROLL_PSEUDO_STATE.clone());
                    } else {
                        ps.try_insert(base_entity, &mut c, HORIZONTAL_SCROLL_PSEUDO_STATE.clone());
                    }

                    let new_width = Val::Percent(new_width);
                    if handle_node.width != new_width {
                        handle_node.width = new_width;
                    }
                }
            }
        }
        if let Some(vertical) = computed_base.vertical {
            // Look up scrollbar's handle.
            if let Some(bar_children) = children.get(vertical) {
                if let Some(mut handle_node) = iter_children.search_descendants(bar_children, &children, |entity|
                    bar_handles.get_mut(entity)
                ) {
                    let new_height = if content_size.y > 0.0 {
                        area_size.y / content_size.y
                    else {
                        1.0
                    };
                    let new_height = (new_height * 100.0).clamp(0.0, 100.0);

                    // We try add/remove these every tick to make sure they are correct, especially on init.
                    if new_height == 100.0 {
                        ps.try_remove(base_entity, &mut c, VERTICAL_SCROLL_PSEUDO_STATE.clone());
                    } else {
                        ps.try_insert(base_entity, &mut c, VERTICAL_SCROLL_PSEUDO_STATE.clone());
                    }

                    let new_height = Val::Percent(new_height);
                    if handle_node.height != new_height {
                        handle_node.height = new_height;
                    }
                }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Consumes scroll delta in one direction.
///
/// Also dispatches `MouseScroll` entity events.
fn consume_scroll_delta(
    c: &mut Commands,
    slider_vals: &mut ReactiveMut<SliderValue>,
    entity: Entity,
    correction_factor: f32,
    scroll_size: f32,
    unconsumed_delta: &mut f32,
)
{
    if unconsumed_delta == 0.0 || scroll_size <= 0.0 { return }
    let Ok(val) = slider_vals.get(entity).and_then(|val| val.single()) else { return };

    if unconsumed_delta > 0.0 && val < 1.0 {
        let available = (1. - val) * scroll_size;

        let val_mut = slider_vals.get_mut(entity, c).unwrap();

        if available >= unconsumed_delta * correction_factor {
            let remaining = available - unconsumed_delta * correction_factor;
            *val_mut = SliderValue::Single(1. - (remaining / scroll_size));
            *val_mut.normalize();
            *unconsumed_delta = 0.;
        } else {
            *val_mut = SliderValue::Single(1.);
            let consumed = if correction_factor != 1.0 {
                available / correction_factor
            } else {
                available
            };
            *unconsumed_delta -= consumed;
        }
    } else if unconsumed_delta < 0.0 && val > 0.0 {
        let available = val * scroll_size;

        let val_mut = slider_vals.get_mut(entity).unwrap();

        if available >= -unconsumed_delta * correction_factor {
            let remaining = available + unconsumed_delta * correction_factor;
            *val_mut = SliderValue::Single(remaining / scroll_size);
            *val_mut.normalize();
            *unconsumed_delta = 0.;
        } else {
            *val_mut = SliderValue::Single(0.);
            let consumed = if correction_factor != 1.0 {
                available / correction_factor
            } else {
                available
            };
            *unconsumed_delta += consumed;
        }
    }

    c.react().entity_event(entity, MouseScroll);
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_mouse_scroll_impl(
    c: &mut Commands,
    iter_children: &mut IterChildren,
    ui_surface: &UiSurface,
    children: &Query<&Children>,
    bases: &Query<(Entity, &ScrollBase, &ComputedScrollBase)>,
    areas: &Query<(Entity, &ComputedNode), With<ScrollArea>>,
    slider_vals: &mut ReactiveMut<SliderValue>,
    hit_entity: Entity,
    mouse_scroll_unit: MouseScrollUnit,
    unconsumed_delta: &mut Vec2,
)
{
    let Ok((base_entity, scroll_base, computed_base)) = bases.get(hit_entity) else { return };

    // Look up scroll area.
    let Some((area_entity, area_node)) = iter_children.search(base_entity, children, |entity| areas.get(entity)) else { return };
    let area_size = area_node.size();

    // Get content size.
    let Some(content_size) = ui_surface
        .get_layout(area_entity).map(|l| Vec2::new(l.content_size.width, l.content_size.height))
    else {
        return;
    };

    let scroll_size = (content_size - area_size).max(Vec2::default());

    let correction_factor = match mouse_scroll_unit {
        MouseScrollUnit::Pixel => 1.0,
        MouseScrollUnit::Line => scroll_base.line_size.max(1.0),
    };

    // Consume scroll delta and dispatch MouseScroll events to scrollbars.
    if let Some(horizontal) = computed_base.horizontal {
        consume_scroll_delta(
            c,
            slider_vals,
            horizontal,
            correction_factor,
            scroll_size.x,
            &mut unconsumed_delta.x
        );
    }
    if let Some(vertical) = computed_base.vertical {
        consume_scroll_delta(
            c,
            slider_vals,
            vertical,
            correction_factor,
            scroll_size.y,
            &mut unconsumed_delta.y
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_mouse_scroll(
    mut c: Commands,
    mut iter_children: IterChildren,
    ui_surface: Res<UiSurface>,
    children: Query<&Children>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    pointers: Query<(&PointerId, &PointerInteraction)>,
    bases: Query<(Entity, &ScrollBase, &ComputedScrollBase)>,
    areas: Query<(Entity, &ComputedNode), With<ScrollArea>>,
    mut slider_vals: ReactiveMut<SliderValue>,
    focus_policies: Query<&FocusPolicy>,
)
{
    // Find mouse pointer.
    // - We assume there's only one of these.
    let Some((ptr_id, ptr_interaction)) = pointers.iter().find(|(id, _)| *id == PointerId::Mouse) else { return };

    // Apply scroll delta to entities under the cursor.
    // - If an entity in the stack has blocked picking or FocusPolicy::Block, then scroll delta won't 'spill over'
    // to lower scroll areas.
    let mut unconsumed_delta = mouse_scroll.delta;

    for (hit_entity, _) in ptr_interaction.iter() {
        if unconsumed_delta == Vec2::default() {
            return;
        }

        // Apply scroll delta to scroll areas.
        apply_mouse_scroll_impl(
            &mut c,
            &mut iter_children,
            &ui_surface,
            &children,
            &bases,
            &areas,
            &mut slider_vals,
            hit_entity,
            mouse_scroll.unit,
            &mut unconsumed_delta
        );

        // Terminate on blocked entities.
        if let Some(focus_policy) = focus_policies.get(hit_entity) {
            if *focus_policy == FocusPolicy::Block {
                return;
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn refresh_scroll_position(
    ui_surface: Res<UiSurface>,
    bases: Query<(&mut ScrollPosition, &ComputedScrollBase)>,
    areas: Query<(Entity, &ScrollArea, &Node)>,
    parents: Query<&Parent>,
    slider_vals: Reactive<SliderValue>,
)
{
    for (area_entity, scroll_area, area_node) in areas.iter() {
        // Get area size.
        let area_size = area_node.size();

        // Get area content size.
        let Some(content_size) = ui_surface
            .get_layout(area_entity).map(|l| Vec2::new(l.content_size.width, l.content_size.height))
        else {
            continue
        };

        let scroll_size = (content_size - area_size).max(Vec2::default());

        // Look up base.
        // - Note: base and area can be the same entity.
        let mut current = area_entity;
        let res = loop {
            if let Ok((scroll_pos, computed_base)) = bases.get_mut(current) {
                break Some((scroll_pos, computed_base));
            }

            let Some(parent) = parents.get(current) else { break None };
            current = *parent;
        };
        let Some((mut scroll_pos, computed_base)) = res else { continue };

        // Update scroll position.
        if let Some(horizontal) = computed_base.horizontal {
            let mut slider_val = slider_vals.get(horizontal).copied().unwrap_or_default();
            slider_val.normalize();
            let val = slider_val.single().unwrap_or_default();
            let computed_x_offset = val * scroll_size.x;

            if scroll_pos.offset_x != computed_x_offset {
                scroll_pos.offset_x = computed_x_offset;
            }
        }
        if let Some(vertical) = computed_base.vertical {
            let mut slider_val = slider_vals.get(vertical).copied().unwrap_or_default();
            slider_val.normalize();
            let val = slider_val.single().unwrap_or_default();
            let computed_y_offset = val * scroll_size.y;

            if scroll_pos.offset_y != computed_y_offset {
                scroll_pos.offset_y = computed_y_offset;
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for cleaning up dead scrollbases after a hot reload removes ScrollBase from a node.
#[derive(Component)]
struct ScrollBaseDying;

//-------------------------------------------------------------------------------------------------------------------

/// Removes a dead `ComputedScrollBase` and reapplies all its scrollbars so they can be relocated to another
/// control map if possible.
struct RemoveDeadScrollBase;

impl EntityCommand for RemoveDeadScrollBase
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(old_scroll_base) = world
            .get_entity(entity)
            .ok()
            .and_then(|mut emut| emut.take::<ComputedScrollBase>())
        else {
            return;
        };

        old_scroll_base.reapply_bars(world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Tracks scrollbar entities associated with a slider widget.
#[derive(Component, Default, Copy, Clone, Debug)]
struct ComputedScrollBase
{
    horizontal: Option<Entity>,
    vertical: Option<Entity>,

    /// Tracks scroll bars that are 'redundant' because we already have a horizontal or vertical bar. Used to
    /// repair bar mappings on hot reload.
    dangling: Vec<Entity>
}

impl ComputedScrollBase
{
    fn add_bar(&mut self, entity: Entity, axis: SliderAxis)
    {
        match axis {
            SliderAxis::X => {
                if let Some(prev) = self.horizontal.as_ref() {
                    if prev != entity {
                        tracing::warn!("failed adding horizontal scroll bar {entity:?} to nearest scroll base; there \
                            is already a horizontal scroll bar {prev:?}");
                        self.dangling.push(entity);
                    }
                } else {
                    self.horizontal = Some(entity);
                }
            }
            SliderAxis::Y => {
                if let Some(prev) = self.vertical.as_ref() {
                    if prev != entity {
                        tracing::warn!("failed adding vertical scroll bar {entity:?} to nearest scroll base; there \
                            is already a vertical scroll bar {prev:?}");
                        self.dangling.push(entity);
                    }
                } else {
                    self.vertical = Some(entity);
                }
            }
            SliderAxis::Planar => {
                tracing::warn!("failed adding scroll bar {entity:?} to nearest scroll base; scroll bar has SliderAxis::Planar \
                    but only X and Y axes are supported");
                self.dangling.push(entity);
            }
        }
    }

    fn reapply_bars(self, world: &mut World)
    {
        if let Some(horizontal) = self.horizontal {
            if let Ok(bar) = world.get::<ScrollBar>(horizontal) {
                bar.clone().apply(horizontal, world);
            }
        }

        if let Some(vertical) = self.vertical {
            if let Ok(bar) = world.get::<ScrollBar>(vertical) {
                bar.clone().apply(vertical, world);
            }
        }

        for dangling in self.dangling {
            if let Ok(bar) = world.get::<ScrollBar>(dangling) {
                bar.clone().apply(dangling, world);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Pseudo state added to a scroll base when its scroll area has horizontally-scrollable content.
///
/// It can be used in COB as `Custom("HorizontalScroll")`.
pub const HORIZONTAL_SCROLL_PSEUDO_STATE: PseudoState = PseudoState::Custom(SmolStr::new_static("HorizontalScroll"));

//-------------------------------------------------------------------------------------------------------------------

/// Pseudo state added to a scroll base when its scroll area has vertically-scrollable content.
///
/// It can be used in COB as `Custom("VerticalScroll")`.
pub const VERTICAL_SCROLL_PSEUDO_STATE: PseudoState = PseudoState::Custom(SmolStr::new_static("VerticalScroll"));

//-------------------------------------------------------------------------------------------------------------------

/// Loadable that sets up the base of a scroll area widget.
///
/// A scroll area widget is composed of a [`ScrollBase`], a [`ScrollArea`] (where content goes), and one or two
/// [`ScrollBars`](ScrollBar) (which each have a [`ScrollHandle`]).
///
/// There are two broad categories of scroll area widgets:
/// 1. Scrollbars overlay on top of scroll content. You can use absolute positioning like this:
/**
```rust
"base"
    ScrollBase
    ScrollArea
    FlexNode{clipping:ScrollXY width:500px height:700px flex_direction:Column}

    "shim"
        AbsoluteNode{width:100% height:100% flex_direction:Column justify_cross:FlexEnd}

        "vertical"
            ScrollBar{axis:Y}
            FlexNode{flex_grow:1 width:10px}

            "handle"
                ScrollHandle
                FlexNode{width:100%} // Height controlled automatically

        "horizontal"
            ScrollBar{axis:X}
            FlexNode{width:100% height:10px}

            "handle"
                ScrollHandle
                FlexNode{height:100%} // Width controlled automatically
```
*/
/// 2. Scrollbars are separated from scroll content. You can use flex-positioning like this:
/**
```rust
"base"
    ScrollBase
    FlexNode{width:500px height:700px flex_direction:Column}

    "shim"
        FlexNode{width:100% flex_grow:1 flex_direction:Row}

        "area"
            ScrollArea
            FlexNode{clipping:ScrollXY height:100% flex_grow:1 flex_direction:Column}

            // Scroll content goes here

        "vertical"
            ScrollBar{axis:Y}
            FlexNode{height:100% width:10px}

            "handle"
                ScrollHandle
                FlexNode{width:100%} // Height controlled automatically

    "horizontal"
        ScrollBar{axis:X}
        FlexNode{width:100% height:10px}

        "handle"
            ScrollHandle
            FlexNode{height:100%} // Width controlled automatically
```
*/
#[derive(Reflect, Component, PartialEq, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
)]
pub struct ScrollBase
{
    /// Size of lines for mouse scrolling.
    ///
    /// Defaults to 16 pixels.
    // TODO: replace this with line size inference?
    #[reflect(default = "ScrollBase::default_line_size")]
    pub line_size: f32,
}

impl ScrollBase
{
    fn default_line_size() -> f32
    {
        16.0
    }
}

impl Instruction for ScrollBase
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

        // Add base.
        emut.insert(self);

        // Add computed scroll base if missing.
        if let Some(mut computed_base) = emut.get_mut::<ComputedScrollBase>() {
            // We are not actually dying, just refreshing the scroll base, so this can be removed.
            emut.remove::<ScrollBaseDying>();
        } else {
            emut.insert(ComputedScrollBase::default());

            // Cold path when applying to an existing scene.
            #[cfg(feature = "hot_reload")]
            if emut.contains::<Children>() {
                // Look backward for ComputedScrollBase to maybe 'steal' its scroll bars.
                if let Some((_, computed_base)) = get_ancestor_mut::<ComputedScrollBase>(world, entity) {
                    let other_computed_base = std::mem::take(&mut computed_base);
                    other_computed_base.reapply_bars(world);
                }

                // Iterate children (stopping at scroll bases) to identify children with ScrollBar.
                let mut dangling = vec![];
                iter_descendants_filtered(
                    world,
                    entity,
                    |world, entity| world.get::<ComputedScrollBase>(entity).is_none(),
                    |world, entity| {
                        if let Some(bar) = world.get::<ScrollBar>(entity) {
                            dangling.push((entity, bar.clone()));
                        }
                    }
                );

                for (entity, bar) in dangling {
                    bar.apply(entity, world);
                }
            }
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.remove::<ScrollBase>();
        emut.insert(ScrollBaseDying);
    }
}

impl Default for ScrollBase
{
    fn default() -> Self
    {
        Self{
            line_size: Self::default_line_size(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable component for the node of a scroll widget that will be scrolled.
///
/// The scroll area's [`Node`] must be manually set to scroll. For example, use
/// `FlexNode{ clipping:ScrollY }` for vertical scrolling. See [`Clipping`].
///
/// Inserts a [`ScrollPosition`] component, which is updated in the [`ScrollUpdateSet`] in [`PostUpdate`].
///
/// See [`ScrollBase`] and [`ScrollBar`].
#[derive(Reflect, Component, Default, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
#[require(ScrollPosition)]
pub struct ScrollArea;

//-------------------------------------------------------------------------------------------------------------------

/// The axis of a scrollbar.
///
/// See [`ScrollBar`].
#[derive(Reflect, Default, PartialEq, Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum ScrollAxis
{
    #[default]
    X,
    Y,
}

impl Into<SliderAxis> for ScrollAxis
{
    fn into(self: Self) -> SliderAxis
    {
        match self {
            Self::X => SliderAxis::X,
            Self::Y => SliderAxis::Y,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction loadable for a scroll widget's scrollbar.
///
/// Inserts a [`Slider`] to the entity. The slider direction will be inferred from the scroll axis
/// (standard for `X` and reverse for `Y`).
///
/// See [`ScrollBase`], [`ScrollArea`], and [`ScrollHandle`].
#[derive(Reflect, Component, Default, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ScrollBar
{
    /// Mirrors [`SliderAxis`].
    #[reflect(default)]
    pub axis: ScrollAxis,
    /// Mirrors [`Slider::bar_press`].
    #[reflect(default)]
    pub bar_press: SliderPress,
}

impl Instruction for ScrollBar
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let direction = match self.axis {
            SliderAxis::X => SliderDirection::Standard,
            SliderAxis::Y => SliderDirection::Reverse,
        };

        Slider{
            axis: self.axis.into(),
            direction,
            bar_press: self.bar_press.clone()
        }.apply(entity, world);

        // Add self to nearest ancestor scroll base.
        if let Some((_, computed_base)) = get_ancestor_mut::<ComputedScrollBase>(world, entity) {
            computed_base.add_bar(entity, slider.axis);
        } else {
            tracing::warn!("failed adding ScrollBar {entity:?} to scroll widget; no ancestor has ScrollBase");
        }
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.remove::<ScrollBar>();
        Slider::revert(entity, world);

        // Reapply nearest computed scroll base in case reverting this bar causes a 'dangling' bar to become
        // non-dangling.
        if let Some((_, computed_base)) = get_ancestor_mut::<ComputedScrollBase>(world, entity) {
            let other_computed_base = std::mem::take(&mut computed_base);
            other_computed_base.reapply_bars(world);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable component for a scroll widget's scrollbar's handle.
///
/// Inserts a [`SliderHandle`] to the target entity.
///
/// See [`ScrollBase`], [`ScrollArea`], and [`ScrollBar`].
#[derive(Reflect, Component, Default, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
#[require(SliderHandle)]
pub struct ScrollHandle;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event sent to [`ScrollBar`] entities whenever a mouse scroll touches them.
///
/// Will be sent even if the scrollbar can't consume any of the mouse scroll because the handle is already at the
/// end of the bar.
pub struct MouseScroll;

//-------------------------------------------------------------------------------------------------------------------

/// System set where scroll widgets are update.
///
/// - **PreUpdate**: The size of scrollbar handles is updated. Note that this will lag by 1 tick from content
/// size changes until bevy makes the UI layout algorithm more flexible.
/// - **PostUpdate**: The [`ScrollPosition`] of [`ScrollAreas`](ScrollArea) is updated.
#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct ScrollUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebScrollPlugin;

impl Plugin for CobwebScrollPlugin
{
    fn build(&self, app: &mut App)
    {
        // TODO: re-enable once COB scene macros are implemented
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/widgets/scroll", "scroll.cob");
        app.register_instruction_type::<ScrollBase>()
            .register_component_type::<ScrollArea>()
            .register_instruction_type::<ScrollBar>()
            .register_component_type::<ScrollHandle>()
            .init_resource::<ChildrenIterScratch>()
            .add_systems(First, cleanup_dead_bases.after(FileProcessingSet))
            .add_systems(
                PreUpdate,
                (
                    refresh_scroll_handles,
                    // We want the effects of picking events to override mouse scroll, so this is ordered before
                    // pointer events.
                    apply_mouse_scroll
                )
                    .in_set(ScrollUpdateSet)
                    .after(InputSystem)
                    .in_set(PickSet::Focus)
                    .after(update_interactions_hack)
                    .before(bevy::picking::events::pointer_events)
            )
            // TODO: this is just a hack because bevy's update_interactions system runs after pointer_events. This
            // system is fairly cheap to run. Revisit in bevy 0.16
            .add_systems(
                PreUpdate,
                update_interactions_hack
                    .in_set(PickSet::Focus)
                    .after(bevy::picking::focus::update_focus)
                    .before(bevy::picking::events::pointer_events),
            );
            .add_systems(
                PostUpdate,
                (
                    cleanup_dead_bases,
                    refresh_scroll_position
                )
                    .chain()
                    .in_set(ScrollUpdateSet)
                    .after(FileProcessingSet)
                    .after(DynamicStylePostUpdate)
                    .before(UiSystem::Prepare),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

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

fn apply_mouse_scroll()
{
/*
    - Update slider value from mouse scroll
        - AccumulatedMouseScroll
            - need to manually detect shift + scroll for horizontal?
            - how to translate 'line' unit to pixels?
                - hard-code arbitrary value? use command for this to get global setting?
        - need to look up pointer hit stack, apply to top-most intersecting scroll view without FocusPolicy::Block on
        higher entities
            - try to divide scroll distance to successive scroll views if topmost doesn't consume all distance
        - translate scroll distance to proportion of scroll area content size, add to slider value
            - use ComputedScrollBase to look up sliders
        - send MouseScroll entity events to ScrollBase entities
*/
}

//-------------------------------------------------------------------------------------------------------------------

fn refresh_scroll_state(
    mut c: Commands,
    ps: PseudoStateParam,
    mut iter_children: IterChildren,
    ui_surface: Res<UiSurface>, //ui_surface.get_layout(entity).content_size.{width, height}
    bases: Query<(&mut ScrollPosition, &ComputedScrollBase)>,
    areas: Query<(Entity, &ScrollArea, &Node)>,
    parents: Query<&Parent>,
    children: Query<&Children>,
    slider_vals: Reactive<SliderValue>,
    bar_handles: Query<&mut Node, (With<SliderHandle>, With<ScrollHandle>)>,
)
{
    for (area_entity, scroll_area, area_node) in areas.iter() {
        // Get area size.
        let area_size = area_node.size();

        // Get area content size.
        let content_size = ui_surface
            .get_layout(area_entity).map(|l| Vec2::new(l.content_size.width, l.content_size.height))
            .unwrap_or_default();

        // Look up base (base and area can be the same entity).
        let mut current = area_entity;
        let res = loop {
            if let Ok((scroll_pos, computed_base)) = bases.get_mut(current) {
                break Some((scroll_pos, computed_base));
            }

            let Some(parent) = parents.get(current) else {
                break None;
            };
            current = *parent;
        };
        let Some((scroll_pos, computed_base)) = res else { continue };

    }


/*
    Refresh ScrollPosition from slider value + relative size of area and content
- look up computed scroll base for each scroll area
*/
/*
    - Refresh handle sizes if changed
        - Look up content size from taffy?
        - If handle size changed to/from 100%, add/remove "HorizontalScroll", "VerticalScroll" states from scroll base
            - Set slider value to 0.0 if size reaches 100%
        - Note: it doesn't matter if this is before/after slider update, because the slider will be 'unaware' of the
        changed size until after layout
*/
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
#[derive(Reflect, Component, Default, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct ScrollBase;

impl Instruction for ScrollBase
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };

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

/// Instruction loadable for a scroll widget's scrollbar.
///
/// Wraps a [`Slider`]. The slider's axis should be [`SliderAxis::X`] or [`SliderAxis::Y`].
///
/// See [`ScrollBase`], [`ScrollArea`], and [`ScrollHandle`].
#[derive(Reflect, Component, Default, PartialEq, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ScrollBar(pub Slider);

impl Instruction for ScrollBar
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        self.0.clone().apply(entity, world);

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

/// System set in [`PostUpdate`] where scroll widgets are update.
///
/// - The [`ScrollPosition`] of [`ScrollAreas`](ScrollArea) is updated.
/// - The size of scrollbar handles is updated. Note that this will lag by 1 tick from content size changes until
/// bevy makes the UI layout algorithm more flexible.
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
            // We want the effects of picking events to override mouse scroll.
            .add_systems(PreUpdate, apply_mouse_scroll.before(PickSet::ProcessInput))
            .add_systems(
                PostUpdate,
                (
                    cleanup_dead_bases,
                    refresh_scroll_state.in_set(ScrollUpdateSet)
                )
                    .chain()
                    .after(FileProcessingSet)
                    .after(DynamicStylePostUpdate)
                    .before(UiSystem::Prepare),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/*
- PreUpdate
    - Refresh handle size if it changed
        - Look up content size from taffy?
        - If handle size changed to/from 100%, add/remove "HorizontalScroll", "VerticalScroll" states from scroll base
            - Set slider value to 0.0 if size reaches 100%
        - Note: it doesn't matter if this is before/after slider update, because the slider will be 'unaware' of the
        changed size until after layout
    - Update slider value from mouse scroll
        - AccumulatedMouseScroll
            - need to manually detect shift + scroll for horizontal?
            - how to translate 'line' unit to pixels?
                - hard-code arbitrary value? use command for this to get global setting?
        - need to look up pointer hit stack, apply to top-most intersecting scroll view without FocusPolicy::Block on
        higher entities
            - try to divide scroll distance to successive scroll views if topmost doesn't consume all distance
        - send MouseScroll entity events to ScrollBase entities

- PostUpdate (before layout)
    - Refresh ScrollPosition from slider value + relative size of area and content

ScrollBase

ScrollArea
    - Uses clipping: Scroll

ScrollBar(Slider)
    - On slider value, update ScrollPosition using appropriate axis

ScrollHandle(SliderHandle)


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
        - bespoke solution likely best: need to also support pagination via mouse scroll events
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


/*
Demos
Note: use buttons to add/remove content from scroll areas

- Entire window is a scroll area using 'show scroll bar when scrolling'
    - Need the 'fold' between elements to overlap w/ bottom of window to imply scrolling is possible.

- Basic

- Scrollbars as overlays
    - invisible scrollbar, visible but semi-transparent handle

- Scrollbars as insets (w/ horizontal spreading across entire width)
    - Column:
        - Scroll area and vertical bar in row together
        - Horizontal bar

- Scrollbar with FocusPolicy::Block and Pickable { should_block_lower: true, is_hoverable: true }
    - layered scrolling does not occur

- Styling
    - Shadow behind vertical scrollbar when there is a horizontal bar.
        - Add shim node behind bar with Static<NodeShadow>{state:["HorizontalScroll"]}, in control group with scroll base
    - Show/hide horizontal bar when there is scrollable content
        - Add shim node with Static<DisplayControl>{value:Hide} and {state:["HorizontalScroll"] value:Display}
    - Invisible/visible vertical bar when there is scrollable content (manual 'scrollbar-gutter')
    - Show scroll bar when scrolling
        - Update
            - Update scroll timers
                - when timer ends, remove 'is scrolling' state
        - ScrollBase
            - On MouseScroll event
                - if no scrolling timer
                    - initialize timer
                    - add 'is scrolling' state to relevant scrollbar
                - if scrolling timer
                    - refresh timer
        - ScrollBar
            - On pointer hover or press while in state 'is scrolling'
                - refresh scrolling timer
            - On enter 'is scrolling' state
                - animate opacity of bar handle to target
                - static
                    - add Enabled state (enables interactions) -> tie animations to 'Enabled'
                    - add picking behavior: Pickable { should_block_lower: false, is_hoverable: true }
                        - don't block lower so scrolls can reach the scroll base (scrolls don't use picking, but we use
                        picking hit stacks to decide where scroll amounts should go)
            - On enter 'not scrolling' state
                - animate opacity of bar handle to zero after delay
                - static
                    - add Disabled state (disables interactions)
                    - add picking behavior: Pickable { should_block_lower: false, is_hoverable: false }
            - Slider bar
                - Set initial opacity to zero with plain loadable
                - In no state: idle value of slider bar is '0.0 alpha', with enter animation
                - on hover
                    - 
                - In state 'is scrolling': hover value of slider bar is '1.0 alpha', animated with zero duration and delay so the
                attribute overrides the idle attribute

*/

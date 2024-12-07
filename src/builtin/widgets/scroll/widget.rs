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


/*
Demos
Note: use buttons to add/remove rows from scroll areas
    - for 'expand horizontal', make the 'content blob entity' expand in width on press
Note: add name to all scroll areas
    - add text with arrow that points at scrollbar for the 'entire window' scrollbar

- Entire window is a scroll area using Firefox style
    - Need the 'fold' between elements to overlap w/ bottom of window to imply scrolling is possible.

- Basic vertical

- Scrollbars as overlays
    - invisible scrollbar, visible but semi-transparent handle

- Scrollbars as insets (w/ horizontal spreading across entire width)
    - Column:
        - Scroll area and vertical bar in row together
        - Horizontal bar

- ScrollArea with FocusPolicy::Block and Picking::Sink
    - layered scrolling does not occur

- SublimeText (bi-directional)
    - Shadow behind vertical scrollbar when there is a horizontal bar.
        - Add shim node behind bar with Static<NodeShadow>{state:["HorizontalScroll"]}, in control group with scroll base
    - Show/hide horizontal bar when there is scrollable content
        - Add shim node with Static<DisplayControl>{value:Hide} and {state:["HorizontalScroll"] value:Display}
    - Invisible/visible vertical bar when there is scrollable content (manual 'scrollbar-gutter')

- Firefox (for entire scroll area)
    - Show scroll bar when scrolling
        - Update
            - Update scroll timers
                - when timer ends, remove 'is scrolling' and 'hover activated' states
        - ScrollBase
            - On MouseScroll event
                - if no scrolling timer
                    - initialize timer
                    - add 'is scrolling' state to relevant scrollbar
                - if scrolling timer
                    - refresh timer
        - ScrollBar
            - Initial bar and handle opacity: zero
            - On pointer hover or press while in state 'is scrolling'
                - refresh scrolling timer
                - add 'hover activated' state
            - On enter 'not scrolling' state
                - animate opacity of bar and handle to zero after delay
                - static
                    - add Disabled state (disables interactions)
                    - add picking behavior: Picking::Ignore
                    - set width to 'small'
            - On enter 'is scrolling' state
                - animate opacity of bar and handle to target
                - static
                    - add Enabled state (enables interactions) -> tie animations to 'Enabled'
                    - add picking behavior: Picking::Pass
                        - don't block lower so scrolls can reach the scroll base (scrolls don't use picking, but we use
                        picking hit stacks to decide where scroll amounts should go)
                    - add Visibility::Hidden to bar (opacity animates 'hidden')
                        - need Visibility::Visible on handle to avoid inheritance
            - On enter 'hover activated' state
                - static
                    - add Visibility::Show
                    - set width to 'large'
                        - note: need to set radius of handle to 50% of bar width

*/

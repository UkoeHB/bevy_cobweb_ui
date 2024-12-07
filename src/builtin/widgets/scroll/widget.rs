/*

- Scroll

- PreUpdate
    - if content area size changed, need to adjust the slider value so it points at the same location as last tick
        - unless there is an active drag, in which case we just apply the current slider value
    - if content area is within the viewing window minus scroll bars, then disable scroll bars (Display::Hidden + PseudoState::Disabled)
    - if content area changes to be outsize viewing window, re-enable scroll bars
    - if content area changes or viewing window changes, adjust slider bar width to equal Val::Percent(percent in view)

- PostUpdate (before slider update)
    - Recalculate current v

    - Need to handle case were slider value must change because content area size changed


Scroll base
    - Add/remove states: "HorizontalScroll", "VerticalScroll"

Scroll area
    - Uses clipping: Scroll

Scroll bar
    - scroll axis
    - slider bar press behavior: inert, jump, animate
    - scroll bar press behavior: move to point, next page (press hold: delay then rapidly paginate)


- AccumulatedMouseScroll
    - if hovering scroll base, refresh 'is scrolling' state timer
    - when 'is scrolling' state timer ends, remove state
        - can use this for the 'show/hide scroll bar when scrolling' animation
    - how to translate 'line' unit to pixels?
        - hard-code arbitrary value?

- TouchEvent
    - need to track touch id lifetime
        - record initial scroll position when touch starts
    - scroll = distance traveled
    - how to block touch events when elements are pressed in view? and likewise, how to cancel presses on elements when
    scrolling?

unimplemented
- mobile kinematic scrolling w/ buffers at top/bottom
    - https://stackoverflow.com/a/7224899
- macos-style 'jump one page on scrollbar press'
    - needs animation framework overhaul or bespoke solution
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
*/


/*
Demos

- Basic

- Scrollbars as overlays

- Scrollbars as insets (w/ horizontal spreading across entire width)
    - Column:
        - Scroll area and vertical bar in row together
        - Horizontal bar

- Styling
    - Shadow behind vertical scrollbar when there is a horizontal bar.
        - Add shim node behind bar with Static<NodeShadow>{state:["HorizontalScroll"]}, in control group with scroll base
    - Show/hide horizontal bar when there is scrollable content
        - Add shim node with Static<DisplayControl>{value:Hide} and {state:["HorizontalScroll"] value:Display}
    - Invisible/visible vertical var when there is scrollable content (manual 'scrollbar-gutter')
    - Show scroll bar when scrolling
        - On enter 'is scrolling' state
            - animate opacity of bar handle to target
            - static: add interactive
        - On enter 'not scrolling' state
            - animate opacity of bar handle to zero
            - static: ignore interactions
        - Slider bar
            - Set initial opacity to zero with plain loadable
            - In no state: idle value of slider bar is '0.0 alpha', with enter animation
            - In state 'is scrolling': hover value of slider bar is '1.0 alpha', animated with zero duration and delay so the
            attribute overrides the idle attribute

*/

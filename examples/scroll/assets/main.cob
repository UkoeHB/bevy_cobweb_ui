/*
Demos (TODO)
Note: use buttons to add/remove rows from scroll areas
    - for 'expand horizontal', make the 'content blob entity' expand in width on press
Note: add name to all scroll areas
    - add text with arrow that points at scrollbar for the 'entire window' scrollbar

- Entire window is a scroll area using Firefox style
    - Need the 'fold' between elements to overlap w/ bottom of window to imply scrolling is possible.

- Basic vertical

- Scrollbars as overlays
    - ScrollBase and ScrollArea on same entity, scrollbars as absolute nodes w/ right/bottom shoved over/down
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
#scenes
"scene"
    TextLine{ text: "Scroll isn't ready yet, check back again later!" }

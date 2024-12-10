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
        - ScrollBar
            - On MouseScroll event
                - if no scrolling timer
                    - initialize timer
                    - add 'is scrolling' state to relevant scrollbar
                - if scrolling timer
                    - refresh timer
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
#import
builtin.colors.tailwind as tw

#scenes
"button_add"
    FlexNode{width:75px height:75px justify_main:Center justify_cross:Center}
    Splat<Border>(2px)
    BrRadius(15px)
    BorderColor(#000000)
    BackgroundColor{idle:#00000000 hover:$tw::GRAY_400}

    ""
        TextLine{text:"+" size:40}
        TextLineColor(#000000)

"add_row"
    Margin{top:20px left:20px}

    //button_add

"blob_row"
    FlexNode{flex_direction:Row justify_main:Center justify_cross:FlexStart}
    Border{bottom:1px}
    BorderColor(#60000000)

    "kill"
        AbsoluteNode{top:5px left:5px}
        Splat<Border>(1px)
        BorderColor(#000000)
        BackgroundColor{idle:#00000000 hover:$tw::GRAY_400}

        ""
            Margin{top:5px bottom:5px left:7px right:7px}
            TextLine{text:"X" size:20}
            TextLineColor(#000000)

    //blob

    //button_add

"blob"
    FlexNode{width:150px height:150px}
    BrRadius(25px)
    BackgroundColor($tw::ROSE_600)

    "kill"
        AbsoluteNode{top:30px left:30px}
        Splat<Border>(1px)
        BorderColor(#000000)
        BackgroundColor{idle:#00000000 hover:$tw::GRAY_400}

        ""
            Margin{top:5px bottom:5px left:7px right:7px}
            TextLine{text:"X" size:20}
            TextLineColor(#000000)


"scene"
    FlexNode{width:100vw flex_direction:Column justify_main:FlexStart}

    ""
        FlexNode{width:100% flex_direction:Row justify_main:SpaceEvenly}

        //-------------------------------------------------------------------------------------------------------------------
        // Simplest possible scrollview.
        // - Scrollbars are always present and are placed adjacent to content.
        //-------------------------------------------------------------------------------------------------------------------
        "basic"
            FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:Center}

            "header"
                TextLine{text:"Basic"}

            "scroll"
                ScrollBase
                FlexNode{width:250px height:254px flex_direction:Row justify_main:FlexStart justify_cross:FlexStart}
                Splat<Border>(2px)
                BorderColor(#000000)
                BackgroundColor($tw::STONE_500)

                "view"
                    ScrollView
                    FlexNode{max_height:100% flex_grow:1 clipping:ScrollY}

                    // TODO: remove this extra node in bevy 0.15.1
                    "shim"
                        ScrollShim
                        FlexNode{
                            width:100% height:300px
                            flex_direction:Column justify_main:FlexStart justify_cross:FlexStart
                            clipping:None
                        }
                        BackgroundColor(#005500)

                        ""
                            FlexNode{width:50px height:150px}
                            BackgroundColor(#888888)

                "vertical"
                    ScrollBar{axis:Y}
                    FlexNode{height:100% width:14px}
                    BackgroundColor($tw::STONE_600)

                    "handle"
                        ScrollHandle
                        AbsoluteNode{width:100%}
                        BackgroundColor($tw::STONE_800)

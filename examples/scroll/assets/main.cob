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

- SublimeText (bi-directional)
    - Shadow behind vertical scrollbar when there is a horizontal bar.
        - Add shim node behind bar with Static<NodeShadow>{state:["HorizontalScroll"]}, in control group with scroll base
        - Add reactor that changes opacity of shim node based on horizontal slider value. For simplicity do 100% = 0, else 1.
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

#defs
$demo_size = \ width:250px height:250px \
$demo_border = 5px
$demo_br_color = #000000
$demo_bg_color = #009900

#scenes
//-------------------------------------------------------------------------------------------------------------------
// Simplest possible vertical scrollview.
// - Vertical scrollbar is always present and is placed adjacent to content.
//-------------------------------------------------------------------------------------------------------------------
"basic"
    FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:Center}

    "header"
        TextLine{text:"Basic"}

    "scroll"
        ScrollBase
        FlexNode{$demo_size flex_direction:Row justify_cross:FlexStart}
        Splat<Border>($demo_border)
        BorderColor($demo_br_color)
        BackgroundColor($demo_bg_color)

        "view"
            ScrollView
            FlexNode{max_height:100% flex_grow:1 clipping:ScrollY}

            // TODO: remove this extra node in bevy 0.15.1
            "shim"
                ScrollShim
                //FlexNode{width:100% flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
                FlexNode{height:300px}

        "vertical"
            ScrollBar{axis:Y}
            FlexNode{height:100% width:14px}
            BackgroundColor(#888888)

            "handle"
                ScrollHandle
                AbsoluteNode{width:100%}
                BackgroundColor(#BBBBBB)

//-------------------------------------------------------------------------------------------------------------------
// Bi-directional scrollview with scrollbars on top of content.
//-------------------------------------------------------------------------------------------------------------------
"overlay"
    FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:Center}

    "header"
        TextLine{text:"Overlay"}

    "scroll"
        ScrollBase
        FlexNode{$demo_size}
        Splat<Border>($demo_border)
        BorderColor($demo_br_color)
        BackgroundColor($demo_bg_color)

        // View is separate from base so we can get the right size for the bar shim. Absolute nodes include the
        // parent's border when doing e.g. `width:100%`, but flex nodes do not. So adding this node will give us the
        // non-border size of the base.
        "view"
            ScrollView
            FlexNode{width:100% height:100% flex_direction:Column}

            // Bars first for higher sort order than content.
            "bar_shim"
                AbsoluteNode{width:100% height:100% flex_direction:Column justify_cross:FlexEnd}

                "vertical"
                    ScrollBar{axis:Y}
                    FlexNode{flex_grow:1 width:14px}

                    "handle"
                        ScrollHandle
                        AbsoluteNode{width:100%}
                        BackgroundColor(#60FFFFFF)

                "horizontal"
                    ScrollBar{axis:X}
                    FlexNode{width:100% height:14px}

                    "handle"
                        ScrollHandle
                        AbsoluteNode{height:100%}
                        BackgroundColor(#60FFFFFF)

            // TODO: remove this extra node in bevy 0.15.1
            "shim"
                ScrollShim
                //FlexNode{width:100% flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
                FlexNode{width:300px height:300px}

//-------------------------------------------------------------------------------------------------------------------
// Bi-directional scrollview with scrollbars separate from content.
//-------------------------------------------------------------------------------------------------------------------
"inset"
    FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:Center}

    "header"
        TextLine{text:"Inset"}

    "scroll"
        ScrollBase
        FlexNode{$demo_size flex_direction:Column}
        Splat<Border>($demo_border)
        BorderColor($demo_br_color)
        BackgroundColor($demo_bg_color)

        // We have extra shims and absolute nodes here because flexbox sucks.
        // TODO: can this be simplified?
        "view_shim"
            FlexNode{width:100% flex_grow:1}

            "view_shim_inner"
                AbsoluteNode{width:100% height:100% flex_direction:Row}

                "view"
                    ScrollView
                    FlexNode{height:100% flex_grow:1 clipping:ScrollXY}

                    // TODO: remove this extra node in bevy 0.15.1
                    "shim"
                        ScrollShim
                        //FlexNode{width:100% flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
                        AbsoluteNode{width:300px height:300px}

                "vertical"
                    ScrollBar{axis:Y}
                    FlexNode{height:100% width:14px}
                    BackgroundColor(#888888)

                    "handle"
                        ScrollHandle
                        AbsoluteNode{width:100%}
                        BackgroundColor(#BBBBBB)

        "horizontal"
            ScrollBar{axis:X}
            FlexNode{width:100% height:14px}
            BackgroundColor(#888888)

            "handle"
                ScrollHandle
                AbsoluteNode{height:100%}
                BackgroundColor(#BBBBBB)

//-------------------------------------------------------------------------------------------------------------------
// Bi-directional scrollview mimicking a SublimeText configuration.
// - Scrollbars are separate from content.
// - Vertical bar is always present. This way content layout does not change when adding scrolling. However the bar
//   is invisible if there is no vertical scrollable content.
// - Horizontal bar is only present in layout if there is horizontal content. This avoids having a gap at the bottom
//   of the scroll view.
// - Vertical bar gains a shadow when there is horizontal content. Its background blends into the content
//   background, giving the illusion of an overlay.
// - Pressing scrollbars without dragging them animates the handle into position. A nice touch.
//-------------------------------------------------------------------------------------------------------------------
"sublime"
    FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:Center}

    "header"
        TextLine{text:"Inset"}

    "scroll"
        ScrollBase
        FlexNode{$demo_size flex_direction:Column}
        Splat<Border>($demo_border)
        BorderColor($demo_br_color)
        BackgroundColor($demo_bg_color)

        // We have extra shims and absolute nodes here because flexbox sucks.
        // TODO: can this be simplified?
        "view_shim"
            FlexNode{width:100% flex_grow:1}

            "view_shim_inner"
                AbsoluteNode{width:100% height:100% flex_direction:Row}

                "view"
                    ScrollView
                    FlexNode{height:100% flex_grow:1 clipping:ScrollXY}

                    // TODO: remove this extra node in bevy 0.15.1
                    "shim"
                        ScrollShim
                        //FlexNode{width:100% flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
                        AbsoluteNode{width:300px height:300px}

                "vertical"
                    ScrollBar{axis:Y}
                    FlexNode{height:100% width:14px}
                    BackgroundColor(#888888)

                    "handle"
                        ScrollHandle
                        AbsoluteNode{width:100%}
                        BackgroundColor(#BBBBBB)

        "horizontal"
            ScrollBar{axis:X}
            FlexNode{width:100% height:14px}
            BackgroundColor(#888888)

            "handle"
                ScrollHandle
                AbsoluteNode{height:100%}
                BackgroundColor(#BBBBBB)

//-------------------------------------------------------------------------------------------------------------------
// Root scene with FireFox-like vertical scrolling.
// - Scrollbar overlays content
// - Scroll handle is only visible after scrolling the mouse. It will fade away after a delay.
// - Scrollbar is only visibile if it is hovered after the scroll handle becomes visible.
// - Scrollbar and handle grow in size if the scrollbar becomes visible.
//-------------------------------------------------------------------------------------------------------------------
"scene"
    ScrollBase
    FlexNode{width:100vw min_height:100vh flex_direction:Column justify_main:FlexStart}
    BackgroundColor($tw::CYAN_700)

    // Bar first for higher sort order than content.
    "bar_shim"
        AbsoluteNode{width:100% height:100% flex_direction:Row justify_main:FlexEnd}

        "vertical"
            ScrollBar{axis:Y}
            FlexNode{height:1 width:14px}

            "handle"
                ScrollHandle
                AbsoluteNode{width:100%}
                BackgroundColor(#60FFFFFF)

    // TODO: remove this extra node in bevy 0.15.1
    "shim"
        ScrollShim
        //FlexNode{width:100% flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
        FlexNode{width:300px height:300px}

        //text pointing at vertical scrollbar

        "row1"
            FlexNode{width:100% margin:{top:25px bottom:25px} flex_direction:Row justify_main:SpaceEvenly}

        "row2"
            FlexNode{width:100% margin:{top:20px bottom:25px} flex_direction:Row justify_main:SpaceEvenly}

//-------------------------------------------------------------------------------------------------------------------
// + button
//-------------------------------------------------------------------------------------------------------------------
"button_add"
    FlexNode{width:75px height:75px justify_main:Center justify_cross:Center}
    Splat<Border>(2px)
    BrRadius(15px)
    BorderColor(#000000)
    BackgroundColor{idle:#00000000 hover:$tw::GRAY_400}

    ""
        TextLine{text:"+" size:40}
        TextLineColor(#000000)

//-------------------------------------------------------------------------------------------------------------------
// Add-row button
//-------------------------------------------------------------------------------------------------------------------
"add_row"
    Margin{top:20px left:20px}

    //button_add

//-------------------------------------------------------------------------------------------------------------------
// Row of blobs
//-------------------------------------------------------------------------------------------------------------------
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

//-------------------------------------------------------------------------------------------------------------------
// Blob to fill up the content of scroll areas.
//-------------------------------------------------------------------------------------------------------------------
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

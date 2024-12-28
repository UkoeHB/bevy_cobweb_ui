#import
builtin.colors.tailwind as tw

#defs
$demo_size = \ width:500px height:500px \
$demo_border = 5px
$demo_br_color = #000000
$demo_bg_color = #009900
$sublime_bg_color = #282922
$sublime_bar_press = Animate{duration:0.05 ease:InOutSine}
$blob_small = 400px
$blob_big = 600px

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
            FlexNode{height:100% flex_grow:1 clipping:ScrollYClipX}

            // TODO: remove this extra node in bevy 0.15.1
            "shim"
                ScrollShim
                AbsoluteNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

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
        "view_shim"
            FlexNode{width:100% height:100% flex_direction:Column clipping:ScrollXY}

            "view"
                ScrollView
                FlexNode{width:100% height:100% flex_direction:Column clipping:ScrollXY}

                // TODO: remove this extra node in bevy 0.15.1
                "shim"
                    ScrollShim
                    AbsoluteNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

            // Bars last for higher sort order than content.
            "bar_shim"
                AbsoluteNode{width:100% height:100% flex_direction:Column justify_cross:FlexEnd}

                "vertical"
                    ScrollBar{axis:Y}
                    FlexNode{flex_grow:1 width:14px}
                    FocusPolicy::Block

                    "handle"
                        ScrollHandle
                        AbsoluteNode{width:100%}
                        BackgroundColor(#60FFFFFF)

                "horizontal"
                    ScrollBar{axis:X}
                    FlexNode{width:100% height:14px}
                    FocusPolicy::Block

                    "handle"
                        ScrollHandle
                        AbsoluteNode{height:100%}
                        BackgroundColor(#60FFFFFF)


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

        "view_shim"
            FlexNode{width:100% flex_grow:1 flex_direction:Row}

            "view"
                ScrollView
                FlexNode{height:100% flex_grow:1 clipping:ScrollXY}

                // TODO: remove this extra node in bevy 0.15.1
                "shim"
                    ScrollShim
                    AbsoluteNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

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
        TextLine{text:"Sublime-like"}

    "scroll"
        ScrollBase
        ControlRoot
        FlexNode{$demo_size flex_direction:Column}
        BackgroundColor($sublime_bg_color)

        "view_shim"
            FlexNode{width:100% flex_grow:1 flex_direction:Row}

            "view"
                ScrollView
                FlexNode{height:100% flex_grow:1 clipping:ScrollXY}

                // TODO: remove this extra node in bevy 0.15.1
                "shim"
                    ScrollShim
                    AbsoluteNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

            // The vertical scrollbar is invisible if there is no scrollable content.
            // - Invisible but *not* removed from layout.
            "vertical"
                ControlMember // Control group with scroll base
                FlexNode{height:100% width:16px}
                Multi<Static<Visibility>>[
                    {value:Hidden}
                    {state:[Custom("VerticalScroll")] value:Inherited}
                    {state:[Custom("HorizontalScroll")] value:Inherited}
                ]

                // Shim node manually controls whether the shadow node is visible. The shadow fades out when close to
                // 100% on the horizontal scrollbar.
                "shadow_shim"
                    AbsoluteNode{
                        left:auto right:0px height:100% width:200%
                        clipping:ClipXY flex_direction:Column justify_cross:FlexEnd justify_main:Center
                    }

                    // Add an extra node to increase the shadow height since clipping Y makes it not cover the entire edge.
                    "height_shim"
                        AbsoluteNode{height:200% width:100% flex_direction:Column justify_cross:FlexEnd}

                        "shadow"
                            ControlMember // Control group with scroll base
                            // Huge height to make the shadow stretch properly along the edge (it doesn't behave well w/
                            // y clipping).
                            FlexNode{height:4000% width:50%}
                            // Note: this will be applied 1 frame late because 'start scroll' is detected after layout (and after
                            // attributes are applied).)
                            Multi<Static<DisplayControl>>[{value:Hide} {state:[Custom("HorizontalScroll")] value:Show}]
                            NodeShadow{spread_radius:1px blur_radius:10px}

                "gutter"
                    FlexNode{height:100% width:100% flex_direction:Column justify_cross:Center padding:{top:6px bottom:6px}}
                    BackgroundColor($sublime_bg_color)

                    "bar"
                        ControlRoot
                        ScrollBar{axis:Y bar_press:$sublime_bar_press}
                        FlexNode{flex_grow:1 width:7px}
                        BrRadius(2px)
                        BackgroundColor(#26AAAAAA)

                        "handle"
                            ScrollHandle
                            ControlMember // Control group with scrollbar
                            AbsoluteNode{width:100% height:100px} // Need pretend height for radius to work.
                            BrRadius(2px)
                            Responsive<BackgroundColor>{idle:#80BBBBBB hover:#B9EEEEEE}

        // The horizontal bar is only displayed when the scroll view has horizontally-scrollable content.
        "horizontal"
            ControlMember // Control group with scroll base
            FlexNode{width:100% height:16px flex_direction:Row justify_cross:Center padding:{left:6px right:20px}}
            BackgroundColor($sublime_bg_color)
            Multi<Static<DisplayControl>>[{value:Hide} {state:[Custom("HorizontalScroll")] value:Show}]

            "bar"
                ScrollBar{axis:X bar_press:$sublime_bar_press}
                ControlRoot
                FlexNode{flex_grow:1 height:7px}
                BrRadius(2px)
                BackgroundColor(#26AAAAAA)

                "handle"
                    ScrollHandle
                    ControlMember // Control group with scrollbar
                    AbsoluteNode{height:100% width:100px} // Need pretend width for radius to work.
                    BrRadius(2px)
                    Responsive<BackgroundColor>{idle:#80BBBBBB hover:#B9EEEEEE}

//-------------------------------------------------------------------------------------------------------------------
// Root scene with FireFox-like vertical scrolling.
// - Scrollbar overlays content
// - Scroll handle is only visible after scrolling the mouse. It will fade away after a delay.
// - Scrollbar is only visibile if it is hovered after the scroll handle becomes visible.
// - Scrollbar and handle grow in size if the scrollbar becomes visible.
//-------------------------------------------------------------------------------------------------------------------
"scene"
    ScrollBase
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:FlexStart}
    BackgroundColor($tw::CYAN_700)

    "text"
        AbsoluteNode{left:auto top:10px right:21px}
        TextLine{text:"Firefox-like"}

    "view"
        ScrollView
        FlexNode{width:100% height:100% flex_direction:Column justify_main:FlexStart clipping:ScrollY}

        // TODO: remove this extra node in bevy 0.15.1
        "shim"
            ScrollShim
            FlexNode{width:100% flex_direction:Column justify_main:FlexStart}

            "row1"
                FlexNode{width:100% margin:{top:25px bottom:25px} flex_direction:Row justify_main:SpaceEvenly}

            "row2"
                FlexNode{width:100% margin:{top:20px bottom:25px} flex_direction:Row justify_main:SpaceEvenly}

    // Scrollbar above content
    "bar_shim"
        AbsoluteNode{width:100% height:100% flex_direction:Row justify_main:FlexEnd}
        Picking::Ignore // The shim should not block picking.

        // Gutter shim adds padding to top and bottom.
        "gutter"
            ControlRoot
            FlexNode{
                height:100% border:{left:1px} padding:{top:2.5px bottom:2.5px right:1px left:-1px}
                flex_direction:Column justify_cross:Center}
            BorderColor(#40555555)
            BackgroundColor(#60BBBBBB)
            Multi<Static<Visibility>>[
                {state:[Custom("IsScrolling")] value:Hidden}
                {state:[Custom("IsScrolling") Custom("HoverActivated")] value:Visible}
            ]
            PropagateOpacity(0)
            Multi<Animated<PropagateOpacity>>[
                {enter_idle_with:{duration:0.1 ease:OutSine} idle:0 delete_on_entered:true}
                {state:[Custom("IsScrolling")] enter_idle_with:{duration:0.1 ease:OutSine} idle:1 delete_on_entered:true}
            ]

            "vertical"
                ScrollBar{axis:Y}
                ControlMember
                FlexNode{flex_grow:1 flex_direction:Column justify_cross:Center border:{left:1px}}
                Multi<Static<Width>>[
                    {state:[Custom("IsScrolling")] value:12px}
                    {state:[Custom("IsScrolling") Custom("HoverActivated")] value:16px}
                ]
                // Note that when 'Sink' is used, mouse scrolls will still propagate to the scroll base via
                // event bubbling.
                Multi<Static<Picking>>[{value:Ignore} {state:[Custom("IsScrolling")] value:Sink}]
                FocusPolicy::Pass
                Visibility::Visible // Required to receive interactions while the gutter is Visibility::Hidden
                Interactive

                "handle"
                    ScrollHandle
                    ControlMember
                    AbsoluteNode{height:100px} // Need pretend height for radius to work.
                    BackgroundColor(#C0212121)
                    Multi<Static<Picking>>[{value:Ignore} {state:[Custom("IsScrolling")] value:Sink}]
                    FocusPolicy::Pass
                    Visibility::Visible // Propagated opacity controls visibility of the handle.
                    Multi<Static<Width>>[
                        {state:[Custom("IsScrolling")] value:8px}
                        {state:[Custom("IsScrolling") Custom("HoverActivated")] value:12px}
                    ]
                    Multi<Static<BrRadius>>[
                        {state:[Custom("IsScrolling")] value:4px}
                        {state:[Custom("IsScrolling") Custom("HoverActivated")] value:6px}
                    ]

//-------------------------------------------------------------------------------------------------------------------
// Blob to fill up the content of scroll areas.
//-------------------------------------------------------------------------------------------------------------------
"blob"
    FlexNode{justify_main:Center justify_cross:Center}
    ControlRoot
    BrRadius(25px)
    BackgroundColor($tw::ROSE_500)
    Multi<Static<Width>>[{value:$blob_small} {state:[Custom("Wide")] value:$blob_big}]
    Multi<Static<Height>>[{value:$blob_small} {state:[Custom("Tall")] value:$blob_big}]
    Splat<Margin>(20px)
    Interactive

    "text"
        ControlMember
        Multi<Static<TextLine>>[
            {value:{text:"Make taller"}}
            {state:[Custom("Tall")] value:{text:"Make wider"}}
            {state:[Custom("Wide") Custom("Tall")] value:{text:"Reset"}}
        ]

//-------------------------------------------------------------------------------------------------------------------
// Blob for SublimeText demo.
//-------------------------------------------------------------------------------------------------------------------
"blob_sublime"
    FlexNode{justify_main:Center justify_cross:Center}
    ControlRoot
    Splat<Border>(2px)
    BrRadius(25px)
    BorderColor(#FFFFFF)
    Multi<Static<Width>>[{value:$blob_small} {state:[Custom("Wide")] value:$blob_big}]
    Multi<Static<Height>>[{value:$blob_small} {state:[Custom("Tall")] value:$blob_big}]
    Splat<Margin>(20px)
    Interactive

    "text"
        ControlMember
        Multi<Static<TextLine>>[
            {value:{text:"Make taller"}}
            {state:[Custom("Tall")] value:{text:"Make wider"}}
            {state:[Custom("Wide") Custom("Tall")] value:{text:"Reset"}}
        ]

// Frame scenes for the editor.

#manifest
self as editor.frame

#import
builtin.colors.tailwind as tw

#commands
// The user can override this by inserting a new PrimaryCursor command.
// TODO: find a better way to 'get back to default' when a temp cursor is done being used, that is compatible with
// users defining their own cursor scheme
PrimaryCursor(System(Default))

#scenes
"base"
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
    BackgroundColor($tw::STONE_800)
    ResponsiveCursor{hover:System(Default)}

    "dropdown"
        FlexNode{
            width:100% border:{bottom:1px}
            flex_direction:Column justify_main:FlexStart justify_cross:Center
        }
        BackgroundColor(#000000)
        BorderColor(#FFFFFF)
        ResponsiveCursor{hover:System(Grab)}
        FocusPolicy::Block

    "content"
        FlexNode{width:100% flex_grow:1 flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

    "footer"
        FlexNode{width:100% flex_direction:Row justify_main:FlexEnd justify_cross:Center}

        "unsaved"
            TextLine{size:14 text:"unsaved changes"}
            Multi<Static<TextLineColor>>[{value:#00000000} {state:[Enabled] value:#AABBBBBB}]

        "save"
            FlexNode{margin:{top:6px bottom:6px right:8px left:8px} justify_main:Center justify_cross:Center}
            BrRadius(3px)
            Responsive<BackgroundColor>{idle:$tw::CYAN_800 hover:$tw::CYAN_700 press:$tw::CYAN_600}

            "text"
                FlexNode{margin:{top:5px bottom:5px left:10px right:10px}}
                TextLine{size:20 text:"Save"}


"empty_dropdown_entry"
    FlexNode{height:35px width:100%}

"dropdown_entry"
    // Shim lets us interact with the whole entry, not just the text.
    ControlRoot
    FlexNode{width:100% padding:{left:7px top:10px bottom:10px} flex_direction:Row justify_main:FlexStart justify_cross:Center}
    Multi<Responsive<BackgroundColor>>[{idle:#00000000 hover:#44BBBBBB} {state:[Selected] idle:#22BBBBBB hover:#44BBBBBB}]

    "text"
        ControlMember
        TextLine{ size:14 text:"" }
        Multi<Responsive<TextLineColor>>[
            {idle:#CCCCCC hover:#FFFFFF} {state:[Selected] idle:#DFDFDF hover:#FFFFFF}
            {state:[Folded] idle:#FFFFFF}
        ]

"file_frame"
    FlexNode{width:100% height:100% flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

    "commands"
        FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

        "title"
            TextLine{size:14 text:"#commands"}
            TextLineColor($tw::RED_400)

        "content"
            FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

    "shim"
        FlexNode{height:15px}

    "scenes"
        FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

        "title"
            TextLine{size:14 text:"#scenes"}
            TextLineColor($tw::RED_400)

        "content"
            FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

"scene_node"
    FlexNode{flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

    "name"
        TextLine{size:14}
        TextLineColor($tw::AMBER_300)

    "content"
        FlexNode{margin:{left:10px} flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

"loadable"
    FlexNode{flex_direction:Row justify_main:FlexStart justify_cross:FlexStart}
    
    "name"
        TextLine{size:14}
        TextLineColor($tw::BLUE_300)

    "content"
        FlexNode{margin:{left:4px} flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

"file_not_editable"
    TextLine{size:14 text:"File not editable"}
    TextLineColor(#FFFFFF)

"unsupported"
    TextLine{size:14 text:"<unsupported>"}
    TextLineColor(#FFFFFF)

"reflect_fail"
    TextLine{size:14 text:"<reflection failed>"}
    TextLineColor(#FFFFFF)

"destructure_unsupported"
    TextLine{size:14 text:"<no widget found>"}
    TextLineColor(#FFFFFF)

